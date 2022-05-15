use std::cell::{Cell, RefCell};

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

use crate::SCALE;

const DEFAULT_HEIGHT: f32 = 16.0 * SCALE;
const DEFAULT_WIDTH: f32 = 8.0 * SCALE;

#[derive(Default)]
pub struct Font {
    pub font_desc: RefCell<pango::FontDescription>,

    pub linespace: Cell<f32>,
    pub height: Cell<f32>,
    pub char_width: Cell<f32>,
    pub ascent: Cell<f32>,
    pub descent: Cell<f32>,
    pub underline_position: Cell<f32>,
    pub underline_thickness: Cell<f32>,
    pub strikethrough_position: Cell<f32>,
    pub strikethrough_thickness: Cell<f32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Font {
    const NAME: &'static str = "Font";
    type Type = super::Font;
    type ParentType = gtk::Widget;
}

impl Font {
    pub fn update_metrics(&self, ctx: pango::Context) {
        let font_metrics = ctx
            .metrics(Some(&self.font_desc.borrow()), None)
            .expect("can't get font metrics");

        self.ascent.set(font_metrics.ascent() as f32);
        self.descent.set(font_metrics.descent() as f32);

        self.underline_position
            .set(font_metrics.underline_position() as f32);
        self.strikethrough_position
            .set(font_metrics.strikethrough_position() as f32);

        // NOTE(ville): Set min size for the thickness. This is to workaround
        // some situations where the reported size for the thickness ends up
        // being less than a singe pixel on the sceen (and thus not rendered).
        self.underline_thickness
            .set((font_metrics.underline_thickness() as f32).max(SCALE));
        self.strikethrough_thickness
            .set((font_metrics.strikethrough_thickness() as f32).max(SCALE));

        let height = font_metrics.height() as f32;
        self.height.set(
            if height != 0.0 {
                height
            } else {
                DEFAULT_HEIGHT
            } + self.linespace.get(),
        );

        let char_width = font_metrics.approximate_char_width() as f32;
        self.char_width.set(if char_width != 0.0 {
            char_width
        } else {
            DEFAULT_WIDTH
        });
    }
}

impl ObjectImpl for Font {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.font_desc
            .replace(pango::FontDescription::from_string("Monospace 12"));

        let ctx = obj.pango_context();
        self.update_metrics(ctx);
    }
}

impl WidgetImpl for Font {}
