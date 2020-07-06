use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use cairo;
use gdk;
use gdk::{EventMask, ModifierType};
use gtk;
use gtk::{DrawingArea, EventBox};

use gtk::prelude::*;

use crate::nvim_bridge::{GridLineSegment, ModeInfo};
use crate::ui::color::HlDefs;
use crate::ui::font::Font;
use crate::ui::grid::context::Context;
use crate::ui::grid::render;

pub struct GridMetrics {
    // Row count in the grid.
    pub rows: f64,
    // Col count in the grid.
    pub cols: f64,
    // Height of a cell.
    pub cell_height: f64,
    // Width of a cell.
    pub cell_width: f64,

    // Width of the whole grid as required by the cell width and cols.
    pub height: f64,
    // Height of the whole grid as required by the cell height and rows.
    pub width: f64,
}

pub enum ScrollDirection {
    Up,
    Down,
}

impl Display for ScrollDirection {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScrollDirection::Up => write!(fmt, "up"),
            ScrollDirection::Down => write!(fmt, "down"),
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
            MouseButton::Left => write!(fmt, "left"),
            MouseButton::Middle => write!(fmt, "middle"),
            MouseButton::Right => write!(fmt, "right"),
        }
    }
}

/// Single grid in the neovim UI. This matches the `ui-linegrid` stuff in
/// the ui.txt documentation for neovim.
pub struct Grid {
    pub id: i64,
    /// Our internal "widget". This is what is drawn to the screen.
    da: DrawingArea,
    /// EventBox to get mouse events for this grid.
    eb: EventBox,
    /// Internal context that is manipulated and used when handling events.
    context: Rc<RefCell<Context>>,
    /// Pointer position for dragging if we should call callback from
    /// `connect_motion_events_for_drag`.
    drag_position: Rc<RefCell<(u64, u64)>>,
    /// Input context that need to be updated for the cursor position
    im_context: Option<gtk::IMMulticontext>,
}

impl Grid {
    pub fn new(
        id: i64,
        win: &gdk::Window,
        font: Font,
        line_space: i64,
        cols: usize,
        rows: usize,
        hl_defs: &HlDefs,
    ) -> Self {
        let da = DrawingArea::new();
        let ctx = Rc::new(RefCell::new(Context::new(
            &da, win, font, line_space, cols, rows, hl_defs,
        )));

        da.connect_draw(clone!(ctx => move |_, cr| {
            let mut ctx = ctx.borrow_mut();
            drawingarea_draw(cr, &mut ctx);
            Inhibit(false)
        }));

        let eb = EventBox::new();
        eb.add_events(EventMask::SCROLL_MASK);
        eb.add(&da);

        Grid {
            id,
            da: da,
            eb: eb,
            context: ctx,
            drag_position: Rc::new(RefCell::new((0, 0))),
            im_context: None,
        }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.eb.clone().upcast()
    }

    pub fn flush(&self, hl_defs: &HlDefs) {
        let mut ctx = self.context.borrow_mut();

        if let Some(Some(cell)) = ctx
            .rows
            .get(ctx.cursor.0 as usize)
            .map(|row| row.cell_at(ctx.cursor.1 as usize))
        {
            // If cursor isn't blinking, drawn the inverted cell into
            // the cursor's cairo context.
            if ctx.cursor_blink_on == 0 {
                render::cursor_cell(
                    &ctx.cursor_context,
                    &self.da.get_pango_context().unwrap(),
                    &cell,
                    &ctx.cell_metrics,
                    hl_defs,
                );
            }

            // Update cursor color.
            let hl = hl_defs.get(&cell.hl_id).unwrap();
            ctx.cursor_color = hl.foreground.unwrap_or(hl_defs.default_fg);
        }

        while let Some(area) = ctx.queue_draw_area.pop() {
            self.da.queue_draw_area(
                area.0.floor() as i32,
                area.1.floor() as i32,
                area.2.ceil() as i32,
                area.3.ceil() as i32,
            );
        }
    }

    pub fn set_im_context(&mut self, im_context: &gtk::IMMulticontext) {
        im_context.set_client_window(self.da.get_window().as_ref());
        self.im_context = Some(im_context.clone());
    }

    /// Returns position (+ width and height) for cell (row, col) relative
    /// to the top level window of this grid.
    pub fn get_rect_for_cell(&self, row: u64, col: u64) -> gdk::Rectangle {
        let ctx = self.context.borrow();

        let (x, y) = render::get_coords(
            ctx.cell_metrics.height,
            ctx.cell_metrics.width,
            row as f64,
            col as f64,
        );

        let (x, y) = self
            .eb
            .translate_coordinates(
                &self.eb.get_parent().unwrap(),
                x as i32,
                y as i32,
            )
            .unwrap();

        gtk::Rectangle {
            x,
            y,
            width: ctx.cell_metrics.width as i32,
            height: ctx.cell_metrics.height as i32,
        }
    }

    /// Connects `f` to internal widget's scroll events. `f` params are scroll
    /// direction, row, col.
    pub fn connect_scroll_events<F: 'static>(&self, f: F)
    where
        F: Fn(ScrollDirection, u64, u64) -> Inhibit,
    {
        let ctx = self.context.clone();

        self.eb.connect_scroll_event(move |_, e| {
            let ctx = ctx.borrow();

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
    where
        F: Fn(MouseButton, u64, u64) -> Inhibit,
    {
        let ctx = self.context.clone();
        let drag_position = self.drag_position.clone();

        self.eb.connect_motion_notify_event(move |_, e| {
            let ctx = ctx.borrow();
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
    where
        F: Fn(MouseButton, u64, u64) -> Inhibit,
    {
        let ctx = self.context.clone();

        self.eb.connect_button_press_event(move |_, e| {
            let ctx = ctx.borrow();

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
    where
        F: Fn(MouseButton, u64, u64) -> Inhibit,
    {
        let ctx = self.context.clone();

        self.eb.connect_button_release_event(move |_, e| {
            let ctx = ctx.borrow();

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
    where
        F: Fn(u64, u64) -> bool,
    {
        let ctx = self.context.clone();

        self.da.connect_configure_event(move |da, _| {
            let ctx = ctx.borrow();

            let w = f64::from(da.get_allocated_width());
            let h = f64::from(da.get_allocated_height());
            let cols = (w / ctx.cell_metrics.width).floor() as u64;
            let rows = (h / ctx.cell_metrics.height).floor() as u64;

            f(rows, cols)
        });
    }

    pub fn put_line(&self, line: GridLineSegment, hl_defs: &HlDefs) {
        let mut ctx = self.context.borrow_mut();

        render::put_line(
            &mut ctx,
            &self.da.get_pango_context().unwrap(),
            line,
            hl_defs,
        );
    }

    pub fn redraw(&self, hl_defs: &HlDefs) {
        let mut ctx = self.context.borrow_mut();
        render::redraw(
            &mut ctx,
            &self.da.get_pango_context().unwrap(),
            hl_defs,
        );
    }

    pub fn cursor_goto(&self, row: u64, col: u64) {
        let mut ctx = self.context.borrow_mut();

        // Clear old cursor position.
        let (x, y, w, h) = ctx.get_cursor_rect();
        ctx.queue_draw_area.push((
            f64::from(x),
            f64::from(y),
            f64::from(w),
            f64::from(h),
        ));
        ctx.cursor.0 = row;
        ctx.cursor.1 = col;

        // Mark the new cursor position to be drawn.
        let (x, y, w, h) = ctx.get_cursor_rect();
        ctx.queue_draw_area.push((
            f64::from(x),
            f64::from(y),
            f64::from(w),
            f64::from(h),
        ));

        if let Some(ref im_context) = self.im_context {
            let rect = gdk::Rectangle {
                x: x,
                y: y,
                width: w,
                height: h,
            };
            im_context.set_cursor_location(&rect);
        }
    }

    pub fn get_grid_metrics(&self) -> GridMetrics {
        let ctx = self.context.borrow();

        let row = ctx.rows.get(0).unwrap();

        let rows = ctx.rows.len() as f64;
        let cols = row.len() as f64;
        let cell_width = ctx.cell_metrics.width;
        let cell_height = ctx.cell_metrics.height;

        GridMetrics {
            rows,
            cols,
            cell_width,
            cell_height,
            width: cols * cell_width,
            height: rows * cell_height,
        }
    }

    /// Calcualtes the current size of the grid.
    pub fn calc_size(&self) -> (i64, i64) {
        let ctx = self.context.borrow();

        // TODO(ville): Dont relay on the drawingarea's size.
        let w = self.da.get_allocated_width();
        let h = self.da.get_allocated_height();
        let cols = (w / ctx.cell_metrics.width as i32) as i64;
        let rows = (h / ctx.cell_metrics.height as i32) as i64;

        (cols, rows)
    }

    pub fn resize(
        &self,
        win: &gdk::Window,
        cols: u64,
        rows: u64,
        hl_defs: &HlDefs,
    ) {
        let mut ctx = self.context.borrow_mut();
        ctx.resize(&self.da, win, cols as usize, rows as usize, hl_defs);
    }

    pub fn clear(&self, hl_defs: &HlDefs) {
        let mut ctx = self.context.borrow_mut();

        // Clear internal grid (rows).
        for row in ctx.rows.iter_mut() {
            row.clear();
        }

        render::clear(&self.da, &mut ctx, hl_defs)
    }

    pub fn scroll(
        &self,
        reg: [u64; 4],
        rows: i64,
        _cols: i64,
        hl_defs: &HlDefs,
    ) {
        let mut ctx = self.context.borrow_mut();

        render::scroll(&mut ctx, hl_defs, reg, rows);
    }

    pub fn set_active(&self, active: bool) {
        let mut ctx = self.context.borrow_mut();

        ctx.active = active;
    }

    pub fn tick(&self) {
        let mut ctx = self.context.borrow_mut();

        // If we dont need to blink, return. Otherwise, handle the cursor
        // blining.
        if ctx.cursor_blink_on == 0 {
            return;
        }

        // Assuming a 60hz framerate
        ctx.cursor_alpha += 100.0 / (6.0 * ctx.cursor_blink_on as f64);

        if ctx.cursor_alpha > 2.0 {
            ctx.cursor_alpha = 0.0;
        }

        let (x, y, w, h) = ctx.get_cursor_rect();

        let mut alpha = ctx.cursor_alpha;
        if alpha > 1.0 {
            alpha = 2.0 - alpha;
        }

        let cr = &ctx.cursor_context;
        cr.save();
        cr.rectangle(0.0, 0.0, 100.0, 100.0);
        cr.set_operator(cairo::Operator::Source);
        cr.set_source_rgba(
            ctx.cursor_color.r,
            ctx.cursor_color.g,
            ctx.cursor_color.b,
            alpha,
        );
        cr.fill();
        cr.restore();

        // Don't use the ctx.queue_draw_area, because those draws will only
        // happen once nvim sends 'flush' event. This draw needs to happen
        // on each tick so the cursor blinks.
        self.da.queue_draw_area(x, y, w, h);
    }

    /// Set a new font and line space. This will likely change the cell metrics.
    /// Use `calc_size` to receive the updated size (cols and rows) of the grid.
    pub fn update_cell_metrics(
        &self,
        font: Font,
        line_space: i64,
        win: &gdk::Window,
    ) {
        let mut ctx = self.context.borrow_mut();
        ctx.update_metrics(font, line_space, &self.da, win);
    }

    /// Get the current line space value.
    pub fn get_line_space(&self) -> i64 {
        let ctx = self.context.borrow();
        ctx.cell_metrics.line_space
    }

    /// Get a copy of the current font.
    pub fn get_font(&self) -> Font {
        let ctx = self.context.borrow();
        ctx.cell_metrics.font.clone()
    }

    pub fn set_mode(&self, mode: &ModeInfo) {
        let mut ctx = self.context.borrow_mut();

        ctx.cursor_blink_on = mode.blink_on;
        ctx.cursor_cell_percentage = mode.cell_percentage;
    }

    pub fn set_busy(&self, busy: bool) {
        let mut ctx = self.context.borrow_mut();

        ctx.busy = busy;
    }
}

impl Drop for Grid {
    fn drop(&mut self) {
        // TODO(ville): Test that we release all resources.
        self.eb.destroy();
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
    if !ctx.busy && ctx.active {
        let (x, y, w, h) = ctx.get_cursor_rect();

        cr.save();
        cr.rectangle(
            f64::from(x),
            f64::from(y),
            f64::from(w) * ctx.cursor_cell_percentage,
            f64::from(h),
        );
        let surface = ctx.cursor_context.get_target();
        surface.flush();
        cr.set_source_surface(&surface, x.into(), y.into());
        cr.fill();
        cr.restore();
    }
}
