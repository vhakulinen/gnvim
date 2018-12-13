use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;

use nvim_bridge;
use ui::ui::HlDefs;

const MAX_HEIGHT: i32 = 500;

#[derive(Default)]
struct State {
    /// Currently selected row in wildmenu.
    selected: i32,
}

pub struct Wildmenu {
    css_provider: gtk::CssProvider,
    frame: gtk::Frame,
    list: gtk::ListBox,

    state: Rc<RefCell<State>>,
}

impl Wildmenu {
    pub fn new(nvim: Arc<Mutex<Neovim>>) -> Self {
        let css_provider = gtk::CssProvider::new();

        let frame = gtk::Frame::new(None);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);

        let scrolledwindow = gtk::ScrolledWindow::new(None, None);
        scrolledwindow
            .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolledwindow.add(&list);

        frame.add(&scrolledwindow);

        let frame_ref = frame.clone();
        // Make sure our container grows to certain height.
        list.connect_size_allocate(move |list, _| {
            // Calculate height based on shown rows.
            let count = list.get_children().len() as i32;
            let row_height = if let Some(item) = list.get_children().get(0) {
                item.get_preferred_height().0
            } else {
                16
            };

            let h = (row_height * count).min(MAX_HEIGHT);

            frame_ref.set_size_request(-1, h);
        });

        let state = Rc::new(RefCell::new(State::default()));

        let state_ref = state.clone();
        // If user selects some row with a mouse, notify nvim about it.
        list.connect_row_activated(move |_, row| {
            let prev = state_ref.borrow().selected;
            let new = row.get_index();

            let op = if new > prev { "<Tab>" } else { "<S-Tab>" };

            let mut nvim = nvim.lock().unwrap();
            for _ in 0..(new - prev).abs() {
                // NOTE(ville): nvim doesn't like single input with many
                //              tabs in it, so we'll have to send each
                //              individually.
                nvim.input(&op).unwrap();
            }
        });

        add_css_provider!(&css_provider, list, frame);

        Wildmenu {
            css_provider,
            list,
            frame,

            state,
        }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.frame.clone().upcast()
    }

    pub fn show(&self) {
        self.frame.show_all();
    }

    pub fn hide(&self) {
        self.frame.hide();
    }

    pub fn clear(&mut self) {
        let mut children = self.list.get_children();
        while let Some(item) = children.pop() {
            item.destroy();
        }
    }

    pub fn set_items(&mut self, items: &Vec<String>) {
        self.clear();

        for item in items {
            let label = gtk::Label::new(item.as_str());
            label.set_halign(gtk::Align::Start);

            let row = gtk::ListBoxRow::new();
            row.add(&label);

            add_css_provider!(&self.css_provider, row, label);

            self.list.add(&row);
        }

        self.list.show_all();
    }

    pub fn select(&mut self, item_num: i32) {
        self.state.borrow_mut().selected = item_num;

        if item_num < 0 {
            self.list.unselect_all();
        } else {
            if let Some(row) = self.list.get_row_at_index(item_num) {
                self.list.select_row(&row);
                row.grab_focus();
            }
        }
    }

    pub fn set_colors(&self, colors: &nvim_bridge::WildmenuColors, hl_defs: &HlDefs) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors, hl_defs);
        } else {
            self.set_colors_post20(colors, hl_defs);
        }
    }

    fn set_colors_pre20(&self, colors: &nvim_bridge::WildmenuColors, hl_defs: &HlDefs) {
        let css = format!(
            "GtkFrame {{
                border: none;
            }}

            GtkListBoxRow {{
                padding: 6px;
                color: #{fg};
                background-color: #{bg};
                outline: none;
            }}

            GtkListBoxRow:selected, GtkListBoxRow:selected > GtkLabel {{
                color: #{sel_fg};
                background: #{sel_bg};
            }}",
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex(),
            sel_fg = colors.sel_fg.unwrap_or(hl_defs.default_fg).to_hex(),
            sel_bg = colors.sel_bg.unwrap_or(hl_defs.default_bg).to_hex(),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    fn set_colors_post20(&self, colors: &nvim_bridge::WildmenuColors, hl_defs: &HlDefs) {
        let css = format!(
            "frame > border {{
                border: none;
            }}

            row {{
                padding: 6px;
                color: #{fg};
                background-color: #{bg};
                outline: none;
            }}

            row:selected, row:selected > label {{
                color: #{sel_fg};
                background: #{sel_bg};
            }}",
            fg = colors.fg.unwrap_or(hl_defs.default_fg).to_hex(),
            bg = colors.bg.unwrap_or(hl_defs.default_bg).to_hex(),
            sel_fg = colors.sel_fg.unwrap_or(hl_defs.default_fg).to_hex(),
            sel_bg = colors.sel_bg.unwrap_or(hl_defs.default_bg).to_hex(),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }
}
