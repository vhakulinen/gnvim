use std::sync::{Arc, Mutex};

use gdk;
use glib;
use gtk;
use gtk::prelude::*;
use pango;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;

use ui::color::Color;
use nvim_bridge::CompletionItem;
use thread_guard::ThreadGuard;

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
}

impl Default for State {
    fn default() -> Self {
        State {
            selected: -1,
            items: vec!(),
        }
    }
}

pub struct Popupmenu {
    /// Box that contains all the "content". This box is placed in side the
    /// layout container.
    box_: gtk::Box,
    /// Top level container. Box is added into this and moved around as needed.
    /// This container should be added to any grid where the popupmenu is needed.
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
    nvim: Arc<Mutex<Neovim>>,
}

impl Popupmenu {
    /// Creates new popupmenu. After creating a new popupmenu, remember to add
    /// it to some container (get the widget by calling `widget()`).
    ///
    /// * `nvim` - Neovim instance. Popupmenu will instruct neovim to act on
    ///            user interaction.
    pub fn new(nvim: Arc<Mutex<Neovim>>) -> Self {
        let css_provider = gtk::CssProvider::new();

        let info_label = gtk::Label::new("");
        info_label.set_halign(gtk::Align::Start);
        info_label.set_valign(gtk::Align::Start);
        info_label.set_margin_top(10);
        info_label.set_margin_bottom(10);
        info_label.set_margin_left(10);
        info_label.set_margin_right(10);
        info_label.set_line_wrap(true);
        info_label.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Because we're setting valign and halign to the info label, we'll
        // need to have some container in between the label and scrolled window.
        // Other wise there'll be some black boxes when scroll bars are needed.
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        info_box.add(&info_label);

        let scrolled_info = gtk::ScrolledWindow::new(None, None);
        scrolled_info.add(&info_box);
        scrolled_info.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled_info.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let scrolled_list = gtk::ScrolledWindow::new(None, None);
        scrolled_list.add(&list);
        scrolled_list.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled_list.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);
        box_.pack_start(&scrolled_list, true, true, 0);
        box_.pack_start(&scrolled_info, true, true, 0);

        let state = Arc::new(ThreadGuard::new(State::default()));

        let state_ref = state.clone();
        let box_ref = box_.clone();
        // Adjust our height when the list box' content changes.
        list.connect_size_allocate(move |list, alloc| {
            let state = state_ref.borrow();

            let max_h = 500;

            // Calculate height based on shown rows.
            let count = list.get_children().len() as i32;
            // Get the first non-selected row. Non-selected so that we don't
            // take account of the height if `info` for each row.
            let index = if state.selected == 0 { 1 } else { 0 };
            let row_height = if let Some(item) = state.items.get(index) {
                item.row.get_preferred_height().0
            } else {
                16
            };
            // If some of the displayed rows is selected, we need to add
            // that extra height for the `info` row.
            let extra = if state.selected == -1 { 0 } else { 1 };

            let h = (row_height * (count + extra)).min(max_h);
            let w = alloc.width;

            let box_ = box_ref.clone();
            // We'll have to wait for the next UI loop before setting the
            // desired height of the container.
            gtk::idle_add(move || {
                box_.set_size_request(w, h);

                // NOTE(ville): Seems like there is no other way to a widget
                //              to resize it self.
                box_.hide();
                box_.show();
                Continue(false)
            });
        });

        let state_ref = state.clone();
        let nvim_ref = nvim.clone();
        // When a row is activated (by mouse click), notify neovim to change
        // the selection to the activated row.
        list.connect_row_activated(move |list, row| {
            let mut state = state_ref.borrow_mut();
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
        list.connect_button_press_event(move |list, e| {
            // ...check if the button press is double click.
            if e.get_event_type() == gdk::EventType::DoubleButtonPress {
                // And if so, tell neovim to select the current completion item.
                let mut nvim = nvim_ref.lock().unwrap();
                nvim.input("<C-y>").unwrap();
            }

            Inhibit(false)
        });

        let layout = gtk::Layout::new(None, None);
        layout.put(&box_, 0, 0);
        layout.show_all();
        scrolled_info.hide();

        Popupmenu {
            box_,
            layout,
            css_provider,
            list,
            scrolled_list,
            scrolled_info,
            info_label,
            state,
            nvim,
            info_shown: false,
        }
    }

    pub fn toggle_show_info(&mut self) {
        let mut state = self.state.borrow_mut();

        if state.selected == -1 {
            return
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

    /// Hides the popupmenu and removes it from any parent it might exists in.
    pub fn hide(&mut self) {
        if self.info_shown {
            self.toggle_show_info();
        }

        if let Some(parent) = self.layout.get_parent() {
            if let Ok(container) = parent.downcast::<gtk::Container>() {
                container.remove(&self.layout);
            }
        }
    }

    /// Returns top level widget (container) for popupmenu. This widget should
    /// be added to a container (of a grid where the menu is needed).
    pub fn widget(&self) -> gtk::Widget {
        self.layout.clone().upcast()
    }

    /// Sets the position of popupmenu, relative to the parent its in.
    pub fn set_position(&self, x: i32, y: i32) {
        self.layout.move_(&self.box_, x, y);
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

    pub fn set_colors(&self,
                      normal_fg: Color,
                      normal_bg: Color,
                      selected_fg: Color,
                      selected_bg: Color) {
        let css = format!(
            "scrolledwindow, layout, grid, label, list, row {{
                border-color: #{normal_fg};
                color: #{normal_fg};
                background-color: #{normal_bg};
                outline: none;
            }}

            row:selected, row:selected > grid, row:selected > grid > label {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}

            scrolledwindow {{
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
            }}
            ", normal_fg=normal_fg.to_hex(),
               normal_bg=normal_bg.to_hex(),
               selected_bg=selected_bg.to_hex(),
               selected_fg=selected_fg.to_hex());
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }


    pub fn set_font(&self, font: &pango::FontDescription) {
        gtk::WidgetExt::override_font(&self.widget(), font);
    }
}

fn create_completionitem_widget(item: CompletionItem, css_provider: &gtk::CssProvider) -> CompletionItemWidgetWrap {
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let kind = gtk::Label::new(item.kind.as_str());
    kind.set_halign(gtk::Align::Start);
    kind.set_margin_left(5);
    kind.set_margin_right(5);
    kind.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    grid.attach(&kind, 0, 0, 1, 1);

    let word = gtk::Label::new(item.word.as_str());
    word.set_halign(gtk::Align::Start);
    word.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    grid.attach(&word, 1, 0, 1, 1);

    let menu = gtk::Label::new(item.menu.as_str());
    menu.set_halign(gtk::Align::End);
    menu.set_hexpand(true);
    menu.set_margin_left(5);
    menu.set_margin_right(5);
    menu.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    grid.attach(&menu, 2, 0, 1, 1);

    let info = gtk::Label::new(shorten_info(&item.info).as_str());
    info.set_halign(gtk::Align::Start);
    info.set_ellipsize(pango::EllipsizeMode::End);
    info.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    <gtk::Widget as WidgetExt>::set_name(&info.clone().upcast(), "gnvim-info");

    // On initially shown, set the info label hidden. We'll show it when
    // the row it belongs to is selected (otherwise its always hidden).
    info.connect_realize(|info| {
        info.hide();
    });

    grid.attach(&info, 1, 1, 1, 1);

    // NOTE(ville): We only need to explicitly a crate this row widget
    //              so we can set css provider to it.
    let row = gtk::ListBoxRow::new();
    row.get_style_context()
        .unwrap()
        .add_provider(css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    row.add(&grid);

    CompletionItemWidgetWrap {
        item,
        info,
        row,
    }
}

fn shorten_info(info: &String) -> String {
    let lines = info.split("\n").collect::<Vec<&str>>();
    let first_line = lines.get(0).unwrap();
    let sentences = first_line.split(".").collect::<Vec<&str>>();
    let sentence = sentences.get(0).unwrap();
    sentence.to_string()
}
