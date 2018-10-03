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
    /// Top level container. Scrolled window is added into this and moved
    /// around as needed. This container should be added to any grid where
    /// the popupmenu is needed.
    layout: gtk::Layout,
    /// Scrolled window that contains a list box.
    scrolled: gtk::ScrolledWindow,
    /// List box that contains all the completion items.
    list: gtk::ListBox,
    /// Style provider for all internal widgets.
    css_provider: gtk::CssProvider,

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

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let scrolled = gtk::ScrolledWindow::new(None, None);
        scrolled.add(&list);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.get_style_context()
            .unwrap()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let state = Arc::new(ThreadGuard::new(State::default()));

        let state_ref = state.clone();
        let scrolled_ref = scrolled.clone();
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
            // But, if some of the displayed rows is selected, we need to add
            // that extra height for the `info` row.
            let extra = if state.selected == -1 { 0 } else { 1 };

            let h = (row_height * (count + extra)).min(max_h);
            let w = alloc.width;

            let scrolled_ref = scrolled_ref.clone();
            // We'll have to wait for the next UI loop before setting the
            // desired height of the scrolled window.
            gtk::idle_add(move || {
                scrolled_ref.set_size_request(w, h);

                // NOTE(ville): Seems like there is no other way to a widget
                //              to resize it self.
                scrolled_ref.hide();
                scrolled_ref.show();
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
        layout.put(&scrolled, 0, 0);
        layout.show_all();

        Popupmenu {
            layout,
            css_provider,
            list,
            scrolled,
            state,
            nvim,
        }
    }

    /// Hides the popupmenu and removes it from any parent it might exists in.
    pub fn hide(&self) {
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
        self.layout.move_(&self.scrolled, x, y);
    }

    pub fn set_items(&self, items: Vec<CompletionItem>) {
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

    pub fn select(&self, item_num: i32) {
        let mut state = self.state.borrow_mut();

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(false);
            }
        }

        state.selected = item_num;

        if state.selected >= 0 {
            if let Some(item) = state.items.get(state.selected as usize) {
                item.info.set_visible(true);
                self.list.select_row(&item.row);
                item.row.grab_focus();
            }

            // TODO(ville): When selecting the last item after having no selection,
            //              the scrolled window wont show the `info` label for
            //              reasons. Fix this.
        }
    }

    pub fn set_colors(&self,
                      normal_fg: Color,
                      normal_bg: Color,
                      selected_fg: Color,
                      selected_bg: Color) {
        let css = format!(
            "grid, label, list, row {{
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
        gtk::WidgetExt::override_font(&self.list, font);
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

    let chars = sentence.chars();

    if chars.clone().count() > 80 {
        chars.into_iter().take(77).collect::<String>() + "..."
    } else {
        sentence.to_string()
    }
}
