use pango::FontDescription;
use pango;
use pangocairo;
use cairo;
use gtk::{DrawingArea};

use gtk::prelude::*;
use pango::prelude::*;

use ui::color::{Color, Highlight};
use ui::grid::row::Row;

pub struct Context {
    pub cairo_context: cairo::Context,
    pub pango_context: pango::Context,
    pub font_desc: FontDescription,
    pub cell_metrics: CellMetrics,
    pub rows: Vec<Row>,

    // row, col
    pub cursor: (u64, u64),
    pub cursor_alpha: f64,
    pub cursor_cell_percentage: f64,
    pub cursor_color: Color,

    pub current_hl: Highlight,
    pub active: bool,
}

impl Context {
    pub fn new(da: &DrawingArea) -> Self {
        let win = da.get_window().unwrap();
        let w = da.get_allocated_width();
        let h = da.get_allocated_height();
        let surface = win.create_similar_surface(cairo::Content::Color, w, h).unwrap();

        let cairo_context = cairo::Context::new(&surface);
        let pango_context = pangocairo::functions::create_context(&cairo_context).unwrap();

        let font_desc = FontDescription::from_string("Monospace 12");
        pango_context.set_font_description(&font_desc);

        let mut cell_metrics = CellMetrics::default();
        cell_metrics.update(&pango_context, &font_desc);

        Context {
            cairo_context,
            pango_context,
            font_desc,
            cell_metrics,
            rows: vec!(),

            cursor: (0, 0),
            cursor_alpha: 1.0,
            cursor_cell_percentage: 1.0,
            cursor_color: Color::from_u64(0),

            current_hl: Highlight::default(),
            active: false,
        }
    }

    pub fn update_font(&mut self, font_name: &str) {
        let mut font_desc = FontDescription::from_string(font_name);

        // Make sure we dont have a font with size of 0, otherwise we'll
        // have problems later.
        if font_desc.get_size() == 0 {
            font_desc.set_size(12 * pango::SCALE);
        }

        self.pango_context.set_font_description(&font_desc);
        self.font_desc = font_desc;
        self.cell_metrics.update(&self.pango_context, &self.font_desc);
    }

    pub fn update(&mut self, da: &DrawingArea) {
        let win = da.get_window().unwrap();
        let w = da.get_allocated_width();
        let h = da.get_allocated_height();
        let surface = win.create_similar_surface(cairo::Content::Color, w, h).unwrap();
        let ctx = cairo::Context::new(&surface);

        let s = self.cairo_context.get_target();
        self.cairo_context.save();
        ctx.set_source_surface(&s, 0.0, 0.0);
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint();
        self.cairo_context.restore();

        let pctx = pangocairo::functions::create_context(&ctx).unwrap();
        pctx.set_font_description(&self.font_desc);

        self.cairo_context = ctx;
        self.pango_context = pctx;

        self.cell_metrics.update(&self.pango_context, &self.font_desc);
    }
}

#[derive(Default, Debug)]
pub struct CellMetrics {
    pub height: f64,
    pub width: f64,
    pub ascent: f64,
    pub decent: f64,
    pub underline_thickness: f64,
    pub underline_position: f64,
}

impl CellMetrics {
    pub fn update(&mut self, ctx: &pango::Context, desc: &FontDescription) {
        let fm = ctx.get_metrics(Some(desc), None).unwrap();
        self.ascent = fm.get_ascent() as f64 / pango::SCALE as f64;
        self.decent = fm.get_descent() as f64 / pango::SCALE as f64;
        self.height = self.ascent + self.decent;
        self.width = (fm.get_approximate_digit_width() / pango::SCALE) as f64;

        self.underline_position = fm.get_underline_position() as f64 / pango::SCALE as f64;
        // TODO(ville): make the underline thickness a bit thicker (one 10th of the cell height?).
        self.underline_thickness = fm.get_underline_thickness() as f64 / pango::SCALE as f64 * 2.0;
    }
}
