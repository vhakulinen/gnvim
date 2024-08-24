use glib::subclass::prelude::*;
use gtk::{gio, prelude::*};

use super::PopupmenuObject;

mod imp;

glib::wrapper! {
    /// `Model` implements `gio::ListModel` and `gtk::SelectionModel`. Used in
    /// the popupmenu, this model sets its items lazily to work around issues
    /// with `gtk::ListView`: when replacing _all_ if the model contents, the
    /// listview discards all the existing widgets and recreates them. The time
    /// to recreate all the required items (e.g. all the 200 of them before
    /// recycling kicks in) can take quite a lot of time (e.g. tens of
    /// milliseconds).
    pub struct Model(ObjectSubclass<imp::Model>)
        @implements gio::ListModel, gtk::SelectionModel;
}

impl Model {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn lazy_add(&self, removed: u32) {
        let imp = self.imp();

        let index = self.n_items();
        let mut to_add = imp.to_add.borrow_mut();

        let n = 40.min(to_add.len());
        let new = to_add.drain(..n);
        let added = new.len() as u32;

        imp.items.borrow_mut().extend(new);

        self.items_changed(index, removed, added);
        self.do_selection_changed(imp.selected_item.get());
    }

    fn do_selection_changed(&self, item: Option<u32>) {
        if let Some(i) = item {
            if i <= self.n_items() {
                self.selection_changed(i, 1);
            }
        }
    }

    pub fn set_items(&self, items: Vec<PopupmenuObject>) {
        let imp = self.imp();

        let mut lazy = imp.lazy.borrow_mut();
        if let Some(old) = lazy.take() {
            old.remove();
        }

        let removed = self.n_items();
        imp.items.replace(vec![]);
        imp.to_add.replace(items);
        self.lazy_add(removed);

        *lazy = Some(glib::idle_add_local(glib::clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or_else]
            || glib::ControlFlow::Break,
            move || {
                this.lazy_add(0);

                let imp = this.imp();
                if imp.to_add.borrow().len() > 0 {
                    glib::ControlFlow::Continue
                } else {
                    imp.lazy.take();
                    glib::ControlFlow::Break
                }
            }
        )));
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
