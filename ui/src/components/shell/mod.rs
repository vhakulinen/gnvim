use std::rc::Rc;

use futures::lock::Mutex;
use gio_compat::CompatWrite;
use gtk::{glib, glib::clone, subclass::prelude::*};
use nvim::types::uievents::{
    GridClear, GridCursorGoto, GridLine, GridResize, GridScroll, ModeInfo,
};

use crate::{colors::Colors, font::Font, nvim_unlock, spawn_local};

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

    pub fn connect_root_grid(
        &self,
        font: Font,
        nvim: Rc<Mutex<Option<nvim::Client<CompatWrite>>>>,
    ) {
        self.imp().root_grid.connect_mouse(
            font,
            clone!(@weak nvim => move |id, mouse, action, modifier, row, col| {
                spawn_local!(async move {
                    let res = nvim_unlock!(nvim)
                        .nvim_input_mouse(
                            mouse.as_nvim_input().to_owned(),
                            action.as_nvim_action().to_owned(),
                            modifier,
                            id,
                            row as i64,
                            col as i64,
                        )
                        .await.expect("call to nvim failed");

                    res.await.expect("nvim_input_mouse failed");
                });
            }),
        )
    }

    pub fn busy_start(&self) {
        self.imp().root_grid.hide_cursor(true);
    }

    pub fn busy_stop(&self) {
        self.imp().root_grid.hide_cursor(false);
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

    pub fn handle_grid_cursor_goto(&self, event: GridCursorGoto) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp().root_grid.cursor_goto(event.col, event.row);
    }

    pub fn handle_grid_scroll(&self, event: GridScroll) {
        assert_eq!(
            event.grid, 1,
            "without ext_multigrid, all events should be on grid 1"
        );

        self.imp().root_grid.scroll(event);
    }

    pub fn handle_mode_change(&self, mode: &ModeInfo) {
        self.imp().root_grid.mode_change(mode);
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
