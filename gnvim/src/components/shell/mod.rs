mod imp;

use gtk::{glib, subclass::prelude::*};
use nvim::types::uievents::{GridLine, GridResize};

use crate::colors::Colors;

glib::wrapper! {
    pub struct Shell(ObjectSubclass<imp::Shell>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Shell {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Window")
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

    pub fn handle_flush(&self, colors: &Colors) {
        self.imp().root_grid.flush(colors);
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
