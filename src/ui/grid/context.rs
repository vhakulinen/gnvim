use cairo;
use gtk::prelude::*;
use gtk::DrawingArea;
use pango;

use crate::ui::color::{Color, Highlight, HlDefs};
use crate::ui::font::Font;
use crate::ui::grid::render;
use crate::ui::grid::row::Row;

/// Context is manipulated by Grid.
pub struct Context {
    /// Our cairo context, that is evetually drawn to the screen.
    pub cairo_context: cairo::Context,
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
    /// Cairo context for cursor.
    pub cursor_context: cairo::Context,

    /// Current highlight.
    pub current_hl: Highlight,
    /// If the grid that this context belongs to is active or not.
    pub active: bool,

    /// Areas to call queue_draw_area on the drawing area on flush.
    pub queue_draw_area: Vec<(f64, f64, f64, f64)>,
}

impl Context {
    pub fn new(
        da: &DrawingArea,
        win: &gdk::Window,
        font: Font,
        line_space: i64,
        cols: usize,
        rows: usize,
        hl_defs: &HlDefs,
    ) -> Self {
        let pango_context = da.get_pango_context().unwrap();

        let font_desc = font.as_pango_font();
        pango_context.set_font_description(&font_desc);

        let mut cell_metrics = CellMetrics::default();
        cell_metrics.font = font;
        cell_metrics.line_space = line_space;
        cell_metrics.update(&pango_context);

        let w = cell_metrics.width * cols as f64;
        let h = cell_metrics.height * rows as f64;
        let surface = win
            .create_similar_surface(
                cairo::Content::Color,
                w.ceil() as i32,
                h.ceil() as i32,
            )
            .unwrap();

        let cairo_context = cairo::Context::new(&surface);

        // Fill the context with default bg color.
        cairo_context.save();
        cairo_context.set_source_rgb(
            hl_defs.default_bg.r,
            hl_defs.default_bg.g,
            hl_defs.default_bg.b,
        );
        cairo_context.paint();
        cairo_context.restore();

        let cursor_context = {
            let surface = win
                .create_similar_surface(
                    cairo::Content::ColorAlpha,
                    (cell_metrics.width * 2.0) as i32, // times two for double width chars.
                    (cell_metrics.height + cell_metrics.ascent).ceil() as i32,
                )
                .unwrap();
            cairo::Context::new(&surface)
        };

        Context {
            cairo_context,
            cell_metrics,
            cell_metrics_update: None,
            rows: vec![],

            cursor: (0, 0),
            cursor_alpha: 1.0,
            cursor_blink_on: 0,
            cursor_cell_percentage: 1.0,
            cursor_color: Color::from_u64(0),
            busy: false,
            cursor_context,

            current_hl: Highlight::default(),
            active: false,

            queue_draw_area: vec![],
        }
    }

    /// Updates internals that are dependant on the drawing area.
    pub fn resize(
        &mut self,
        da: &DrawingArea,
        win: &gdk::Window,
        cols: usize,
        rows: usize,
        hl_defs: &HlDefs,
    ) {
        let prev_rows = self.rows.len();
        let prev_cols = self.rows.get(0).map(|r| r.len()).unwrap_or(0);

        if self.rows.len() != rows {
            self.rows.resize_with(rows, || Row::new(rows));
        }

        if self.rows.get(0).unwrap().len() != cols {
            for row in self.rows.iter_mut() {
                row.resize(cols);
            }
        }

        let pctx = da.get_pango_context().unwrap();
        pctx.set_font_description(&self.cell_metrics.font.as_pango_font());

        self.cell_metrics.update(&pctx);

        let w = self.cell_metrics.width * cols as f64;
        let h = self.cell_metrics.height * rows as f64;
        let surface = win
            .create_similar_surface(
                cairo::Content::Color,
                w.ceil() as i32,
                h.ceil() as i32,
            )
            .unwrap();
        let ctx = cairo::Context::new(&surface);

        // Fill the context with default bg color.
        ctx.save();
        ctx.set_source_rgb(
            hl_defs.default_bg.r,
            hl_defs.default_bg.g,
            hl_defs.default_bg.b,
        );
        ctx.paint();
        ctx.restore();

        let s = self.cairo_context.get_target();
        self.cairo_context.save();
        ctx.set_source_surface(&s, 0.0, 0.0);
        ctx.set_operator(cairo::Operator::Source);
        // Make sure we only paint the area that _was_ visible before this update
        // so we don't undo the bg color paint we did earlier. Note that we're
        // calculating the used area based on the current cell metrics. This is
        // becuase if font changes that might reduce the area we "have available".
        // Otherwise, when changing to smaller font, we might draw our "old" surface
        // on a area that wont be cleared by nvim (e.g. over "fresh" whitespace).
        ctx.rectangle(
            0.0,
            0.0,
            self.cell_metrics.width * prev_cols as f64,
            self.cell_metrics.height * prev_rows as f64,
        );
        ctx.fill();
        self.cairo_context.restore();

        self.cairo_context = ctx;
    }

    /// Sets the cell metrics to be updated. If font or line_space is None,
    /// the earlier value for each is used. Call `finish_metrics_update` to
    /// make the update take place.
    pub fn update_metrics(
        &mut self,
        font: Font,
        line_space: i64,
        da: &gtk::DrawingArea,
        win: &gdk::Window,
    ) {
        let pango_context = da.get_pango_context().unwrap();
        pango_context.set_font_description(&font.as_pango_font());

        self.cell_metrics.font = font;
        self.cell_metrics.line_space = line_space;
        self.cell_metrics.update(&pango_context);

        self.cursor_context = {
            let surface = win
                .create_similar_surface(
                    cairo::Content::ColorAlpha,
                    (self.cell_metrics.width * 2.0).ceil() as i32, // times two for double width chars.
                    (self.cell_metrics.height + self.cell_metrics.ascent).ceil()
                        as i32,
                )
                .unwrap();
            cairo::Context::new(&surface)
        };
    }

    /// Returns x, y, width and height for current cursor location.
    pub fn get_cursor_rect(&self) -> (i32, i32, i32, i32) {
        let double_width = self
            .rows
            .get(self.cursor.0 as usize)
            .and_then(|row| {
                Some(
                    row.cell_at(self.cursor.1 as usize)
                        .map(|c| c.double_width)
                        .unwrap_or(false),
                )
            })
            .unwrap_or(false);

        let cm = &self.cell_metrics;
        let (x, y) = render::get_coords(
            cm.height,
            cm.width,
            self.cursor.0 as f64,
            self.cursor.1 as f64,
        );
        (
            x.floor() as i32,
            y.floor() as i32,
            if double_width {
                (cm.width * 2.0).ceil() as i32
            } else {
                cm.width.ceil() as i32
            },
            cm.height.ceil() as i32,
        )
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
    pub font: Font,
}

impl CellMetrics {
    pub fn update(&mut self, ctx: &pango::Context) {
        let fm = ctx
            .get_metrics(Some(&self.font.as_pango_font()), None)
            .unwrap();
        let extra = self.line_space as f64 / 2.0;
        let scale = f64::from(pango::SCALE);
        self.ascent = (f64::from(fm.get_ascent()) / scale + extra).ceil();
        self.decent = (f64::from(fm.get_descent()) / scale + extra).ceil();
        self.height = self.ascent + self.decent;
        self.width = f64::from(fm.get_approximate_char_width()) / scale;

        self.underline_position =
            f64::from(fm.get_underline_position()) / scale - extra;
        // TODO(ville): make the underline thickness a bit thicker (one 10th of the cell height?).
        self.underline_thickness =
            f64::from(fm.get_underline_thickness()) / scale * 2.0;
    }
}
