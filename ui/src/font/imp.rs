use std::cell::{Cell, RefCell};

use gtk::{glib, pango, prelude::*, subclass::prelude::*};

use crate::SCALE;

const DEFAULT_HEIGHT: f32 = 16.0 * SCALE;
const DEFAULT_WIDTH: f32 = 8.0 * SCALE;

#[derive(Default)]
pub struct Font {
    pub guifont: RefCell<String>,
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
        let font_metrics = ctx.metrics(Some(&self.font_desc.borrow()), None);

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
    fn constructed(&self) {
        self.parent_constructed();

        let ctx = self.obj().pango_context();
        self.update_metrics(ctx);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecString::builder("guifont")
                    .default_value(Some("Monospace 12"))
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY)
                    .build(),
                glib::ParamSpecFloat::builder("linespace")
                    .minimum(0.0)
                    .default_value(0.0)
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "guifont" => self.guifont.borrow().to_value(),
            "linespace" => self.linespace.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "guifont" => {
                let font_str = value
                    .get::<&str>()
                    .expect("property guifont needs to be &str");

                let mut font_desc = pango::FontDescription::from_string(font_str);
                if font_desc.size() == 0 {
                    // TODO(ville): Should probably notify the user here.
                    font_desc.set_size(12 * SCALE as i32);
                }

                self.guifont.replace(font_str.to_string());
                self.font_desc.replace(font_desc);
            }
            "linespace" => {
                self.linespace.set(
                    value
                        .get::<f32>()
                        .expect("property linepsace needs to be f32")
                        * SCALE,
                );
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for Font {}
