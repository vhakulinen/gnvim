use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::fmt;

use pango::FontDescription;
use cairo;
use gdk::{EventMask, ModifierType};
use gdk;
use gtk::{DrawingArea, EventBox};
use gtk;

use cairo::prelude::*;
use gtk::prelude::*;

use nvim_bridge::{GridLineSegment, ModeInfo};
use ui::ui::HlDefs;
use ui::grid::context::Context;
use ui::grid::render;
use ui::grid::row::Row;
use thread_guard::ThreadGuard;

pub enum ScrollDirection {
    Up,
    Down,
}

impl Display for ScrollDirection {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScrollDirection::Up => write!(fmt, "ScrollWheelUp"),
            ScrollDirection::Down => write!(fmt, "ScrollWheelDown"),
        }
    }
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Display for MouseButton {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MouseButton::Left => write!(fmt, "Left"),
            MouseButton::Middle => write!(fmt, "Middle"),
            MouseButton::Right => write!(fmt, "Right"),
        }
    }
}

/// Single grid in the neovim UI. This matches the `ui-linegrid` stuff in
/// the ui.txt documentation for neovim.
pub struct Grid {
    /// Our internal "widget". This is what is drawn to the screen.
    da: DrawingArea,
    /// EventBox to get mouse events for this grid.
    eb: EventBox,
    /// Internal context that is manipulated and used when handling events.
    context: Arc<ThreadGuard<Option<Context>>>,
    /// Reference to the highlight defs.
    hl_defs: Arc<Mutex<HlDefs>>,
    /// Pointer position for dragging if we should call callback from
    /// `connect_motion_events_for_drag`.
    drag_position: Arc<ThreadGuard<(u64, u64)>>,
}

impl Grid {
    pub fn new(_id: u64, hl_defs: Arc<Mutex<HlDefs>>) -> Self {
        let da = DrawingArea::new();
        let ctx = Arc::new(ThreadGuard::new(None));

        let ctx_ref = ctx.clone();
        da.connect_configure_event(move |da, _| {
            let mut ctx = ctx_ref.borrow_mut();
            if ctx.is_none() {
                // On initial expose, we'll need to create our internal context,
                // since this is the first time we'll have drawing area present...
                *ctx = Some(Context::new(&da));
            } else {
                // ...but if we already have context, our size is changing, so
                // we'll need to update our internals.
                ctx.as_mut().unwrap().update(&da);
            }

            false
        });

        let ctx_ref = ctx.clone();
        da.connect_draw(move |_, cr| {
            let ctx = ctx_ref.clone();
            if let Some(ref mut ctx) = *ctx.borrow_mut() {
                // After making sure we have our internal context, draw us (e.g.
                // our drawingarea) to the screen!
                drawingarea_draw(cr, ctx);
            }
            Inhibit(false)
        });


        let eb = EventBox::new();
        eb.add_events(EventMask::SCROLL_MASK.bits() as i32);
        eb.add(&da);

        Grid {
            da: da,
            eb: eb,
            context: ctx,
            hl_defs,
            drag_position: Arc::new(ThreadGuard::new((0, 0))),
        }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.eb.clone().upcast()
    }

    pub fn flush(&self) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        // Update cursor color.
        let hl_defs = self.hl_defs.lock().unwrap();
        let row = ctx.rows.get(ctx.cursor.0 as usize).unwrap();
        let leaf = row.leaf_at(ctx.cursor.1 as usize + 1);
        let hl = hl_defs.get(&leaf.hl_id()).unwrap();
        ctx.cursor_color = hl.foreground.unwrap_or(hl_defs.default_fg);

        while let Some(area) = ctx.queue_draw_area.pop() {
            self.da.queue_draw_area(area.0, area.1, area.2, area.3);
        }
    }

    /// Returns position (+ width and height) for cell (row, col) relative
    /// to the top level window of this grid.
    pub fn get_rect_for_cell(&self, row: u64, col: u64) -> gdk::Rectangle {
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().unwrap();

        let (x, y) = render::get_coords(
            ctx.cell_metrics.height,
            ctx.cell_metrics.width,
            row as f64,
            col as f64);

        let (x, y) = self.eb.translate_coordinates(
            &self.eb.get_toplevel().unwrap(), x as i32, y as i32).unwrap();

        gtk::Rectangle {
            x, y,
            width: ctx.cell_metrics.width as i32,
            height: ctx.cell_metrics.height as i32,
        }
    }

    /// Connects `f` to internal widget's scroll events. `f` params are scroll
    /// direction, row, col.
    pub fn connect_scroll_events<F: 'static>(&self, f: F)
        where F: Fn(ScrollDirection, u64, u64) -> Inhibit {
        let ctx = self.context.clone();

        self.eb.connect_scroll_event(move |_, e| {
            let ctx = ctx.borrow();
            let ctx = ctx.as_ref().unwrap();

            let dir = match e.get_direction() {
                gdk::ScrollDirection::Up => ScrollDirection::Up,
                _ => ScrollDirection::Down,
            };

            let pos = e.get_position();
            let col = (pos.0 / ctx.cell_metrics.width).floor() as u64;
            let row = (pos.1 / ctx.cell_metrics.height).floor() as u64;

            f(dir, row, col)
        });
    }

    /// Connects `f` to internal widget's motion events. `f` params are button,
    /// row, col. `f` is only called when the cell under the pointer changes.
    pub fn connect_motion_events_for_drag<F: 'static>(&self, f: F)
        where F: Fn(MouseButton, u64, u64) -> Inhibit {
        let ctx = self.context.clone();
        let drag_position = self.drag_position.clone();

        self.eb.connect_motion_notify_event(move |_, e| {
            let ctx = ctx.borrow();
            let ctx = ctx.as_ref().unwrap();
            let mut drag_position = drag_position.borrow_mut();

            let button = match e.get_state() {
                ModifierType::BUTTON3_MASK => MouseButton::Right,
                ModifierType::BUTTON2_MASK => MouseButton::Middle,
                _ => MouseButton::Left,
            };

            let pos = e.get_position();
            let col = (pos.0 / ctx.cell_metrics.width).floor() as u64;
            let row = (pos.1 / ctx.cell_metrics.height).floor() as u64;

            if drag_position.0 != col || drag_position.1 != row {
                *drag_position = (col, row);
                f(button, row, col)
            } else {
                Inhibit(false)
            }
        });
    }

    /// Connects `f` to internal widget's mouse button press event. `f` params
    /// are button, row, col.
    pub fn connect_mouse_button_press_events<F: 'static>(&self, f: F)
        where F: Fn(MouseButton, u64, u64) -> Inhibit {
        let ctx = self.context.clone();

        self.eb.connect_button_press_event(move |_, e| {
            let ctx = ctx.borrow();
            let ctx = ctx.as_ref().unwrap();

            let button = match e.get_button() {
                3 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => MouseButton::Left,
            };

            let pos = e.get_position();
            let col = (pos.0 / ctx.cell_metrics.width).floor() as u64;
            let row = (pos.1 / ctx.cell_metrics.height).floor() as u64;

            f(button, row, col)
        });
    }

    /// Connects `f` to internal widget's mouse button release event. `f` params
    /// are button, row, col.
    pub fn connect_mouse_button_release_events<F: 'static>(&self, f: F)
        where F: Fn(MouseButton, u64, u64) -> Inhibit {
        let ctx = self.context.clone();

        self.eb.connect_button_release_event(move |_, e| {
            let ctx = ctx.borrow();
            let ctx = ctx.as_ref().unwrap();

            let button = match e.get_button() {
                3 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => MouseButton::Left,
            };

            let pos = e.get_position();
            let col = (pos.0 / ctx.cell_metrics.width).floor() as u64;
            let row = (pos.1 / ctx.cell_metrics.height).floor() as u64;

            f(button, row, col)
        });
    }

    /// Connects `f` to internal widget's resize events. `f` params are rows, cols.
    pub fn connect_da_resize<F: 'static>(&self, f: F)
        where F: Fn(u64, u64) -> bool {
        let ctx = self.context.clone();

        self.da.connect_configure_event(move |da, _| {
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
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        render::put_line(&self.da, ctx, line, &mut *self.hl_defs.lock().unwrap());
    }

    pub fn cursor_goto(&self, row: u64, col: u64) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        // Clear old cursor position.
        let (x, y, w, h) = {
            let cm = &ctx.cell_metrics;
            let (x, y) = render::get_coords(cm.height,
                                            cm.width,
                                            ctx.cursor.0 as f64,
                                            ctx.cursor.1 as f64);
            (x, y, cm.width, cm.height)
        };
        ctx.queue_draw_area.push((x as i32, y as i32, w as i32, h as i32));

        ctx.cursor.0 = row;
        ctx.cursor.1 = col;

        // Mark the new cursor position to be drawn.
        let (x, y, w, h) = {
            let cm = &ctx.cell_metrics;
            let (x, y) = render::get_coords(cm.height,
                                            cm.width,
                                            ctx.cursor.0 as f64,
                                            ctx.cursor.1 as f64);
            (x, y, cm.width, cm.height)
        };
        ctx.queue_draw_area.push((x as i32, y as i32, w as i32, h as i32));
    }

    pub fn resize(&self, width: u64, height: u64) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        // Clear internal grid (rows).
        ctx.rows = vec!();
        for _ in 0..height {
            ctx.rows.push(Row::new(width as usize));
        }
    }

    pub fn clear(&self) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();
        let hl_defs = self.hl_defs.lock().unwrap();

        // Clear internal grid (rows).
        for row in ctx.rows.iter_mut() {
            row.clear();
        }

        render::clear(&self.da, ctx, &hl_defs)
    }

    pub fn scroll(&self, reg: [u64;4], rows: i64, _cols: i64) {
        let mut ctx = self.context.borrow_mut();
        let mut ctx = ctx.as_mut().unwrap();
        let hl_defs = self.hl_defs.lock().unwrap();

        render::scroll(&self.da, &mut ctx, &hl_defs, reg, rows);
    }

    pub fn set_active(&self, active: bool) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.active = active;
    }

    pub fn tick(&self) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

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

        // Don't use the ctx.queue_draw_area, because those draws will only
        // happen once nvim sends 'flush' event. This draw needs to happen
        // on each tick so the cursor blinks.
        self.da.queue_draw_area(x as i32, y as i32, w as i32, h as i32);
    }

    pub fn set_font(&self, font: FontDescription) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();
        ctx.update_font(font);
    }

    pub fn set_mode(&self, mode: &ModeInfo) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.cursor_cell_percentage = mode.cell_percentage;
    }

    pub fn set_busy(&self, busy: bool) {
        let mut ctx = self.context.borrow_mut();
        let ctx = ctx.as_mut().unwrap();

        ctx.busy = busy;
    }

    /// Calculates the current gird size. Returns (rows, cols).
    pub fn calc_size(&self) -> (usize, usize) {
        let ctx = self.context.borrow();
        let ctx = ctx.as_ref().unwrap();

        let w = self.da.get_allocated_width();
        let h = self.da.get_allocated_height();
        let cols = (w / ctx.cell_metrics.width as i32) as usize;
        let rows = (h / ctx.cell_metrics.height as i32) as usize;

        (rows, cols)
    }
}

/// Handler for grid's drawingarea's draw event. Draws the internal cairo
/// context (`ctx`) surface to the `cr`.
fn drawingarea_draw(cr: &cairo::Context, ctx: &mut Context) {
    let surface = ctx.cairo_context.get_target();
    surface.flush();

    cr.save();
    cr.set_source_surface(&surface, 0.0, 0.0);
    cr.paint();
    cr.restore();

    // If we're not "busy", draw the cursor.
    if !ctx.busy {

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
}
