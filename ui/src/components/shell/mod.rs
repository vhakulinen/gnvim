use gtk::{glib, subclass::prelude::*};
use nvim::types::uievents::{GridClear, GridCursorGoto, GridLine, GridResize};

use crate::{colors::Colors, font::Font};

mod imp;

glib::wrapper! {
    pub struct Shell(ObjectSubclass<imp::Shell>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Shell {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Shell")
    }

    pub fn handle_grid_line(&self, event: GridLine) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp().root_grid.put(event);
    }

    pub fn handle_grid_resize(&self, event: GridResize) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp().root_grid.resize(event);
    }

    pub fn handle_flush(&self, colors: &Colors, font: &Font) {
        self.imp().root_grid.flush(colors, font);
    }

    pub fn handle_grid_clear(&self, event: GridClear) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp().root_grid.clear();
    }

    pub fn handle_grid_cursor_goto(&self, event: GridCursorGoto, font: &Font, colors: &Colors) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp()
            .root_grid
            .cursor_goto(font, colors, event.col, event.row);
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
