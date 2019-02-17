use std::sync::{Arc, Mutex};

use gdk;
use gio;
use glib;
use gtk;
use gtk::prelude::*;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;
use pango;

use nvim_bridge::{CompletionItem, PmenuColors};
use thread_guard::ThreadGuard;
use ui::color::Color;
use ui::common::calc_line_space;
use ui::common::{
    get_preferred_horizontal_position, get_preferred_vertical_position,
};
use ui::font::{Font, FontUnit};
use ui::ui::HlDefs;

macro_rules! icon {
    ($file:expr, $color:expr) => {
        format!(include_str!($file), $color,)
    };
}

/// Maximum height of completion menu.
const MAX_HEIGHT: i32 = 500;
/// Fixed width of completion menu.
const WIDTH_NO_DETAILS: i32 = 430;
const WIDTH_WITH_DETAILS: i32 = 660;

/// Wraps completion item into a structure which contains the item and some
/// of the widgets to display it.
struct CompletionItemWidgetWrap {
    /// Actual completion item.
    item: CompletionItem,
    /// Label displaying `info` for this item in the list.
    info: gtk::Label,
    /// Label displaying `menu` for this item in the list.
    menu: gtk::Label,
    /// Image of the item in the row.
    kind: gtk::Image,
    /// Root container.
    row: gtk::ListBoxRow,
}

struct State {
    /// Currently selected item.
    selected: i32,
    /// All items in current popupmenu.
    items: Vec<CompletionItemWidgetWrap>,
    /// Size available for the popupmenu to use (width and height).
    available_size: Option<gdk::Rectangle>,
    /// Our anchor position where the popupmenu should be "pointing" to.
    anchor: gdk::Rectangle,

    width: i32,
}

impl Default for State {
    fn default() -> Self {
        State {
            selected: -1,
            items: vec![],
            available_size: None,
            anchor: gdk::Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },

            width: WIDTH_NO_DETAILS,
        }
    }
}

pub struct Popupmenu {
    /// Box that contains all the "content". This box is placed in side the
    /// layout container.
    box_: gtk::Box,
    /// Top level container.
    layout: gtk::Layout,
    /// Scrolled window that contains the list box that displays all the items.
    scrolled_list: gtk::ScrolledWindow,
    /// Scrolled window that contains the info label for full info view.
    scrolled_info: gtk::ScrolledWindow,
    /// List box that contains all the completion items.
    list: gtk::ListBox,
    /// Style provider for all internal widgets.
    css_provider: gtk::CssProvider,

    /// Flag telling if the info label is shown.
    info_shown: bool,
    /// Label for displaying full info of a completion item.
    info_label: gtk::Label,

    /// State that is in Arc because its passed into widget signal handlers.
    state: Arc<ThreadGuard<State>>,

    /// Our colors.
    colors: PmenuColors,
    /// Our font.
    font: Font,

    /// Line spacing.
    line_space: i64,
}

impl Popupmenu {
    /// Creates a new popupmenu.
    ///
    /// * `parent` - Overlay where popupmenu is placed. Ideally, this overlay
    ///              is where all the (neovim) grids are drawn.
    /// * `nvim` - Neovim instance. Popupmenu will instruct neovim to act on
    ///            user interaction.
    pub fn new(parent: &gtk::Overlay, nvim: Arc<Mutex<Neovim>>) -> Self {
        let css_provider = gtk::CssProvider::new();

        let info_label = gtk::Label::new("");
        info_label.set_halign(gtk::Align::Start);
        info_label.set_valign(gtk::Align::Start);
        info_label.set_xalign(0.0);
        info_label.set_line_wrap(true);
        info_label.set_line_wrap_mode(pango::WrapMode::WordChar);
        gtk::WidgetExt::set_name(&info_label, "info-label");

        // Because we're setting valign and halign to the info label, we'll
        // need to have some container in between the label and scrolled window.
        // Other wise there'll be some black boxes when scroll bars are needed.
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        info_box.add(&info_label);

        let scrolled_info = gtk::ScrolledWindow::new(None, None);
        scrolled_info.add(&info_box);
        scrolled_info
            .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let list = gtk::ListBox::new();
        list.set_valign(gtk::Align::Start);
        list.set_selection_mode(gtk::SelectionMode::Single);

        let scrolled_list = gtk::ScrolledWindow::new(None, None);
        scrolled_list.add(&list);
        scrolled_list
            .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        box_.pack_start(&scrolled_list, true, true, 0);
        box_.pack_start(&scrolled_info, true, true, 0);
        box_.set_size_request(WIDTH_NO_DETAILS, MAX_HEIGHT);
        box_.set_homogeneous(true);

        add_css_provider!(
            &css_provider,
            info_label,
            scrolled_info,
            list,
            scrolled_list,
            box_,

            // In gtk 3.18, the list box it self can't have borders,
            // so we'll have to add the border to its parent (which is the
            // viewport that scorlled window adds). This aint perfect,
            // but I didn't any find better solutions.
            scrolled_list.get_child().unwrap()
        );

        let state = Arc::new(ThreadGuard::new(State::default()));

        let state_ref = state.clone();
        let nvim_ref = nvim.clone();
        // When a row is activated (by mouse click), notify neovim to change
        // the selection to the activated row.
        list.connect_row_activated(move |_, row| {
            let state = state_ref.borrow_mut();
            let new = row.get_index();

            let op = if new > state.selected {
                "<C-n>"
            } else {
                "<C-p>"
            };

            let mut payload = String::new();
            for _ in 0..(new - state.selected).abs() {
                payload.push_str(op)
            }

            let mut nvim = nvim_ref.lock().unwrap();
            nvim.input(payload.as_str()).unwrap();
        });

        let nvim_ref = nvim.clone();
        // On (mouse) button press...
        list.connect_button_press_event(move |_, e| {
            // ...check if the button press is double click.
            if e.get_event_type() == gdk::EventType::DoubleButtonPress {
                // And if so, tell neovim to select the current completion item.
                let mut nvim = nvim_ref.lock().unwrap();
                nvim.input("<C-y>").unwrap();
            }

            Inhibit(false)
        });

        // TODO(ville): Should use gtk::Fixed here.
        let layout = gtk::Layout::new(None, None);
        layout.put(&box_, 0, 0);
        layout.show_all();
        scrolled_info.hide();

        let state_ref = state.clone();
        layout.connect_size_allocate(move |_, alloc| {
            let mut state = state_ref.borrow_mut();
            state.available_size = Some(*alloc);
        });

        let state_ref = state.clone();
        let layout_ref = layout.clone();
        let scrolled_list_ref = scrolled_list.clone();
        let scrolled_info_ref = scrolled_info.clone();
        box_.connect_size_allocate(move |box_, alloc| {
            let state = state_ref.borrow();
            let layout = layout_ref.clone();

            if let Some(area) = state.available_size {
                let pos = state.anchor;

                let (x, width) =
                    get_preferred_horizontal_position(&area, &pos, state.width);
                let (y, height) = get_preferred_vertical_position(
                    &area,
                    &pos,
                    alloc.height.min(MAX_HEIGHT),
                );

                layout.move_(box_, x, y);

                box_.set_size_request(width, height);

                // If we moved the popupmenu above the achor position, make
                // sure our contents are aligned to the bottom so there is not
                // cap between the achor and the content it self.
                if y < pos.y {
                    // Use get_child to get the viewport which is between
                    // the scrolled window and the actual widget that is
                    // inside it.
                    scrolled_list_ref
                        .get_child()
                        .unwrap()
                        .set_valign(gtk::Align::End);
                    scrolled_info_ref
                        .get_child()
                        .unwrap()
                        .set_valign(gtk::Align::End);
                } else {
                    scrolled_list_ref
                        .get_child()
                        .unwrap()
                        .set_valign(gtk::Align::Start);
                    scrolled_info_ref
                        .get_child()
                        .unwrap()
                        .set_valign(gtk::Align::Start);
                }
            }
        });

        parent.add_overlay(&layout);
        // Hide the layout initially so it wont catch any input events that
        // should go to the girds.
        layout.hide();

        Popupmenu {
            box_,
            layout,
            css_provider,
            list,
            scrolled_list,
            scrolled_info,
            info_label,
            state,
            info_shown: false,
            colors: PmenuColors::default(),
            font: Font::default(),
            line_space: 0,
        }
    }

    pub fn toggle_show_info(&mut self) {
        let mut state = self.state.borrow_mut();

        if state.selected == -1 {
            return;
        }

        self.info_shown = !self.info_shown;

        if let Some(item) = state.items.get(state.selected as usize) {
            item.info.set_visible(!self.info_shown);
            item.menu.set_visible(!self.info_shown);

            if item.item.info.len() == 0 {
                item.info.set_visible(false);
            }
        }

        if !self.info_shown {
            let adj = self.scrolled_info.get_vadjustment().unwrap();
            adj.set_value(0.0);
            // TODO(ville): There is a bug in GTK+ and some adjustment animations,
            //              where the adjustment's value is set back to upper - page-size
            //              if the user has "overshot" the scrolling. Work around this.
        }

        self.scrolled_info.set_visible(self.info_shown);

        state.width = if self.info_shown {
            WIDTH_WITH_DETAILS
        } else {
            WIDTH_NO_DETAILS
        };

        self.box_.set_size_request(state.width, MAX_HEIGHT);
    }

    /// Hides the popupmenu.
    pub fn hide(&mut self) {
        self.layout.hide();
    }

    /// Shows the popupmenu.
    pub fn show(&self) {
        self.layout.show();
        self.box_.check_resize();
    }

    /// Sets the anchor point for popupmenu.
    pub fn set_anchor(&self, rect: gdk::Rectangle) {
        let mut state = self.state.borrow_mut();
        self.layout.move_(&self.box_, rect.x, rect.y + rect.height);
        state.anchor = rect;
    }

    pub fn set_items(&mut self, items: Vec<CompletionItem>, hl_defs: &HlDefs) {
        let mut state = self.state.borrow_mut();
        state.selected = -1;

        while let Some(item) = state.items.pop() {
            item.row.destroy();
        }

        for item in items.into_iter() {
            let wrap = create_completionitem_widget(
                item,
                &self.css_provider,
                &self.colors.fg.unwrap_or(hl_defs.default_fg),
            );

            self.list.add(&wrap.row);
            state.items.push(wrap);
        }
        self.list.show_all();
    }

    pub fn select(&mut self, item_num: i32, hl_defs: &HlDefs) {
        let mut state = self.state.borrow_mut();

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(false);
                item.menu.set_visible(false);

                // Update the `kind` icon with defualt fg color.
                let buf = get_icon_pixbuf(
                    &item.item.kind,
                    &self.colors.fg.unwrap_or(hl_defs.default_fg),
                );
                item.kind.set_from_pixbuf(&buf);
            }
        }

        let prev = state.selected;
        state.selected = item_num;

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(!self.info_shown);
                item.menu.set_visible(!self.info_shown);

                if item.item.info.len() == 0 {
                    item.info.set_visible(false);
                }

                self.list.select_row(&item.row);
                item.row.grab_focus();

                // Update the `kind` icon with "selected" fg color.
                let buf = get_icon_pixbuf(
                    &item.item.kind,
                    &self.colors.sel_fg.unwrap_or(hl_defs.default_fg),
                );
                item.kind.set_from_pixbuf(&buf);

                // If we went from no selection to state where the last item
                // is selected, we'll have to do some extra work to make sure
                // that the whole item is visible.
                let max = state.items.len() as i32 - 1;
                let adj = self.scrolled_list.get_vadjustment().unwrap();
                if prev == -1 && state.selected == max {
                    adj.set_value(adj.get_upper());
                }

                let newline = if item.item.menu.len() > 0 && item.item.info.len() > 0 {
                    "\n"
                } else {
                    ""
                };

                self.info_label.set_text(&format!(
                    "{}{}{}",
                    item.item.menu, newline, item.item.info
                ));
            }
        } else {
            self.list.unselect_all();
            self.info_label.set_text("");

            // If selecteion is removed, move the srolled window to the top.
            let adj = self.scrolled_list.get_vadjustment().unwrap();
            gtk::idle_add(move || {
                adj.set_value(0.0);
                Continue(false)
            });
        }
    }

    pub fn set_colors(&mut self, colors: PmenuColors, hl_defs: &HlDefs) {
        self.colors = colors;
        self.set_styles(hl_defs);
    }

    pub fn set_line_space(&mut self, space: i64, hl_defs: &HlDefs) {
        self.line_space = space;
        self.set_styles(hl_defs);

        // Set line space to the info_label with pango attrs.
        let attrs = pango::AttrList::new();
        let attr =
            pango::Attribute::new_rise(self.line_space as i32 * pango::SCALE)
                .unwrap();
        attrs.insert(attr);
        self.info_label.set_attributes(&attrs);
    }

    fn set_styles(&self, hl_defs: &HlDefs) {
        if gtk::get_minor_version() < 20 {
            self.set_styles_pre20(hl_defs);
        } else {
            self.set_styles_post20(hl_defs);
        }
    }

    fn set_styles_post20(&self, hl_defs: &HlDefs) {
        let (above, below) = calc_line_space(self.line_space);

        let css = format!(
            "{font_wild}

            grid, list, row, label {{
                color: #{normal_fg};
                background-color: #{normal_bg};
                outline: none;
            }}

            #info-label, list {{
                border: 1px solid #{normal_fg};
            }}

            row {{
                padding-top: {above}px;
                padding-bottom: {below}px;
            }}

            row:selected, row:selected > grid, row:selected > grid > label {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}

            box {{
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Point),
            normal_fg = self.colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            normal_bg = self.colors.bg.unwrap_or(hl_defs.default_bg).to_hex(),
            selected_bg =
                self.colors.sel_bg.unwrap_or(hl_defs.default_bg).to_hex(),
            selected_fg =
                self.colors.sel_fg.unwrap_or(hl_defs.default_fg).to_hex(),
            above = above.max(0),
            below = below.max(0),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_styles_pre20(&self, hl_defs: &HlDefs) {
        let (above, below) = calc_line_space(self.line_space);

        let css = format!(
            "{font_wild}

            GtkGrid, GtkListBox, GtkListBoxRow, GtkLabel {{
                color: #{normal_fg};
                background-color: #{normal_bg};
                outline: none;
            }}

            GtkViewport {{
                border-radius: 0px;
            }}

            #info-label, GtkViewport {{
                border: 1px solid #{normal_fg};
            }}

            GtkListBoxRow {{
                padding-top: {above}px;
                padding-bottom: {below}px;
            }}

            GtkListBoxRow:selected,
            GtkListBoxRow:selected > GtkGrid,
            GtkListBoxRow:selected > GtkGrid > GtkLabel {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Pixel),
            normal_fg = self.colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            normal_bg = self.colors.bg.unwrap_or(hl_defs.default_bg).to_hex(),
            selected_bg =
                self.colors.sel_bg.unwrap_or(hl_defs.default_bg).to_hex(),
            selected_fg =
                self.colors.sel_fg.unwrap_or(hl_defs.default_fg).to_hex(),
            above = above.max(0),
            below = below.max(0),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    pub fn set_font(&mut self, font: Font, hl_defs: &HlDefs) {
        self.font = font;
        self.set_styles(hl_defs);
    }
}

fn create_completionitem_widget(
    item: CompletionItem,
    css_provider: &gtk::CssProvider,
    fg: &Color,
) -> CompletionItemWidgetWrap {
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);

    let buf = get_icon_pixbuf(&item.kind.as_str(), &fg);
    let kind = gtk::Image::new_from_pixbuf(&buf);

    kind.set_halign(gtk::Align::Start);
    kind.set_margin_start(5);
    kind.set_margin_end(5);
    grid.attach(&kind, 0, 0, 1, 1);

    let menu = gtk::Label::new(item.menu.as_str());
    menu.set_halign(gtk::Align::End);
    menu.set_hexpand(true);
    menu.set_margin_start(5);
    menu.set_margin_end(5);
    menu.set_ellipsize(pango::EllipsizeMode::End);
    grid.attach(&menu, 2, 0, 1, 1);

    let word = gtk::Label::new(item.word.as_str());
    word.set_ellipsize(pango::EllipsizeMode::End);
    grid.attach(&word, 1, 0, 1, 1);

    let info = gtk::Label::new(shorten_info(&item.info).as_str());
    info.set_halign(gtk::Align::Start);
    info.set_ellipsize(pango::EllipsizeMode::End);

    info.connect_realize(|info| {
        info.hide();
    });
    menu.connect_realize(|menu| {
        menu.hide();
    });

    grid.attach(&info, 1, 1, 2, 1);

    // NOTE(ville): We only need to explicitly create this row widget
    //              so we can set css provider to it.
    let row = gtk::ListBoxRow::new();
    row.add(&grid);

    add_css_provider!(css_provider, grid, kind, word, info, row, menu);

    CompletionItemWidgetWrap {
        item,
        info,
        row,
        kind,
        menu,
    }
}

/// Returns first line of `info`.
fn shorten_info(info: &String) -> String {
    let lines = info.split("\n").collect::<Vec<&str>>();
    let first_line = lines.get(0).unwrap();
    first_line.to_string()
}

fn get_icon_pixbuf(kind: &str, color: &Color) -> gdk_pixbuf::Pixbuf {
    let contents = get_icon_name_for_kind(kind, &color);
    let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
        contents.as_bytes(),
    ));
    let buf = gdk_pixbuf::Pixbuf::new_from_stream(&stream, None).unwrap();

    buf
}

fn get_icon_name_for_kind(kind: &str, color: &Color) -> String {
    let color = color.to_hex();

    match kind {
        "method" | "function" | "constructor" => {
            icon!("../../assets/icons/box.svg", color)
        }
        "field" => icon!("../../assets/icons/chevrons-right.svg", color),
        "event" => icon!("../../assets/icons/zap.svg", color),
        "operator" => icon!("../../assets/icons/sliders.svg", color),
        "variable" => icon!("../../assets/icons/disc.svg", color),
        "class" => icon!("../../assets/icons/share-2.svg", color),
        "interface" => icon!("../../assets/icons/book-open.svg", color),
        "struct" => icon!("../../assets/icons/align-left.svg", color),
        "type parameter" => icon!("../../assets/icons/type.svg", color),
        "module" => icon!("../../assets/icons/code.svg", color),
        "property" => icon!("../../assets/icons/key.svg", color),
        "unit" => icon!("../../assets/icons/compass.svg", color),
        "constant" => icon!("../../assets/icons/shield.svg", color),
        "value" | "enum" => icon!("../../assets/icons/database.svg", color),
        "enum member" => icon!("../../assets/icons/tag.svg", color),
        "keyword" => icon!("../../assets/icons/link-2.svg", color),
        "text" => icon!("../../assets/icons/at-sign.svg", color),
        "color" => icon!("../../assets/icons/aperture.svg", color),
        "file" => icon!("../../assets/icons/file.svg", color),
        "reference" => icon!("../../assets/icons/link.svg", color),
        "snippet" => icon!("../../assets/icons/file-text.svg", color),
        "folder" => icon!("../../assets/icons/folder.svg", color),

        _ => icon!("../../assets/icons/help-circle.svg", color),
    }
}
