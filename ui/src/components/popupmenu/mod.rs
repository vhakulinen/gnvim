use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp;
mod row;

use nvim::types::PopupmenuItem;
use row::Row;

use crate::{nvim::Neovim, spawn_local, SCALE};

glib::wrapper! {
    pub struct Popupmenu(ObjectSubclass<imp::Popupmenu>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Popupmenu {
    pub fn set_items(&self, items: Vec<PopupmenuItem>) {
        let imp = self.imp();

        let store = imp.store.borrow();
        store.remove_all();

        for item in items {
            store.append(&glib::BoxedAnyObject::new(item));
        }
    }

    pub fn get_padding_x(&self) -> f32 {
        self.imp().font_char_width.get()
    }

    /// Proxy to get the internal listview's preferred size.
    pub fn listview_preferred_size(&self) -> (gtk::Requisition, gtk::Requisition) {
        self.imp().listview.preferred_size()
    }

    pub fn select(&self, n: i64) {
        let imp = self.imp();

        if n < 0 {
            imp.selection_model.unselect_all();
        } else {
            let n = n as u32;
            imp.selection_model.select_item(n, true);
            imp.listview
                .activate_action("list.scroll-to-item", Some(&n.to_variant()))
                .expect("failed to activate list.scroll-to-item action");
        }
    }

    pub fn set_max_height(&self, h: i32) {
        self.imp().max_height.set(h);
    }

    pub fn set_max_width(&self, w: i32) {
        self.imp().max_width.set(w);
    }

    pub fn report_pum_bounds(&self, nvim: &Neovim, x: f32, y: f32) {
        let imp = self.imp();
        let font = imp.font.borrow();
        let (_, req) = self.preferred_size();
        let (w, h) = (req.width() as f32, req.height() as f32);

        let w = (w / (font.char_width() / SCALE)) as f64;
        let h = (h / (font.height() / SCALE)) as f64;
        let col = (x / (font.char_width() / SCALE)) as f64;
        let row = (y / (font.height() / SCALE)) as f64;

        spawn_local!(clone!(@weak nvim => async move {
            let res = nvim
                .client()
                .await
                .nvim_ui_pum_set_bounds(w, h, row, col)
                .await
                .unwrap();

            res.await.expect("nvim_ui_pum_set_bounds failed");
        }));
    }
}
