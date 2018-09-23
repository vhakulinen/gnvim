use std::sync::{Arc, Mutex};

use pango::FontDescription;
use cairo;
use gtk::{DrawingArea};
use gtk;

use cairo::prelude::*;
use gtk::prelude::*;

use nvim_bridge::{GridLineSegment, ModeInfo};
use ui::ui::HlDefs;
use ui::grid::context::Context;
use ui::grid::render;
use ui::grid::row::Row;
use ui::color::Color;
use thread_guard::ThreadGuard;

pub struct Grid {
    da: ThreadGuard<DrawingArea>,
    context: Arc<ThreadGuard<Option<Context>>>,
    hl_defs: Arc<Mutex<HlDefs>>,
}

impl Grid {
    pub fn new(id: u64, container: &gtk::Container, hl_defs: Arc<Mutex<HlDefs>>) -> Self {

        let da = DrawingArea::new();
        let ctx = Arc::new(ThreadGuard::new(None));

        let ctx_ref = ctx.clone();
        da.connect_configure_event(move |da, _| {
            let mut ctx = ctx_ref.borrow_mut();
            if ctx.is_none() {
                *ctx = Some(Context::new(&da))
            } else {
                ctx.as_mut().unwrap().update(&da);
            }

            false
        });

        let ctx_ref = ctx.clone();
        da.connect_draw(move |_, cr| {
            let ctx = ctx_ref.clone();
            if let Some(ref mut ctx) = *ctx.borrow_mut() {
                drawingarea_draw(cr, ctx);
            }
            Inhibit(false)
        });

        container.add(&da);

        Grid {
            da: ThreadGuard::new(da),
            context: ctx,
            hl_defs,
        }
    }

    pub fn connect_da_resize<F: 'static>(&self, f: F)
        where F: Fn(u64, u64) -> bool {
        let da = self.da.borrow();
        let ctx = self.context.clone();

        // TODO(ville): Set resize timeout so this wont be triggered constantly.
        da.connect_configure_event(move |da, _| {
            let ctx = ctx.borrow();
            let ctx = ctx.as_ref().unwrap();

            let w = da.get_allocated_width();
            let h = da.get_allocated_height();
            let cols = (w / ctx.cell_metrics.width as i32) as u64;
            let rows = (h / ctx.cell_metrics.height as i32) as u64;

            f(rows, cols)
        });
    }

    pub fn put_line(&self, line: &GridLineSegment) {
        //let state = self.state.borrow();
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        let da = self.da.borrow();
        render::put_line(&da, ctx, line, &mut *self.hl_defs.lock().unwrap());
    }

    pub fn cursor_goto(&self, row: u64, col: u64) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();
        let da = self.da.borrow();

        // Clear old cursor position.
        let (x, y, w, h) = {
            let cm = &ctx.cell_metrics;
            let (x, y) = render::get_coords(cm.height,
                                            cm.width,
                                            ctx.cursor.0 as f64,
                                            ctx.cursor.1 as f64);
            (x, y, cm.width, cm.height)
        };
        da.queue_draw_area(x as i32, y as i32, w as i32, h as i32);

        ctx.cursor.0 = row;
        ctx.cursor.1 = col;

        // Update cursor color.
        let hl_defs = self.hl_defs.lock().unwrap();
        let row = ctx.rows.get(ctx.cursor.0 as usize).unwrap();
        let leaf = row.leaf_at(ctx.cursor.1 as usize + 1);
        let hl = hl_defs.get(&leaf.hl_id()).unwrap();
        ctx.cursor_color = hl.foreground.unwrap_or(hl_defs.default_fg);

        // Mark the new cursor position to be drawn.
        let (x, y, w, h) = {
            let cm = &ctx.cell_metrics;
            let (x, y) = render::get_coords(cm.height,
                                            cm.width,
                                            ctx.cursor.0 as f64,
                                            ctx.cursor.1 as f64);
            (x, y, cm.width, cm.height)
        };

        da.queue_draw_area(x as i32, y as i32, w as i32, h as i32);
    }

    pub fn resize(&self, width: u64, height: u64) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.rows = vec!();
        for _ in 0..height {
            ctx.rows.push(Row::new(width as usize));
        }
    }

    pub fn clear(&self) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();
        let da = self.da.borrow();
        let hl_defs = self.hl_defs.lock().unwrap();

        for row in ctx.rows.iter_mut() {
            row.clear();
        }

        render::clear(&da, ctx, &hl_defs)
    }

    pub fn scroll(&self, reg: [u64;4], rows: i64, cols: i64) {
        let mut ctx = self.context.borrow_mut();
        let mut ctx = ctx.as_mut().unwrap();
        let da = self.da.borrow();
        let hl_defs = self.hl_defs.lock().unwrap();

        render::scroll(&da, &mut ctx, &hl_defs, reg, rows);
    }

    pub fn set_active(&self, active: bool) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.active = active;
    }

    pub fn tick(&self) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();
        let da = self.da.borrow();

        ctx.cursor_alpha += 0.05;
        if ctx.cursor_alpha > 2.0 {
            ctx.cursor_alpha = 0.0;
        }

        let (x, y, w, h) = {
            let cm = &ctx.cell_metrics;
            let (x, y) = render::get_coords(cm.height,
                                            cm.width,
                                            ctx.cursor.0 as f64,
                                            ctx.cursor.1 as f64);
            (x, y, cm.width, cm.height)
        };

        da.queue_draw_area(x as i32, y as i32, w as i32, h as i32);
    }

    pub fn set_font(&self, name: String) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        let font_desc = FontDescription::from_string(&name);
        ctx.font_desc = font_desc;

        // Update the font metrics according to the new font. The parent (e.g.
        // the main UI) should tell neovim to resize according to the new size
        // if needed - this is when the main grid gets new font.
        // This is basically done by calling grid.calc_size and passing
        // the size to neovim.
        ctx.cell_metrics.update(&ctx.pango_context, &ctx.font_desc);

        // TODO(ville): How to handle font updates on other grids than the main grid?
    }

    pub fn set_mode(&self, mode: &ModeInfo) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.cursor_cell_percentage = mode.cell_percentage;
    }

    /// Calculates the current gird size. Returns (rows, cols).
    pub fn calc_size(&self) -> (usize, usize) {
        let da = self.da.borrow();
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().unwrap();

        let w = da.get_allocated_width();
        let h = da.get_allocated_height();
        let cols = (w / ctx.cell_metrics.width as i32) as usize;
        let rows = (h / ctx.cell_metrics.height as i32) as usize;

        (rows, cols)
    }

    pub fn get_row_text(&self, row: usize) -> String {
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().unwrap();

        ctx.rows.get(row).unwrap().text()
    }

    pub fn hl_id_at(&self, row: usize, col: usize) -> u64 {
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().unwrap();

        ctx.rows.get(row).unwrap().leaf_at(col).hl_id()
    }
}

fn drawingarea_draw(cr: &cairo::Context, ctx: &mut Context) {
    //println!("DRAW");

    //let ctx = ctx.lock().unwrap();
    let surface = ctx.cairo_context.get_target();
    surface.flush();

    cr.save();
    cr.set_source_surface(&surface, 0.0, 0.0);
    cr.paint();
    cr.restore();

    //cr.set_source_rgb(0.0, 255.0, 255.0);
    //cr.paint();

    let (x, y, w, h) = {
        let cm = &ctx.cell_metrics;
        let (x, y) = render::get_coords(cm.height,
                                        cm.width,
                                        ctx.cursor.0 as f64,
                                        ctx.cursor.1 as f64);
        (x, y, cm.width, cm.height)
    };

    let mut alpha = ctx.cursor_alpha;
    if alpha > 1.0 {
        alpha = 2.0 - alpha;
    }

    cr.save();
    cr.rectangle(x, y, w * ctx.cursor_cell_percentage, h);
    cr.set_source_rgba(ctx.cursor_color.r,
                       ctx.cursor_color.g,
                       ctx.cursor_color.b,
                       alpha);
    cr.fill();
    cr.restore();
}
