use std::cell::Ref;

use gtk::{glib, pango, subclass::prelude::*};

use crate::SCALE;

mod imp;

glib::wrapper! {
    /// Font for gnvim. Combines neovim's font settings (i.e. guifont and
    /// linespace) with pango font description & font metrics.
    pub struct Font(ObjectSubclass<imp::Font>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Font {
    /// Creates new font.
    ///
    /// # Arguments
    ///
    /// * `guifont` - The neovim guifont value. Should be something that
    ///               `pango::FontDescription::from_value` knows.
    /// * `linespace` - The neovim linespace value.
    pub fn new(guifont: &str, linespace: f32) -> Self {
        glib::Object::new(&[("guifont", &guifont), ("linespace", &linespace)])
            .expect("Failed to create Font")
    }

    /// Pango font description for this font.
    pub fn font_desc(&self) -> Ref<pango::FontDescription> {
        self.imp().font_desc.borrow()
    }

    /// Neovim guifont. This is what was used to construct the
    /// pango font description.
    pub fn guifont(&self) -> Ref<String> {
        self.imp().guifont.borrow()
    }

    /// Baseline in pango units.
    pub fn baseline(&self) -> f32 {
        self.height() - self.descent() - self.linespace() / 2.0
    }

    /// Linespace in pango units.
    pub fn linespace(&self) -> f32 {
        self.imp().linespace.get()
    }

    /// Ascent in pango units.
    pub fn ascent(&self) -> f32 {
        self.imp().ascent.get()
    }

    /// descent in pango units.
    pub fn descent(&self) -> f32 {
        self.imp().descent.get()
    }

    /// Strikethrough position in pango units.
    pub fn strikethrough_position(&self) -> f32 {
        self.imp().strikethrough_position.get()
    }

    /// Strikethrough thickness in pango units.
    pub fn strikethrough_thickness(&self) -> f32 {
        self.imp().strikethrough_thickness.get()
    }

    /// Underline poisition in pango units.
    pub fn underline_position(&self) -> f32 {
        self.imp().underline_position.get()
    }

    /// Underline thickness in pango units.
    pub fn underline_thickness(&self) -> f32 {
        self.imp().underline_thickness.get()
    }

    /// Font height in pango units.
    pub fn height(&self) -> f32 {
        self.imp().height.get()
    }

    /// Approximate character width in pango units.
    pub fn char_width(&self) -> f32 {
        self.imp().char_width.get()
    }

    /// Calculates grid size for given allocation.
    ///
    /// Returns (cols, rows).
    pub fn grid_size_for_allocation(&self, alloc: &gtk::Allocation) -> (usize, usize) {
        let rows = (alloc.height() as f32 / (self.height() / SCALE)).floor();
        let cols = (alloc.width() as f32 / (self.char_width() / SCALE)).floor();

        (cols as usize, rows as usize)
    }

    /// Scales x coordinate to column. Useful for scaling cursor on the screen
    /// to column on the grid.
    pub fn scale_to_col(&self, x: f64) -> usize {
        (x / (self.char_width() / SCALE) as f64).floor() as usize
    }

    /// Scales y coordinate to row. Useful for scaling cursor on the screen to
    /// column on the grid.
    pub fn scale_to_row(&self, y: f64) -> usize {
        (y / (self.height() / SCALE) as f64).floor() as usize
    }

    /// Calculates column's x coordinate.
    pub fn col_to_x(&self, col: f64) -> f64 {
        col * (self.char_width() / SCALE) as f64
    }

    /// Calculates row's y coordinate.
    pub fn row_to_y(&self, row: f64) -> f64 {
        row * (self.height() / SCALE) as f64
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new("Monospace 12", 0.0)
    }
}
