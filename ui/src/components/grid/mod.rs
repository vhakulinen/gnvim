use std::{cell::RefCell, rc::Rc};

use glib::Object;
use gtk::{gdk, glib, glib::clone, prelude::*, subclass::prelude::*};

use nvim::types::uievents::{GridLine, GridResize, GridScroll, ModeInfo};

use crate::{
    colors::Colors,
    font::Font,
    mouse::{Action, Mouse},
};

mod imp;

glib::wrapper! {
    pub struct Grid(ObjectSubclass<imp::Grid>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Grid {
    pub fn new(id: i64) -> Self {
        Object::new(&[("grid-id", &id)]).expect("Failed to create Grid")
    }

    fn input_modifier(evt: &gtk::EventController) -> String {
        let state = evt.current_event_state();

        let mut modifier = String::new();
        if state.contains(gdk::ModifierType::SHIFT_MASK) {
            modifier.push('S');
        }
        if state.contains(gdk::ModifierType::CONTROL_MASK) {
            modifier.push('C');
        }
        if state.contains(gdk::ModifierType::ALT_MASK) {
            modifier.push('A');
        }

        modifier
    }

    fn input_mouse(gst: &gtk::GestureSingle) -> Mouse {
        match gst.current_button() {
            gdk::BUTTON_PRIMARY => Mouse::Left,
            gdk::BUTTON_SECONDARY => Mouse::Right,
            gdk::BUTTON_MIDDLE => Mouse::Middle,
            _ => {
                println!("unknown button, defaulting to primary");
                Mouse::Left
            }
        }
    }

    pub fn connect_mouse<F>(&self, font: Font, f: F)
    where
        F: Fn(i64, Mouse, Action, String, usize, usize) + 'static + Clone,
    {
        let click = clone!(@weak self as obj, @weak font, @strong f, => move |
            gst: &gtk::GestureClick,
            action: Action,
            n: i32,
            x: f64,
            y: f64,
        | {
            let col = font.scale_to_col(x) as usize;
            let row = font.scale_to_row(y) as usize;

            let modifier = Grid::input_modifier(gst.upcast_ref::<gtk::EventController>());
            let mouse = Grid::input_mouse(gst.upcast_ref::<gtk::GestureSingle>());

            for _ in 0..n {
                f(obj.imp().id.get(), mouse, action, modifier.clone(), row, col)
            }
        });

        let imp = self.imp();
        imp.gesture_click.connect_pressed(
            clone!(@strong click => move |gst, n, x, y| click(gst, Action::Pressed, n, x, y)),
        );
        imp.gesture_click.connect_released(
            clone!(@strong click => move |gst, n, x, y| click(gst, Action::Released, n, x, y)),
        );

        let start = Rc::new(RefCell::new((0.0, 0.0)));
        let pos = Rc::new(RefCell::new((0, 0)));
        imp.gesture_drag
            .connect_drag_begin(clone!(@weak font, @strong start => move |_, x, y| {
                start.replace((x, y));
            }));
        imp.gesture_drag.connect_drag_update(
            clone!(@strong start, @strong pos, @weak self as obj, @weak font, @strong f => move |gst, x, y| {
                let start = start.borrow();
                let x = start.0 + x;
                let y = start.1 + y;

                let mut prev = pos.borrow_mut();
                let col = font.scale_to_col(x);
                let row = font.scale_to_row(y);

                if prev.0 != row || prev.1 != col {
                    *prev = (row, col);

                    let modifier = Grid::input_modifier(gst.upcast_ref::<gtk::EventController>());
                    let mouse = Grid::input_mouse(gst.upcast_ref::<gtk::GestureSingle>());
                    f(obj.imp().id.get(), mouse, Action::Drag, modifier, row, col);
                }
            }),
        );

        let mouse_pos = Rc::new(RefCell::new((0.0, 0.0)));
        imp.event_controller_motion
            .connect_motion(clone!(@strong mouse_pos => move |_, x, y| {
                mouse_pos.replace((x, y));
            }));

        imp.event_controller_scroll.connect_scroll(
            clone!(@weak self as obj, @weak font, @strong mouse_pos => @default-return gtk::Inhibit(false), move |evt, dx, dy| {
                let modifier = Grid::input_modifier(evt.upcast_ref::<gtk::EventController>());
                let pos = mouse_pos.borrow();
                let col = font.scale_to_col(pos.0);
                let row = font.scale_to_row(pos.1);

                let id = obj.imp().id.get();

                if dx > 0.0 {
                    f(id, Mouse::Wheel, Action::ScrollRight, modifier, row, col);
                } else if dx < 0.0 {
                    f(id, Mouse::Wheel, Action::ScrollLeft, modifier, row, col);
                } else if dy > 0.0 {
                    f(id, Mouse::Wheel, Action::ScrollDown, modifier, row, col);
                } else if dy < 0.0 {
                    f(id, Mouse::Wheel, Action::ScrollUp, modifier, row, col);
                }

                gtk::Inhibit(true)
            }),
        );
    }

    pub fn hide_cursor(&self, hide: bool) {
        self.imp().cursor.hide(hide);
    }

    pub fn put(&self, event: GridLine) {
        // TODO(ville): This function should be proxied to the buffer.

        let mut rows = self.imp().buffer.get_rows_mut();
        let row = rows.get_mut(event.row as usize).expect("invalid row");

        row.update(&event);
    }

    pub fn resize(&self, event: GridResize) {
        self.imp()
            .buffer
            .resize(event.width as usize, event.height as usize);
    }

    pub fn flush(&self, colors: &Colors, font: &Font) {
        let imp = self.imp();
        imp.buffer.flush(colors, font);

        // Update the text under the cursor, since in some cases neovim doesn't
        // dispatch cursor goto (e.g. when grid scroll happens but cursor
        // doesn't move).
        let rows = imp.buffer.get_rows();
        let row = rows
            .get(imp.cursor.row() as usize)
            .expect("bad cursor position");
        let cell = row
            .cells
            .get(imp.cursor.col() as usize)
            .expect("bad cursor position");
        imp.cursor.set_text(cell.text.clone());
        imp.cursor.flush(colors, font);
    }

    pub fn clear(&self) {
        self.imp().buffer.clear();
    }

    pub fn cursor_goto(&self, col: i64, row: i64) {
        let imp = self.imp();

        let rows = imp.buffer.get_rows();
        let cells = &rows.get(row as usize).expect("invalid row").cells;
        let cell = cells.get(col as usize).expect("invalid col");

        imp.cursor.move_to(cell, col, row);
    }

    pub fn scroll(&self, event: GridScroll) {
        self.imp().buffer.scroll(event);
    }

    pub fn mode_change(&self, mode: &ModeInfo) {
        let cell_percentage = mode
            .cell_percentage
            // Make sure we have non 0 value.
            .map(|v| if v == 0 { 100 } else { v })
            .map(|v| v as f32 / 100.0)
            .unwrap_or(100.0);

        let imp = self.imp();
        imp.cursor.set_width_percentage(cell_percentage);
        imp.cursor.set_attr_id(mode.attr_id.unwrap_or(0) as i64);

        // TODO(ville): Handle rest of the mode properties (blink, cursor shape).
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(0)
    }
}
