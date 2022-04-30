use std::cell::{Cell, RefCell};

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

const DEFAULT_HEIGHT: f32 = 16.0;
const DEFAULT_WIDTH: f32 = 8.0;

#[derive(Default)]
pub struct Font {
    pub font_desc: RefCell<pango::FontDescription>,

    pub linespace: Cell<f32>,
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

impl Font {
    pub fn update_metrics(&self, ctx: pango::Context) {
        let font_metrics = ctx
            .metrics(Some(&self.font_desc.borrow()), None)
            .expect("can't get font metrics");

        let extra = self.linespace.get() / 2.0;

        let scale = pango::SCALE as f32;
        self.ascent
            .set(font_metrics.ascent() as f32 / scale + extra);
        self.descent
            .set(font_metrics.descent() as f32 / scale + extra);

        let height = font_metrics.height() as f32;
        self.height.set(
            if height != 0.0 {
                height / scale
            } else {
                DEFAULT_HEIGHT
            } + self.linespace.get(),
        );

        let char_width = font_metrics.approximate_char_width() as f32;
        self.char_width.set(if char_width != 0.0 {
            char_width / scale
        } else {
            DEFAULT_WIDTH
        })
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
