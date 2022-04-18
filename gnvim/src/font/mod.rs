use std::cell::Ref;

use gtk::{glib, pango, subclass::prelude::*};

mod imp;

glib::wrapper! {
    pub struct Font(ObjectSubclass<imp::Font>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Font {
    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Font")
    }

    pub fn font_desc(&self) -> Ref<pango::FontDescription> {
        self.imp().font_desc.borrow()
    }

    pub fn ascent(&self) -> f32 {
        self.imp().ascent.get()
    }

    pub fn height(&self) -> f32 {
        self.imp().height.get()
    }

    pub fn char_width(&self) -> f32 {
        self.imp().char_width.get()
    }

    pub fn grid_size_for_allocation(&self, alloc: &gtk::Allocation) -> (usize, usize) {
        let rows = (alloc.height() as f32 / self.height()).floor() as usize;
        let cols = (alloc.width() as f32 / self.char_width()).floor() as usize;

        (cols, rows)
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}
