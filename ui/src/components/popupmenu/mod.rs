use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp;
mod row;

use nvim::types::PopupmenuItem;
use row::Row;

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

    /// Proxy to get the internal listview's preferred size.
    pub fn listview_preferred_size(&self) -> (gtk::Requisition, gtk::Requisition) {
        self.imp().listview.preferred_size()
    }

    pub fn select(&self, n: u32) {
        let imp = self.imp();
        imp.selection_model.select_item(n, true);
        imp.listview
            .activate_action("list.scroll-to-item", Some(&n.to_variant()))
            .expect("failed to activate list.scroll-to-item action");
    }

    pub fn set_max_height(&self, h: i32) {
        self.imp().max_height.set(h);
    }

    pub fn set_max_width(&self, w: i32) {
        self.imp().max_width.set(w);
    }
}
