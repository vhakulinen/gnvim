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

    pub fn to_css(&self) -> String {
        let desc = self.font_desc();

        let family = desc
            .family()
            .map(|family| format!("font-family: \"{}\";", family));
        let variant = format!(
            "font-variant: {};",
            match desc.variant() {
                pango::Variant::Normal => "normal",
                pango::Variant::SmallCaps => "small-caps",
                pango::Variant::AllSmallCaps => "all-small-caps",
                pango::Variant::PetiteCaps => "petite-caps",
                pango::Variant::AllPetiteCaps => "all-petite-caps",
                pango::Variant::Unicase => "unicase",
                pango::Variant::TitleCaps => "titling-caps",
                _ => "none",
            }
        );
        let style = format!(
            "font-style: {};",
            match desc.style() {
                pango::Style::Normal => "normal",
                pango::Style::Oblique => "oblique",
                pango::Style::Italic => "italic",
                _ => "none",
            }
        );

        let weight = format!(
            "font-weight: {};",
            match desc.weight() {
                pango::Weight::Thin => 100,
                pango::Weight::Ultralight => 200,
                pango::Weight::Light => 300,
                pango::Weight::Semilight => 350,
                pango::Weight::Book => 380,
                pango::Weight::Normal => 400,
                pango::Weight::Medium => 500,
                pango::Weight::Semibold => 600,
                pango::Weight::Bold => 700,
                pango::Weight::Ultrabold => 800,
                pango::Weight::Heavy => 900,
                pango::Weight::Ultraheavy => 1000,
                _ => 400,
            }
        );

        let size = format!("font-size: {}pt;", desc.size() as f32 / SCALE);

        let stretch = format!(
            "font-stretch: {};",
            match desc.stretch() {
                pango::Stretch::UltraCondensed => "ultra-condensed",
                pango::Stretch::ExtraCondensed => "extra-condensed",
                pango::Stretch::Condensed => "condensed",
                pango::Stretch::SemiCondensed => "semi-condensed",
                pango::Stretch::Normal => "normal",
                pango::Stretch::SemiExpanded => "semi-expanded",
                pango::Stretch::Expanded => "expanded",
                pango::Stretch::ExtraExpanded => "extra-expanded",
                pango::Stretch::UltraExpanded => "ultra-expanded",
                _ => "none",
            }
        );

        let variations = desc
            .variations()
            .map(|variations| format!("font-variation-settings: {};", variations));

        let f = format!(
            "
            {family}
            {variant}
            {style}
            {weight}
            {size}
            {stretch}
            {variations}
            ",
            family = family.unwrap_or(String::new()),
            variations = variations.unwrap_or(String::new()),
        );

        f
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new("Monospace 12", 0.0)
    }
}
