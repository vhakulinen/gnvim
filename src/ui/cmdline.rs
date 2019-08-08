use std::sync::{Arc, Mutex};

use gtk;
use gtk::prelude::*;

use neovim_lib::neovim::Neovim;

use nvim_bridge;
use ui::common::calc_line_space;
use ui::font::{Font, FontUnit};
use ui::ui::HlDefs;
use ui::wildmenu::Wildmenu;

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

        let scrolledwindow = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        scrolledwindow
            .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);

        let frame = gtk::Frame::new(None);

        scrolledwindow.add(&textview);
        frame.add(&scrolledwindow);

        textview.connect_size_allocate(clone!(scrolledwindow => move |tv, _| {
            let h = tv.get_preferred_height();

            if h.1 > 250 {
                if scrolledwindow.get_size_request().1 == -1 {
                    scrolledwindow.set_size_request(-1, h.1);
                    scrolledwindow.set_policy(
                        gtk::PolicyType::Automatic,
                        gtk::PolicyType::Automatic,
                    );
                }

                let adj = scrolledwindow.get_vadjustment().unwrap();
                adj.set_value(adj.get_upper());
            }
        }));

        add_css_provider!(&css_provider, textview, scrolledwindow, frame);

        CmdlineBlock {
            frame,
            scrolledwindow,
            textview,
            css_provider,
        }
    }

    fn set_line_space(&self, space: i64) {
        let (above, below) = calc_line_space(space);
        self.textview.set_pixels_above_lines(above as i32);
        self.textview.set_pixels_below_lines(below as i32);
    }

    fn widget(&self) -> gtk::Widget {
        self.frame.clone().upcast()
    }

    fn show(&mut self, show: &nvim_bridge::CmdlineBlockShow, hl_defs: &HlDefs) {
        self.frame.show();
        let buffer = self.textview.get_buffer().unwrap();
        let mut iter = buffer.get_iter_at_offset(0);

        for (i, line) in show.lines.iter().enumerate() {
            let mut markup = String::new();
            for seg in line.iter() {
                let hl = hl_defs.get(&seg.0).unwrap();

                markup += &hl.pango_markup(
                    &seg.1,
                    &hl_defs.default_fg,
                    &hl_defs.default_bg,
                    &hl_defs.default_sp,
                );
            }

            if i > 0 {
                buffer.insert(&mut iter, "\n");
            }

            buffer.insert_markup(&mut iter, &markup);
        }
    }

    fn append(
        &mut self,
        append: &nvim_bridge::CmdlineBlockAppend,
        hl_defs: &HlDefs,
    ) {
        let buffer = self.textview.get_buffer().unwrap();

        let mut iter = buffer.get_end_iter();

        let markup: String = append
            .line
            .iter()
            .map(|seg| {
                let hl = hl_defs.get(&seg.0).unwrap();
                hl.pango_markup(
                    &seg.1,
                    &hl_defs.default_fg,
                    &hl_defs.default_bg,
                    &hl_defs.default_sp,
                )
            })
            .collect();

        buffer.insert(&mut iter, "\n");
        buffer.insert_markup(&mut iter, &markup);

        // NOTE(ville): After a lot of try and error, this is the only way I
        //              managed to get the scrolling to work properly. This,
        //              in combination of manual use of adjustment in the
        //              scrolled window, makes the scrolling to be not smooth.
        iter.backward_line();
        let mark = buffer.create_mark(None, &iter, false).unwrap();
        self.textview
            .scroll_to_mark(&mark, 0.0000000001, false, 0.0, 0.0);
    }

    fn hide(&self) {
        self.frame.hide();
        self.scrolledwindow.set_size_request(-1, -1);
        self.scrolledwindow
            .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);

        let buffer = self.textview.get_buffer().unwrap();
        buffer.set_text("");
    }

    fn set_colors(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors, hl_defs);
        } else {
            self.set_colors_post20(colors, hl_defs);
        }
    }

    fn set_colors_pre20(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
        let css = format!(
            "GtkFrame {{
                border: none;
                padding: 5px;
                background: #{bg};
                border-radius: 0;
            }}

            GtkTextView {{
                color: #{fg};
                background: #{bg};
            }}",
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_colors_post20(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
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
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }
}

struct CmdlineInput {
    frame: gtk::Frame,
    textview: gtk::TextView,
    css_provider: gtk::CssProvider,

    /// Content, excluding prompt, firstc etc.
    content: String,

    /// Length of the prompt part (firstc, prompt, etc. things before
    /// actual content) in chars.
    prompt_len: i32,
    /// Cursor position in `content` (in bytes).
    cursor_pos: usize,
    /// Level from the latest `cmdline_show`.
    current_level: u64,
}

impl CmdlineInput {
    fn new() -> Self {
        let css_provider = gtk::CssProvider::new();

        let textview = gtk::TextView::new();
        textview.set_editable(false);

        // Catch all button events to prevent selection of text etc.
        textview.connect_button_press_event(|_, _| Inhibit(true));

        let scroll = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
        scroll.add(&textview);

        // Wrap the textview into a frame, mainly to add some padding (with css).
        let frame = gtk::Frame::new(None);
        frame.add(&scroll);

        add_css_provider!(&css_provider, frame, textview);

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

    fn set_text(
        &mut self,
        content: &nvim_bridge::CmdlineShow,
        hl_defs: &HlDefs,
    ) {
        let buffer = self.textview.get_buffer().unwrap();

        // Reset the buffer.
        buffer.set_text("");
        // Get iter from the beginning.
        let mut iter = buffer.get_iter_at_offset(0);
        // Write the prompt.
        let prompt = format!(
            "{}{}{}",
            content.firstc,
            " ".repeat(content.indent as usize),
            content.prompt
        );
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

        self.set_cursor(content.pos as usize, content.level);
    }

    fn show_special_char(&mut self, ch: String, _shift: bool, _level: u64) {
        // TODO(ville): What to do with `_shift` and `_level`?
        let buffer = self.textview.get_buffer().unwrap();
        let mark_insert = buffer.get_insert().unwrap();
        let mut iter = buffer.get_iter_at_mark(&mark_insert);
        buffer.insert(&mut iter, &ch);
        iter.backward_char();
    }

    fn set_colors(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors, hl_defs);
        } else {
            self.set_colors_post20(colors, hl_defs);
        }
    }

    fn set_colors_pre20(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
        let css = format!(
            "GtkFrame {{
                border: none;
                padding: 5px;
                background: #{bg};
                border-radius: 0;
            }}

            GtkTextView {{
                color: #{fg};
                background: #{bg};
            }}",
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_colors_post20(
        &self,
        colors: &nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
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
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_cursor(&mut self, pos: usize, level: u64) {
        if level != self.current_level {
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

    fn set_line_space(&self, space: i64) {
        let (above, below) = calc_line_space(space);
        self.textview.set_pixels_above_lines(above as i32);
        self.textview.set_pixels_below_lines(below as i32);
    }
}

pub struct Cmdline {
    css_provider: gtk::CssProvider,
    fixed: gtk::Fixed,

    input: CmdlineInput,
    block: CmdlineBlock,
    wildmenu: Wildmenu,

    /// If the block should be shown or not.
    show_block: bool,
    /// If the wildmenu should be shown or not.
    show_wildmenu: bool,

    colors: nvim_bridge::CmdlineColors,
    /// Our font. This is inherited to input, block and wildmenu through our
    /// styles.
    font: Font,
}

impl Cmdline {
    pub fn new(parent: &gtk::Overlay, nvim: Arc<Mutex<Neovim>>) -> Self {
        let css_provider = gtk::CssProvider::new();

        // Inner box contains cmdline block and input.
        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let input = CmdlineInput::new();
        let block = CmdlineBlock::new();
        inner_box.pack_start(&block.widget(), true, true, 0);
        inner_box.pack_start(&input.widget(), true, true, 0);

        // Frame will contain inner_box. This is so we can add some padding
        // around them without adding padding around the wildmenu.
        let frame = gtk::Frame::new(None);
        frame.add(&inner_box);

        let wildmenu = Wildmenu::new(nvim.clone());

        // box_ is the actual container for cmdline and wildmenu.
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);
        box_.pack_start(&frame, true, true, 0);
        box_.pack_start(&wildmenu.widget(), true, true, 0);

        add_css_provider!(&css_provider, box_, frame, inner_box);

        let fixed = gtk::Fixed::new();
        fixed.put(&box_, 0, 0);

        parent.add_overlay(&fixed);

        parent.connect_size_allocate(clone!(fixed, box_ => move |_, alloc| {
            // Make sure we'll fit to the available space.
            let width = MAX_WIDTH.min(alloc.width);
            box_.set_size_request(width, -1);

            let x = alloc.width / 2 - width / 2;
            fixed.move_(&box_, x, 0);
        }));

        Cmdline {
            css_provider,
            fixed,
            input,
            block,
            wildmenu,
            show_block: false,
            show_wildmenu: false,
            font: Font::default(),
            colors: nvim_bridge::CmdlineColors::default(),
        }
    }

    pub fn set_colors(
        &mut self,
        colors: nvim_bridge::CmdlineColors,
        hl_defs: &HlDefs,
    ) {
        self.input.set_colors(&colors, hl_defs);
        self.block.set_colors(&colors, hl_defs);
        self.colors = colors;

        self.set_styles(hl_defs);
    }

    fn set_styles(&self, hl_defs: &HlDefs) {
        if gtk::get_minor_version() < 20 {
            self.set_styles_pre20(hl_defs);
        } else {
            self.set_styles_post20(hl_defs);
        }
    }

    fn set_styles_post20(&self, hl_defs: &HlDefs) {
        let css = format!(
            "{font_wild}

            frame > border {{
                border: none;
            }}

            frame {{
                background: #{bg};
                padding: 6px;
            }}

            box {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}

            frame > box {{
                box-shadow: none;
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Point),
            bg = self.colors.border.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_styles_pre20(&self, hl_defs: &HlDefs) {
        let css = format!(
            "{font_wild}

            GtkBox {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}

            GtkFrame > GtkBox {{
                box-shadow: none;
            }}

            GtkFrame {{
                background: #{bg};
                padding: 6px;
                border: none;
                border-radius: 0;
            }}",
            font_wild = self.font.as_wild_css(FontUnit::Pixel),
            bg = self.colors.border.unwrap_or(hl_defs.default_bg).to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    pub fn hide(&self) {
        self.fixed.hide();
    }

    pub fn show(
        &mut self,
        content: &nvim_bridge::CmdlineShow,
        hl_defs: &HlDefs,
    ) {
        self.input.set_text(content, hl_defs);
        self.fixed.show_all();

        if !self.show_block {
            self.block.hide();
        }

        if !self.show_wildmenu {
            self.wildmenu.hide();
        }
    }

    pub fn show_special_char(&mut self, ch: String, shift: bool, level: u64) {
        self.input.show_special_char(ch, shift, level);
    }

    pub fn set_line_space(&self, space: i64) {
        self.block.set_line_space(space);
        self.input.set_line_space(space);
    }

    pub fn set_font(&mut self, font: Font, hl_defs: &HlDefs) {
        self.font = font;
        self.set_styles(hl_defs);

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

    pub fn show_block(
        &mut self,
        show: &nvim_bridge::CmdlineBlockShow,
        hl_defs: &HlDefs,
    ) {
        self.block.show(show, hl_defs);
        self.show_block = true;
    }

    pub fn hide_block(&mut self) {
        self.block.hide();
        self.show_block = false;
    }

    pub fn block_append(
        &mut self,
        line: &nvim_bridge::CmdlineBlockAppend,
        hl_defs: &HlDefs,
    ) {
        self.block.append(line, &hl_defs);
    }

    pub fn wildmenu_show(&mut self, items: &Vec<String>) {
        self.show_wildmenu = true;
        self.wildmenu.set_items(items);
        self.wildmenu.show();

        self.fixed.check_resize();
    }

    pub fn wildmenu_hide(&mut self) {
        self.show_wildmenu = false;
        self.wildmenu.clear();
        self.wildmenu.hide();

        self.fixed.check_resize();
    }

    pub fn wildmenu_select(&mut self, item_num: i64) {
        self.wildmenu.select(item_num as i32);
    }

    pub fn wildmenu_set_colors(
        &self,
        colors: &nvim_bridge::WildmenuColors,
        hl_defs: &HlDefs,
    ) {
        self.wildmenu.set_colors(colors, hl_defs);
    }
}
