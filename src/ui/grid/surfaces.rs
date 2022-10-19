use gtk::{cairo, gdk};

use crate::error::Error;
use crate::ui::animation::Animation;
use crate::ui::color::Color;

use super::CellMetrics;

pub struct Surfaces {
    // Front buffer is where all the new content will be drawn inbetween
    // draw signals.
    pub front: cairo::Context,
    // Back buffer is where, when required, contents of previous draw iteration
    // is kept.
    pub back: cairo::Context,
    // Prev is a intermediate buffer to which front and back buffers are drawn
    // before the contents are drawn to the screen. This allows us to aniamte
    // grid_scroll changes.
    pub prev: cairo::Context,

    pub offset_y: f64,
    pub offset_y_anim: Option<Animation<f64>>,
}

impl Surfaces {
    pub fn new(
        win: &gdk::Window,
        cell_metrics: &CellMetrics,
        rows: usize,
        cols: usize,
        fill: &Color,
    ) -> Result<Self, Error> {
        Ok(Surfaces {
            front: Self::create_surface(win, cell_metrics, rows, cols, fill)?,
            back: Self::create_surface(win, cell_metrics, rows, cols, fill)?,
            prev: Self::create_surface(win, cell_metrics, rows, cols, fill)?,

            offset_y: 0.0,
            offset_y_anim: None,
        })
    }

    fn create_surface(
        win: &gdk::Window,
        cell_metrics: &CellMetrics,
        rows: usize,
        cols: usize,
        fill: &Color,
    ) -> Result<cairo::Context, Error> {
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

        cairo_context.save()?;
        cairo_context.set_source_rgb(fill.r, fill.g, fill.b);
        cairo_context.paint()?;
        cairo_context.restore()?;

        Ok(cairo_context)
    }

    pub fn set_animation(&mut self, y: f64, duration_ms: i64, ft_now: i64) {
        self.offset_y_anim = Some(Animation {
            start: -y + self.offset_y,
            end: 0.0,
            start_time: ft_now,
            end_time: ft_now + 1000 * duration_ms,
        });
    }

    pub fn tick(&mut self, ft: i64) -> bool {
        if let Some(ref anim) = self.offset_y_anim {
            if let Some(t) = anim.tick(ft) {
                // NOTE(ville): There are some precision issues when rendeing, hence the floor.
                self.offset_y =
                    (anim.start + t * (anim.end - anim.start)).floor();
            } else {
                self.offset_y = anim.end;
                self.offset_y_anim = None;
            }

            true
        } else {
            false
        }
    }
}
