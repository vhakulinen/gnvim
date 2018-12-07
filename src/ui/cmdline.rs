use std::sync::{Arc, Mutex};
use pango;
use gtk;
use gtk::prelude::*;
use gdk::prelude::*;

use nvim_bridge;
use ui::grid::Grid;
use ui::ui::HlDefs;

const MAX_WIDTH: i32 = 650;

struct CmdlineBlock {
    frame: gtk::Frame,
    scrolledwindow: gtk::ScrolledWindow,

    textview: gtk::TextView,

    css_provider: gtk::CssProvider,
}

impl CmdlineBlock {
    fn new() -> Self {
        let css_provider = gtk::CssProvider::new();

        let textview = gtk::TextView::new();
        textview.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let scrolledwindow = gtk::ScrolledWindow::new(None, None);
        scrolledwindow.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        scrolledwindow.set_policy(
            gtk::PolicyType::Automatic,
            gtk::PolicyType::Never);

        let frame = gtk::Frame::new(None);
        frame.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        scrolledwindow.add(&textview);
        frame.add(&scrolledwindow);

        let scrolledwindow_ref = scrolledwindow.clone();
        textview.connect_size_allocate(move |tv, alloc| {
            let h =  tv.get_preferred_height();

            if h.1 > 250 {
                if scrolledwindow_ref.get_size_request().1 == -1 {
                    scrolledwindow_ref.set_size_request(-1, h.1);
                    scrolledwindow_ref.set_policy(
                        gtk::PolicyType::Automatic,
                        gtk::PolicyType::Automatic);
                }

                let adj = scrolledwindow_ref.get_vadjustment().unwrap();
                adj.set_value(adj.get_upper());
            }
        });

        CmdlineBlock {
            frame,
            scrolledwindow,
            textview,
            css_provider,
        }
    }

    fn widget(&self) -> gtk::Widget {
        self.frame.clone().upcast()
    }

    fn show(&mut self, lines: &Vec<(u64, String)>, hl_defs: &HlDefs) {
        self.frame.show();
        let buffer = self.textview.get_buffer().unwrap();
        let mut iter = buffer.get_iter_at_offset(0);

        for (i, line) in lines.iter().enumerate() {
            let hl = hl_defs.get(&line.0).unwrap();

            let markup = hl.pango_markup(
                &line.1,
                &hl_defs.default_fg,
                &hl_defs.default_bg,
                &hl_defs.default_sp,
            );

            if i > 0 {
                buffer.insert(&mut iter, "\n");
            }

            buffer.insert_markup(&mut iter, &markup);
        }
    }

    fn append(&mut self, line: &(u64, String), hl_defs: &HlDefs) {
        let buffer = self.textview.get_buffer().unwrap();

        let mut iter = buffer.get_end_iter();

        let hl = hl_defs.get(&line.0).unwrap();

        let markup = hl.pango_markup(
            &line.1,
            &hl_defs.default_fg,
            &hl_defs.default_bg,
            &hl_defs.default_sp,
            );

        buffer.insert(&mut iter, "\n");
        buffer.insert_markup(&mut iter, &markup);

        // NOTE(ville): After a lot of try and error, this is the only way I
        //              managed to get the scrolling to work properly. This,
        //              in combination of manual use of adjustment in the
        //              scrolled window, makes the scrolling to be not smooth.
        iter.backward_line();
        let mark = buffer.create_mark(None, &iter, false).unwrap();
        self.textview.scroll_to_mark(&mark, 0.0000000001, false, 0.0, 0.0);
    }


    fn hide(&self) {
        self.frame.hide();
        self.scrolledwindow.set_size_request(-1, -1);
        self.scrolledwindow.set_policy(
            gtk::PolicyType::Automatic,
            gtk::PolicyType::Never);

        let buffer = self.textview.get_buffer().unwrap();
        buffer.set_text("");
    }

    fn set_colors(&self, colors: &nvim_bridge::CmdlineColors) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors);
        } else {
            self.set_colors_post20(colors);
        }
    }

    fn set_colors_pre20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "GtkFrame {{
                border: none;
                padding: 5px;
                background: #{bg};
            }}

            GtkTextView {{
                color: #{fg};
                background: #{bg};
            }}",
            fg=colors.fg.to_hex(),
            bg=colors.bg.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    fn set_colors_post20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "frame {{
                padding: 5px;
                background: #{bg};
            }}

            frame > border {{
                border: none;
            }}

            textview, text {{
                color: #{fg};
                background: #{bg};
            }}",
            fg=colors.fg.to_hex(),
            bg=colors.bg.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }
}

struct CmdlineInput {
    frame: gtk::Frame,
    textview: gtk::TextView,
    css_provider: gtk::CssProvider,

    // Content, excluding prompt, firstc etc.
    content: String,

    // Length of the prompt part (firstc, prompt, etc. things before
    // actual content) in chars.
    prompt_len: i32,
    // Cursor position in `content` (in bytes).
    cursor_pos: usize,
    // Level from the latest `cmdline_show`.
    current_level: u64,
}

impl CmdlineInput {
    fn new() -> Self {
        let css_provider = gtk::CssProvider::new();

        let textview = gtk::TextView::new();
        textview.set_editable(false);
        textview.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Catch all button events to prevent selection of text etc.
        textview.connect_button_press_event(|_, _| {
            Inhibit(true)
        });

        // Wrap the textview into a frame, mainly to add some padding (with css).
        let frame = gtk::Frame::new(None);
        frame.add(&textview);
        frame.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        CmdlineInput {
            frame,
            textview,
            css_provider,

            content: String::new(),
            prompt_len: 0,
            cursor_pos: 0,
            current_level: 0,
        }
    }

    fn widget(&self) -> gtk::Widget {
        self.frame.clone().upcast()
    }

    fn set_text(&mut self, content: &nvim_bridge::CmdlineShow, hl_defs: &HlDefs) {
        let mut buffer = self.textview.get_buffer().unwrap();

        // Reset the buffer.
        buffer.set_text("");
        // Get iter from the beginning.
        let mut iter = buffer.get_iter_at_offset(0);
        // Write the prompt.
        let prompt = format!("{}{}{}",
                             content.firstc,
                             " ".repeat(content.indent as usize),
                             content.prompt);
        buffer.insert(&mut iter, &prompt);
        self.prompt_len = prompt.chars().count() as i32;

        // Write the contents.
        for item in content.content.iter() {
            let hl = hl_defs.get(&item.0).unwrap();

            let markup = hl.pango_markup(
                &item.1,
                &hl_defs.default_fg,
                &hl_defs.default_bg,
                &hl_defs.default_sp,
            );

            buffer.insert_markup(&mut iter, &markup);
        }

        self.current_level = content.level;
        self.content = content.content.iter().map(|c| c.1.clone()).collect();

        self.textview.grab_focus();

        self.cursor_pos = content.pos as usize;
        self.ensure_cursor_pos();
    }

    fn show_special_char(&mut self, ch: String, shift: bool) {
        let buffer = self.textview.get_buffer().unwrap();
        let mark_insert = buffer.get_insert().unwrap();
        let mut iter = buffer.get_iter_at_mark(&mark_insert);
        buffer.insert(&mut iter, &ch);
        iter.backward_char();
    }

    fn set_colors(&self, colors: &nvim_bridge::CmdlineColors) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors);
        } else {
            self.set_colors_post20(colors);
        }
    }

    fn set_colors_pre20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "GtkFrame {{
                border: none;
                padding: 5px;
                background: #{bg};
            }}

            GtkTextView {{
                color: #{fg};
                background: #{bg};
            }}",
            fg=colors.fg.to_hex(),
            bg=colors.bg.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    fn set_colors_post20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "frame {{
                padding: 5px;
                background: #{bg};
            }}

            frame > border {{
                border: none;
            }}

            textview, text {{
                caret-color: #{fg};
                color: #{fg};
                background: #{bg};
            }}",
            fg=colors.fg.to_hex(),
            bg=colors.bg.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    fn set_cursor(&mut self, pos: usize, level: u64) {
        if (level != self.current_level) {
            return;
        }

        self.cursor_pos = pos;
        self.ensure_cursor_pos();
    }

    fn ensure_cursor_pos(&self) {
        let buffer = self.textview.get_buffer().unwrap();
        let mut iter = buffer.get_start_iter();

        let pos = self.content.split_at(self.cursor_pos).0.chars().count();

        iter.forward_chars(self.prompt_len + pos as i32);
        buffer.place_cursor(&iter);

        let mark = buffer.create_mark(None, &iter, false).unwrap();
        self.textview.scroll_to_mark(&mark, 0.1, false, 0.0, 0.0);
    }
}

pub struct Cmdline {
    css_provider: gtk::CssProvider,
    box_: gtk::Box,
    fixed: gtk::Fixed,

    hl_defs: Arc<Mutex<HlDefs>>,

    input: CmdlineInput,
    block: CmdlineBlock,

    show_block: bool,
}

impl Cmdline {
    pub fn new(parent: &gtk::Overlay, hl_defs: Arc<Mutex<HlDefs>>) -> Self {
        let css_provider = gtk::CssProvider::new();

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let block = CmdlineBlock::new();
        box_.pack_start(&block.widget(), true, true, 0);

        let input = CmdlineInput::new();
        box_.pack_start(&input.widget(), true, true, 0);

        // Depending on which gtk version we're running, the top container
        // for cmdline varies. It'll be GtkBox either way.
        let mut container: gtk::Widget = box_.clone().upcast();
        if gtk::get_minor_version() < 20 {
            // For gtk versions < 20, we'll need to have a GtkFrame around our
            // box where the input/block is to get padding, but to get box-shadow,
            // we'll have to have another container around the frame because
            // frame it self cant have shadows.
            let outer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let frame = gtk::Frame::new(None);
            frame.get_style_context()
                .unwrap()
                .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            outer_box.get_style_context()
                .unwrap()
                .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            frame.add(&box_);
            outer_box.add(&frame);
            container = outer_box.clone().upcast();
        }

        // Add style provider to the container.
        container.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let fixed = gtk::Fixed::new();
        fixed.put(&container, 0, 0);

        parent.add_overlay(&fixed);

        let fixed_ref = fixed.clone();
        let container_ref = container.clone();
        parent.connect_size_allocate(move |_, alloc| {

            // Make sure we'll fit to the available space.
            let width = MAX_WIDTH.min(alloc.width);
            container_ref.set_size_request(width, -1);

            let x = alloc.width / 2 - width / 2;
            fixed_ref.move_(&container, x, 0);
        });

        Cmdline{
            css_provider,
            box_,
            fixed,
            hl_defs,
            input,
            block,
            show_block: false,
        }
    }

    pub fn set_colors(&self, colors: &nvim_bridge::CmdlineColors) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors);
        } else {
            self.set_colors_post20(colors);
        }

        self.input.set_colors(colors);
        self.block.set_colors(colors);
    }

    fn set_colors_post20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "box {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
                background: #{bg};
                padding: 6px;
            }}",
            bg=colors.border.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    fn set_colors_pre20(&self, colors: &nvim_bridge::CmdlineColors) {
        let css = format!(
            "GtkBox {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}

            GtkFrame {{
                background: #{bg};
                padding: 6px;
                border: none;
            }}",
            bg=colors.border.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    pub fn hide(&self) {
        self.fixed.hide();
    }

    pub fn show(&mut self, content: &nvim_bridge::CmdlineShow) {
        let hl_defs = self.hl_defs.lock().unwrap();
        self.input.set_text(content, &hl_defs);
        self.fixed.show_all();

        if !self.show_block {
            self.block.hide();
        }
    }

    pub fn show_special_char(&mut self, ch: String, shift: bool, level: u64) {
        self.input.show_special_char(ch, shift);
    }

    pub fn set_font(&mut self, font: &pango::FontDescription) {
        gtk::WidgetExt::override_font(&self.fixed, font);

        // Some tricks to make sure the input has correct height after
        // font change.
        self.fixed.show();
        let f = self.fixed.clone();
        gtk::idle_add(move || {
            f.hide();
            Continue(false)
        });
    }

    pub fn set_pos(&mut self, pos: u64, level: u64) {
        self.input.set_cursor(pos as usize, level);
    }

    pub fn show_block(&mut self, lines: &Vec<(u64, String)>) {
        let hl_defs = self.hl_defs.lock().unwrap();
        self.block.show(lines, &hl_defs);
        self.show_block = true;
    }

    pub fn hide_block(&mut self) {
        self.block.hide();
        self.show_block = false;
    }

    pub fn block_append(&mut self, line: &(u64, String)) {
        let hl_defs = self.hl_defs.lock().unwrap();
        self.block.append(line, &hl_defs);
    }
}
