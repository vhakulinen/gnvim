use cairo;
use gtk::DrawingArea;
use pango;
use pango::FontDescription;
use pangocairo;

use gtk::prelude::*;
use pango::prelude::*;

use ui::color::{Color, Highlight};
use ui::grid::row::Row;

/// Context is manipulated by Grid.
pub struct Context {
    /// Our cairo context, that is evetually drawn to the screen.
    pub cairo_context: cairo::Context,
    /// Our pango context.
    pub pango_context: pango::Context,
    /// Our cell metrics.
    pub cell_metrics: CellMetrics,
    /// Cell metrics to be updated.
    pub cell_metrics_update: Option<CellMetrics>,

    /// Internal grid.
    pub rows: Vec<Row>,

    /// Cursor, (row, col):
    pub cursor: (u64, u64),
    /// Cursor alpha color. Used to make the cursor blink.
    pub cursor_alpha: f64,
    /// The duration of the cursor blink
    pub cursor_blink_on: u64,
    /// Width of the cursor.
    pub cursor_cell_percentage: f64,
    /// Color of the cursor.
    pub cursor_color: Color,
    /// If the current status is busy or not. When busy, the cursor is not
    /// drawn (like when in terminal mode in inserting text).
    pub busy: bool,

    /// Current highlight.
    pub current_hl: Highlight,
    /// If the grid that this context belongs to is active or not.
    pub active: bool,

    /// Areas to call queue_draw_area on the drawing area on flush.
    pub queue_draw_area: Vec<(i32, i32, i32, i32)>,
}

impl Context {
    pub fn new(da: &DrawingArea) -> Self {
        let win = da.get_window().unwrap();
        let w = da.get_allocated_width();
        let h = da.get_allocated_height();
        let surface = win
            .create_similar_surface(cairo::Content::Color, w, h)
            .unwrap();

        let cairo_context = cairo::Context::new(&surface);
        let pango_context =
            pangocairo::functions::create_context(&cairo_context).unwrap();

        let font_desc = FontDescription::from_string("Monospace 12");
        pango_context.set_font_description(&font_desc);

        let mut cell_metrics = CellMetrics::default();
        cell_metrics.font_desc = font_desc;
        cell_metrics.line_space = 0;
        cell_metrics.update(&pango_context);

        Context {
            cairo_context,
            pango_context,
            cell_metrics,
            cell_metrics_update: None,
            rows: vec![],

            cursor: (0, 0),
            cursor_alpha: 1.0,
            cursor_blink_on: 0,
            cursor_cell_percentage: 1.0,
            cursor_color: Color::from_u64(0),
            busy: false,

            current_hl: Highlight::default(),
            active: false,

            queue_draw_area: vec![],
        }
    }

    /// Updates internals that are dependant on the drawing area.
    pub fn update(&mut self, da: &DrawingArea) {
        let win = da.get_window().unwrap();
        let w = da.get_allocated_width();
        let h = da.get_allocated_height();
        let surface = win
            .create_similar_surface(cairo::Content::Color, w, h)
            .unwrap();
        let ctx = cairo::Context::new(&surface);

        let s = self.cairo_context.get_target();
        self.cairo_context.save();
        ctx.set_source_surface(&s, 0.0, 0.0);
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint();
        self.cairo_context.restore();

        let pctx = pangocairo::functions::create_context(&ctx).unwrap();
        pctx.set_font_description(&self.cell_metrics.font_desc);

        self.cairo_context = ctx;
        self.pango_context = pctx;

        self.cell_metrics.update(&self.pango_context);
    }

    /// Sets the cell metrics to be updated. If font or line_space is None,
    /// the earlier value for each is used. Call `finish_metrics_update` to
    /// make the update take place.
    pub fn update_metrics(
        &mut self,
        mut font: Option<FontDescription>,
        mut line_space: Option<i64>,
    ) {
        if self.cell_metrics_update.is_none() {
            self.cell_metrics_update = Some(self.cell_metrics.clone());
        }

        if let Some(ref mut cm) = self.cell_metrics_update {
            if let Some(font) = font.take() {
                cm.font_desc = font;
            }
            if let Some(line_space) = line_space.take() {
                cm.line_space = line_space;
            }

            cm.update(&self.pango_context);
        }
    }

    /// Finishes cell metrics update if one is on going.
    pub fn finish_metrics_update(&mut self) {
        if let Some(cm) = self.cell_metrics_update.take() {
            self.pango_context.set_font_description(&cm.font_desc);
            self.cell_metrics = cm;
        }
    }
}

/// Cell metrics tells the size (and other metrics) of the cells in a grid.
#[derive(Default, Debug, Clone)]
pub struct CellMetrics {
    pub height: f64,
    pub width: f64,
    pub ascent: f64,
    pub decent: f64,
    pub underline_thickness: f64,
    pub underline_position: f64,

    pub line_space: i64,
    pub font_desc: FontDescription,
}

impl CellMetrics {
    pub fn update(&mut self, ctx: &pango::Context) {
        let fm = ctx.get_metrics(Some(&self.font_desc), None).unwrap();
        let extra = self.line_space as f64 / 2.0;
        self.ascent = fm.get_ascent() as f64 / pango::SCALE as f64 + extra;
        self.decent = fm.get_descent() as f64 / pango::SCALE as f64 + extra;
        self.height = self.ascent + self.decent;
        self.width = (fm.get_approximate_digit_width() / pango::SCALE) as f64;

        self.underline_position =
            fm.get_underline_position() as f64 / pango::SCALE as f64 - extra;
        // TODO(ville): make the underline thickness a bit thicker (one 10th of the cell height?).
        self.underline_thickness =
            fm.get_underline_thickness() as f64 / pango::SCALE as f64 * 2.0;
    }
}
