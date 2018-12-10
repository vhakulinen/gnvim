use std::sync::{Arc, Mutex};

use gdk;
use gtk;
use gtk::prelude::*;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;
use pango;

use nvim_bridge::{CompletionItem, PmenuColors};
use thread_guard::ThreadGuard;
use ui::font::{Font, FontUnit};

/// Maximum height of completion menu.
const MAX_HEIGHT: i32 = 500;
/// Fixed width of completion menu.
const FIXED_WIDTH: i32 = 800;

/// Wraps completion item into a structure which contains the item and some
/// of the widgets to display it.
struct CompletionItemWidgetWrap {
    /// Actual completion item.
    item: CompletionItem,
    /// Widget displaying the (shortened) info from `item`. This is only
    /// shown when this completion item is selected.
    info: gtk::Label,
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
        info_label.set_line_wrap(true);
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
        list.set_selection_mode(gtk::SelectionMode::Single);

        let scrolled_list = gtk::ScrolledWindow::new(None, None);
        scrolled_list.add(&list);
        scrolled_list
            .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);
        box_.pack_start(&scrolled_list, true, true, 0);
        box_.pack_start(&scrolled_info, true, true, 0);

        add_css_provider!(
            &css_provider,
            info_label,
            scrolled_info,
            list,
            scrolled_list,
            box_
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
        let box_ref = box_.clone();
        let layout_ref = layout.clone();
        // Adjust our size and position when the list box's content changes.
        list.connect_size_allocate(move |list, alloc| {
            let state = state_ref.borrow();
            let box_ = box_ref.clone();
            let layout = layout_ref.clone();

            // Calculate height based on shown rows.
            let count = list.get_children().len() as i32;
            // Get the first non-selected row. Non-selected so that we don't
            // take into account the height of `info` for each row.
            let index = if state.selected == 0 { 1 } else { 0 };
            let row_height = if let Some(item) = state.items.get(index) {
                item.row.get_preferred_height().0
            } else {
                16
            };
            // If some of the displayed rows is selected, we need to add
            // that extra height for the `info` row.
            let extra = if state.selected == -1 { 0 } else { 1 };

            let mut h = (row_height * (count + extra)).min(MAX_HEIGHT);

            if let Some(available_size) = state.available_size {
                // Check if we need to adjust our position, x-axis wise.
                let x2 = state.anchor.x + alloc.width;
                if x2 > available_size.width {
                    // Magic number 5 here is making sure there is a small cap
                    // between the popupmenu and the window border.
                    let x_offset = x2 - available_size.width + 5;
                    let new_x = (state.anchor.x - x_offset).max(0);

                    // TODO(ville): Do we want to truncate the width of the popupmenu
                    //              in case when new_x == 0 && w > state.available_size.width?

                    layout.move_(
                        &box_,
                        new_x,
                        state.anchor.y + state.anchor.height,
                    );
                }

                // Check if we need to adjust our height.
                // TODO(ville): Move the popupmenu upwards from the anchor position
                //              of there is no room downwards.
                let y2 = state.anchor.y + h;
                if y2 > available_size.height {
                    h = available_size.height
                        - state.anchor.y
                        - state.anchor.height
                        - 10;
                }
            }

            // We'll have to wait for the next UI loop before setting the
            // desired height of the container.
            gtk::idle_add(move || {
                box_.set_size_request(FIXED_WIDTH, h);

                // NOTE(ville): Seems like there is no other way to a widget
                //              to resize it self.
                box_.hide();
                box_.show();
                Continue(false)
            });
        });

        let state_ref = state.clone();
        let list_ref = list.clone();
        let box_ref = box_.clone();
        info_box.connect_size_allocate(move |_, alloc| {
            let state = state_ref.borrow();
            let a = list_ref.get_allocation();

            // When `info_box` is shown, make sure that we can show as much
            // of its content - to the point its height reaches MAX_HEIGHT.
            let mut h = alloc.height.max(a.height).min(MAX_HEIGHT);

            if let Some(available_size) = state.available_size {
                // Check if we need to adjust our height.
                // TODO(ville): See comment from list's connect_size:allocate
                let y2 = state.anchor.y + h;
                if y2 > available_size.height {
                    h = available_size.height
                        - state.anchor.y
                        - state.anchor.height
                        - 10;
                }
            }

            let box_ref = box_ref.clone();
            gtk::idle_add(move || {
                box_ref.set_size_request(FIXED_WIDTH, h);

                box_ref.hide();
                box_ref.show();
                Continue(false)
            });
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
        }
    }

    pub fn toggle_show_info(&mut self) {
        let state = self.state.borrow_mut();

        if state.selected == -1 {
            return;
        }

        self.info_shown = !self.info_shown;

        if let Some(item) = state.items.get(state.selected as usize) {
            if !self.info_shown {
                let adj = self.scrolled_info.get_vadjustment().unwrap();
                adj.set_value(0.0);
                // TODO(ville): There is a bug in GTK+ and some adjustment animations,
                //              where the adjustment's value is set back to upper - page-size
                //              if the user has "overshot" the scrolling. Work around this.
            }

            self.info_label.set_text(&item.item.info);
            self.scrolled_list.set_visible(!self.info_shown);
            self.scrolled_info.set_visible(self.info_shown);
        }
    }

    /// Hides the popupmenu.
    pub fn hide(&mut self) {
        if self.info_shown {
            self.toggle_show_info();
        }

        self.layout.hide();
    }

    /// Shows the popupmenu.
    pub fn show(&self) {
        self.layout.show();
    }

    /// Sets the anchor point for popupmenu.
    pub fn set_anchor(&self, rect: gdk::Rectangle) {
        let mut state = self.state.borrow_mut();
        self.layout.move_(&self.box_, rect.x, rect.y + rect.height);
        state.anchor = rect;
    }

    pub fn set_items(&mut self, items: Vec<CompletionItem>) {
        if self.info_shown {
            self.toggle_show_info();
        }

        let mut state = self.state.borrow_mut();
        state.selected = -1;

        while let Some(item) = state.items.pop() {
            item.row.destroy();
        }

        for item in items.into_iter() {
            let wrap = create_completionitem_widget(item, &self.css_provider);
            self.list.add(&wrap.row);
            state.items.push(wrap);
        }
        self.list.show_all();
    }

    pub fn select(&mut self, item_num: i32) {
        if self.info_shown {
            self.toggle_show_info();
        }
        let mut state = self.state.borrow_mut();

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(false);
            }
        }

        let prev = state.selected;
        state.selected = item_num;

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(true);
                self.list.select_row(&item.row);
                item.row.grab_focus();

                // If we went from no selection to state where the last item
                // is selected, we'll have to do some extra work to make sure
                // that the whole item is visible.
                let max = state.items.len() as i32 - 1;
                let adj = self.scrolled_list.get_vadjustment().unwrap();
                if prev == -1 && state.selected == max {
                    adj.set_value(adj.get_upper());
                }
            }
        } else {
            self.list.unselect_all();

            // If selecteion is removed, move the srolled window to the top.
            let adj = self.scrolled_list.get_vadjustment().unwrap();
            gtk::idle_add(move || {
                adj.set_value(0.0);
                Continue(false)
            });
        }
    }

    pub fn set_colors(&mut self, colors: PmenuColors) {
        self.colors = colors;
        self.set_styles();
    }

    fn set_styles(&self) {
        if gtk::get_minor_version() < 20 {
            self.set_styles_pre20();
        } else {
            self.set_styles_post20();
        }
    }

    fn set_styles_post20(&self) {
        let css = format!(
            "{font_wild}

            box, grid, list, row, label {{
                color: #{normal_fg};
                background-color: #{normal_bg};
                outline: none;
            }}

            #info-label {{
                padding: 10px;
            }}

            row:selected, row:selected > grid, row:selected > grid > label {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}

            box {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Point),
            normal_fg = self.colors.fg.to_hex(),
            normal_bg = self.colors.bg.to_hex(),
            selected_bg = self.colors.sel_bg.to_hex(),
            selected_fg = self.colors.sel_fg.to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_styles_pre20(&self) {
        let css = format!(
            "{font_wild}

            GtkBox, GtkGrid, GtkListBox, GtkListBoxRow, GtkLabel {{
                color: #{normal_fg};
                background-color: #{normal_bg};
                outline: none;
            }}

            #info-label {{
                padding: 10px;
            }}

            GtkListBoxRow:selected,
            GtkListBoxRow:selected > GtkGrid,
            GtkListBoxRow:selected > GtkGrid > GtkLabel {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}

            GtkBox {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Pixel),
            normal_fg = self.colors.fg.to_hex(),
            normal_bg = self.colors.bg.to_hex(),
            selected_bg = self.colors.sel_bg.to_hex(),
            selected_fg = self.colors.sel_fg.to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font;
        self.set_styles();
    }
}

fn create_completionitem_widget(
    item: CompletionItem,
    css_provider: &gtk::CssProvider,
) -> CompletionItemWidgetWrap {
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);

    let kind = gtk::Label::new(item.kind.as_str());
    kind.set_halign(gtk::Align::Start);
    kind.set_margin_left(5);
    kind.set_margin_right(5);
    grid.attach(&kind, 0, 0, 1, 1);

    let word = gtk::Label::new(item.word.as_str());
    word.set_halign(gtk::Align::Start);
    word.set_ellipsize(pango::EllipsizeMode::End);
    grid.attach(&word, 1, 0, 1, 1);

    let menu = gtk::Label::new(item.menu.as_str());
    menu.set_halign(gtk::Align::End);
    menu.set_hexpand(true);
    menu.set_margin_left(5);
    menu.set_margin_right(5);
    grid.attach(&menu, 2, 0, 1, 1);

    let info = gtk::Label::new(shorten_info(&item.info).as_str());
    info.set_halign(gtk::Align::Start);
    info.set_ellipsize(pango::EllipsizeMode::End);
    <gtk::Widget as WidgetExt>::set_name(&info.clone().upcast(), "gnvim-info");

    // On initially shown, set the info label hidden. We'll show it when
    // the row it belongs to is selected (otherwise its always hidden).
    info.connect_realize(|info| {
        info.hide();
    });

    grid.attach(&info, 1, 1, 2, 1);

    // NOTE(ville): We only need to explicitly create this row widget
    //              so we can set css provider to it.
    let row = gtk::ListBoxRow::new();
    row.add(&grid);

    add_css_provider!(css_provider, grid, kind, word, menu, info, row);

    CompletionItemWidgetWrap { item, info, row }
}

/// Returns first line of `info`.
fn shorten_info(info: &String) -> String {
    let lines = info.split("\n").collect::<Vec<&str>>();
    let first_line = lines.get(0).unwrap();
    first_line.to_string()
}
