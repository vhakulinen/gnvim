use glib::Object;
use gtk::{glib, subclass::prelude::*};

use nvim::types::uievents::{GridLine, GridResize};

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

    pub fn put(&self, event: GridLine) {
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
        self.imp().buffer.flush(colors, font);
    }

    pub fn clear(&self) {
        self.imp().buffer.clear();
    }

    pub fn cursor_goto(&self, font: &Font, colors: &Colors, col: i64, row: i64) {
        let imp = self.imp();
        let hl_id = imp
            .buffer
            .get_rows()
            .get(row as usize)
            .expect("invalid row")
            .cells
            .get(col as usize)
            .expect("invalid col")
            .hl_id;

        let fg = colors.get_hl_fg(hl_id);

        imp.cursor.move_to(font, col, row, fg);
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(0)
    }
}
