use gtk::{glib, prelude::*};

mod imp;

glib::wrapper! {
    pub struct LayoutManager(ObjectSubclass<imp::LayoutManager>)
        @extends gtk::LayoutManager;
}

impl LayoutManager {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("failed to create FixedzLayoutManager")
    }

    pub fn layout_child(&self, child: &impl IsA<gtk::Widget>) -> super::Child {
        gtk::LayoutManager::layout_child(self.upcast_ref(), child)
            .downcast()
            .expect("invalid layout child")
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}
