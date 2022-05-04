use glib::Object;
use gtk::{glib, subclass::prelude::*};

use nvim::types::uievents::{GridLine, GridResize, GridScroll, ModeInfo};

use crate::{colors::Colors, font::Font};

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
