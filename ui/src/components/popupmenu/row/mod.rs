use gtk::{glib, subclass::prelude::*};
use nvim::types::PopupmenuItem;

mod imp;

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Row {
    fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_item(&self, item: &PopupmenuItem) {
        let imp = self.imp();

        imp.word.set_label(&item.word);
        imp.kind.set_label(&item.kind);
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}
