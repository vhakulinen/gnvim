use std::cell::{Cell, RefCell};

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

#[derive(Default)]
pub struct Font {
    pub font_desc: RefCell<pango::FontDescription>,

    pub height: Cell<f32>,
    pub char_width: Cell<f32>,
    pub ascent: Cell<f32>,
    pub descent: Cell<f32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Font {
    const NAME: &'static str = "Font";
    type Type = super::Font;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Font {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.font_desc
            .replace(pango::FontDescription::from_string("Monospace 12"));

        let ctx = obj.pango_context();
        let font_metrics = ctx
            .metrics(Some(&self.font_desc.borrow()), None)
            .expect("can't get font metrics");

        let linespace = 4_f32; // TODO(ville): Get from options.
        let extra = linespace / 2.0;

        let scale = pango::SCALE as f32;
        self.ascent
            .set(font_metrics.ascent() as f32 / scale + extra);
        self.descent
            .set(font_metrics.descent() as f32 / scale + extra);
        self.height.set(self.ascent.get() + self.descent.get());
        self.char_width
            .set(font_metrics.approximate_char_width() as f32 / scale)
    }
}

impl WidgetImpl for Font {}
