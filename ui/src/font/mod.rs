use std::cell::Ref;

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

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

    pub fn update_metrics(&self) {
        self.imp().update_metrics(self.pango_context());
    }

    pub fn font_desc(&self) -> Ref<pango::FontDescription> {
        self.imp().font_desc.borrow()
    }

    pub fn set_font_from_str(&self, font: &str) {
        self.imp()
            .font_desc
            .replace(pango::FontDescription::from_string(font));
    }

    pub fn set_linespace(&self, linespace: f32) {
        self.imp().linespace.set(linespace);
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
        let rows = (alloc.height() as f32 / self.height()).floor();
        let cols = (alloc.width() as f32 / self.char_width()).floor();

        (cols as usize, rows as usize)
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}
