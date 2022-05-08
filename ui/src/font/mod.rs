use std::cell::Ref;

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

use crate::SCALE;

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

    pub fn set_font_from_str(&self, font: &str) -> Result<(), &'static str> {
        let desc = pango::FontDescription::from_string(font);

        if desc.size() == 0 {
            return Err("font doesn't have size");
        }

        self.imp().font_desc.replace(desc);

        Ok(())
    }

    pub fn set_linespace(&self, linespace: f32) {
        self.imp().linespace.set(linespace * SCALE);
    }

    pub fn baseline(&self) -> f32 {
        //let imp = self.imp();
        self.height() - self.descent() - self.linespace() / 2.0
    }

    pub fn linespace(&self) -> f32 {
        self.imp().linespace.get()
    }

    pub fn ascent(&self) -> f32 {
        self.imp().ascent.get()
    }

    pub fn descent(&self) -> f32 {
        self.imp().descent.get()
    }

    pub fn strikethrough_position(&self) -> f32 {
        self.imp().strikethrough_position.get()
    }

    pub fn strikethrough_thickness(&self) -> f32 {
        self.imp().strikethrough_thickness.get()
    }

    pub fn underline_position(&self) -> f32 {
        self.imp().underline_position.get()
    }

    pub fn underline_thickness(&self) -> f32 {
        self.imp().underline_thickness.get()
    }

    pub fn height(&self) -> f32 {
        self.imp().height.get()
    }

    pub fn char_width(&self) -> f32 {
        self.imp().char_width.get()
    }

    pub fn grid_size_for_allocation(&self, alloc: &gtk::Allocation) -> (usize, usize) {
        let rows = (alloc.height() as f32 / (self.height() / SCALE)).floor();
        let cols = (alloc.width() as f32 / (self.char_width() / SCALE)).floor();

        (cols as usize, rows as usize)
    }

    pub fn scale_to_col(&self, x: f64) -> usize {
        (x / (self.char_width() / SCALE) as f64).floor() as usize
    }

    pub fn scale_to_row(&self, y: f64) -> usize {
        (y / (self.height() / SCALE) as f64).floor() as usize
    }

    pub fn col_to_x(&self, col: f64) -> f64 {
        col * (self.char_width() / SCALE) as f64
    }

    pub fn row_to_y(&self, row: f64) -> f64 {
        row * (self.height() / SCALE) as f64
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}
