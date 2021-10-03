use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::nvim_bridge;
use crate::nvim_gio::GioNeovim;
use crate::ui::color::{Color, HlDefs, HlGroup};
use crate::ui::common::spawn_local;

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
    pub fn new(nvim: GioNeovim) -> Self {
        let css_provider = gtk::CssProvider::new();

        let frame = gtk::Frame::new(None);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);

        let scrolledwindow = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        scrolledwindow
            .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolledwindow.add(&list);

        frame.add(&scrolledwindow);

        let frame_weak = frame.downgrade();
        // Make sure our container grows to certain height.
        list.connect_size_allocate(clone!(frame_weak => move |list, _| {
            let frame = upgrade_weak!(frame_weak);
            // Calculate height based on shown rows.
            let count = list.children().len() as i32;
            let row_height = if let Some(item) = list.children().get(0) {
                item.preferred_height().0
            } else {
                16
            };

            let h = (row_height * count).min(MAX_HEIGHT);

            frame.set_size_request(-1, h);
        }));

        let state = Rc::new(RefCell::new(State::default()));

        // If user selects some row with a mouse, notify nvim about it.
        list.connect_row_activated(clone!(state => move |_, row| {
            let prev = state.borrow().selected;
            let new = row.index();

            let op = if new > prev { "<Tab>" } else { "<S-Tab>" };

            for _ in 0..(new - prev).abs() {
                // NOTE(ville): nvim doesn't like single input with many
                //              tabs in it, so we'll have to send each
                //              individually.
                let nvim = nvim.clone();
                spawn_local(async move {
                    nvim.input(&op)
                        .await
                        .unwrap();
                })
            }
        }));

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
        let mut children = self.list.children();
        while let Some(item) = children.pop() {
            self.list.remove(&item);
        }
    }

    pub fn set_items(&mut self, items: &[nvim_bridge::CompletionItem]) {
        self.clear();

        for item in items {
            let label = gtk::Label::new(Some(item.word.as_str()));
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
        } else if let Some(row) = self.list.row_at_index(item_num) {
            self.list.select_row(Some(&row));
            row.grab_focus();
        }
    }

    pub fn set_colors(&self, hl_defs: &HlDefs) {
        let color = hl_defs.get_hl_group(&HlGroup::Wildmenu);
        let color_sel = hl_defs.get_hl_group(&HlGroup::WildmenuSel);
        let fg = color
            .and_then(|hl| hl.foreground)
            .unwrap_or(hl_defs.default_fg);
        let bg = color
            .and_then(|hl| hl.background)
            .unwrap_or(hl_defs.default_bg);
        let sel_fg = color_sel
            .and_then(|hl| hl.foreground)
            .unwrap_or(hl_defs.default_fg);
        let sel_bg = color_sel
            .and_then(|hl| hl.background)
            .unwrap_or(hl_defs.default_bg);

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
            fg = fg.to_hex(),
            bg = bg.to_hex(),
            sel_fg = sel_fg.to_hex(),
            sel_bg = sel_bg.to_hex(),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }
}
