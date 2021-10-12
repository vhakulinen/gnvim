use gtk::prelude::*;
use gtk::DrawingArea;
use gtk::{cairo, gdk, pango};

use crate::error::Error;
use crate::ui::color::HlDefs;
use crate::ui::font::Font;
use crate::ui::grid::cursor::Cursor;
use crate::ui::grid::render;
use crate::ui::grid::row::{Cell, Row};

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

    pub cursor: Cursor,
    /// Cairo context for cursor.
    pub cursor_context: cairo::Context,

    /// If the current status is busy or not. When busy, the cursor is not
    /// drawn (like when in terminal mode in inserting text).
    pub busy: bool,

    /// If the grid that this context belongs to is active or not.
    pub active: bool,

    /// Areas to call queue_draw_area on the drawing area on flush.
    pub queue_draw_area: Vec<(f64, f64, f64, f64)>,
}

impl Context {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        da: &DrawingArea,
        win: &gdk::Window,
        font: Font,
        line_space: i64,
        cols: usize,
        rows: usize,
        hl_defs: &HlDefs,
        enable_cursor_animations: bool,
    ) -> Result<Self, Error> {
        let pango_context = da.pango_context();

        let font_desc = font.as_pango_font();
        pango_context.set_font_description(&font_desc);

        let mut cell_metrics = CellMetrics::default();
        cell_metrics.font = font;
        cell_metrics.line_space = line_space;
        cell_metrics.update(&pango_context)?;

        let w = cell_metrics.width * cols as f64;
        let h = cell_metrics.height * rows as f64;
        let surface = win
            .create_similar_surface(
                cairo::Content::Color,
                w.ceil() as i32,
                h.ceil() as i32,
            )
            .ok_or(Error::FailedToCreateSurface())?;

        let cairo_context = cairo::Context::new(&surface)?;

        // Fill the context with default bg color.
        cairo_context.save()?;
        cairo_context.set_source_rgb(
            hl_defs.default_bg.r,
            hl_defs.default_bg.g,
            hl_defs.default_bg.b,
        );
        cairo_context.paint()?;
        cairo_context.restore()?;

        let cursor_context = {
            let surface = win
                .create_similar_surface(
                    cairo::Content::ColorAlpha,
                    (cell_metrics.width * 2.0) as i32, // times two for double width chars.
                    (cell_metrics.height + cell_metrics.ascent).ceil() as i32,
                )
                .ok_or(Error::FailedToCreateSurface())?;
            cairo::Context::new(&surface)?
        };

        let cursor = Cursor {
            disable_animation: !enable_cursor_animations,
            ..Cursor::default()
        };

        Ok(Context {
            cairo_context,
            cell_metrics,
            cell_metrics_update: None,
            rows: vec![],

            cursor,
            cursor_context,

            busy: false,
            active: false,

            queue_draw_area: vec![],
        })
    }

    /// Updates internals that are dependant on the drawing area.
    pub fn resize(
        &mut self,
        da: &DrawingArea,
        win: &gdk::Window,
        cols: usize,
        rows: usize,
        hl_defs: &HlDefs,
    ) -> Result<(), Error> {
        let prev_rows = self.rows.len();
        let prev_cols = self.rows.get(0).map(|r| r.len()).unwrap_or(0);

        if self.rows.len() != rows {
            self.rows.resize_with(rows, || Row::new(cols));
        }

        if self.rows.get(0).unwrap().len() != cols {
            for row in self.rows.iter_mut() {
                row.resize(cols);
            }
        }

        let pctx = da.pango_context();
        pctx.set_font_description(&self.cell_metrics.font.as_pango_font());

        self.cell_metrics.update(&pctx)?;

        let w = self.cell_metrics.width * cols as f64;
        let h = self.cell_metrics.height * rows as f64;
        let surface = win
            .create_similar_surface(
                cairo::Content::Color,
                w.ceil() as i32,
                h.ceil() as i32,
            )
            .ok_or(Error::FailedToCreateSurface())?;
        let ctx = cairo::Context::new(&surface)?;

        // Fill the context with default bg color.
        ctx.save()?;
        ctx.set_source_rgb(
            hl_defs.default_bg.r,
            hl_defs.default_bg.g,
            hl_defs.default_bg.b,
        );
        ctx.paint()?;
        ctx.restore()?;

        let s = self.cairo_context.target();
        self.cairo_context.save()?;
        ctx.set_source_surface(&s, 0.0, 0.0)?;
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
        ctx.fill()?;
        self.cairo_context.restore()?;

        self.cairo_context = ctx;

        Ok(())
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
    ) -> Result<(), Error> {
        let pango_context = da.pango_context();
        pango_context.set_font_description(&font.as_pango_font());

        self.cell_metrics.font = font;
        self.cell_metrics.line_space = line_space;
        self.cell_metrics.update(&pango_context)?;

        self.cursor_context = {
            let surface = win
                .create_similar_surface(
                    cairo::Content::ColorAlpha,
                    (self.cell_metrics.width * 2.0).ceil() as i32, // times two for double width chars.
                    (self.cell_metrics.height + self.cell_metrics.ascent).ceil()
                        as i32,
                )
                .ok_or(Error::FailedToCreateSurface())?;
            cairo::Context::new(&surface)?
        };

        Ok(())
    }

    /// Returns x, y, width and height for cursor position on the screen (e.g. might be in middle
    /// of an animation).
    pub fn get_cursor_rect(&self) -> (i32, i32, i32, i32) {
        let double_width = self
            .cell_at_cursor()
            .map(|cell| cell.double_width)
            .unwrap_or(false);

        // Dont use cursor.get_position here, because we want to use the position on the screen.
        let pos = self.cursor.pos.unwrap_or((0.0, 0.0));

        let cm = &self.cell_metrics;
        let (x, y) = render::get_coords(cm.height, cm.width, pos.0, pos.1);
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

    pub fn cursor_goto(&mut self, row: u64, col: u64, clock: &gdk::FrameClock) {
        // Clear old cursor position.
        let (x, y, w, h) = self.get_cursor_rect();
        self.queue_draw_area.push((
            f64::from(x),
            f64::from(y),
            f64::from(w),
            f64::from(h),
        ));
        self.cursor.goto(row as f64, col as f64, clock.frame_time());

        // Mark the new cursor position to be drawn.
        let (x, y, w, h) = self.get_cursor_rect();
        self.queue_draw_area.push((
            f64::from(x),
            f64::from(y),
            f64::from(w),
            f64::from(h),
        ));
    }

    pub fn tick(
        &mut self,
        da: &DrawingArea,
        clock: &gdk::FrameClock,
    ) -> Result<(), Error> {
        let (x, y, w, h) = self.get_cursor_rect();
        da.queue_draw_area(x, y, w, h);

        self.cursor.tick(clock.frame_time());

        // We're not blinking, so skip the blink animation phase.
        if self.cursor.blink_on == 0 {
            return Ok(());
        }

        let (x, y, w, h) = self.get_cursor_rect();

        let mut alpha = self.cursor.alpha;
        if alpha > 1.0 {
            alpha = 2.0 - alpha;
        }

        let cr = &self.cursor_context;
        cr.save()?;
        // Draw the cursor surface. Make it double width, so our cursor
        // will always be wide enough (it'll get clipped if needed).
        cr.rectangle(
            0.0,
            0.0,
            self.cell_metrics.width * 2.0,
            self.cell_metrics.height,
        );
        cr.set_operator(cairo::Operator::Source);
        cr.set_source_rgba(
            self.cursor.color.r,
            self.cursor.color.g,
            self.cursor.color.b,
            alpha,
        );
        cr.fill()?;
        cr.restore()?;

        // Don't use the queue_draw_area, because those draws will only
        // happen once nvim sends 'flush' event. This draw needs to happen
        // on each tick so the cursor blinks.
        da.queue_draw_area(x, y, w, h);

        Ok(())
    }

    pub fn cell_at_cursor(&self) -> Option<&Cell> {
        self.cursor.get_position().and_then(|pos| {
            self.rows
                .get(pos.0.ceil() as usize)
                .and_then(|row| row.cell_at(pos.1.ceil() as usize))
        })
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
    pub fn update(&mut self, ctx: &pango::Context) -> Result<(), Error> {
        let fm = ctx
            .metrics(Some(&self.font.as_pango_font()), None)
            .ok_or(Error::GetPangoMetrics())?;
        let extra = self.line_space as f64 / 2.0;
        let scale = f64::from(pango::SCALE);
        self.ascent = (f64::from(fm.ascent()) / scale + extra).ceil();
        self.decent = (f64::from(fm.descent()) / scale + extra).ceil();
        self.height = self.ascent + self.decent;
        self.width = f64::from(fm.approximate_char_width()) / scale;

        self.underline_position =
            f64::from(fm.underline_position()) / scale - extra;
        // TODO(ville): make the underline thickness a bit thicker (one 10th of the cell height?).
        self.underline_thickness =
            f64::from(fm.underline_thickness()) / scale * 2.0;

        Ok(())
    }
}
