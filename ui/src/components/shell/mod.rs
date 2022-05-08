use std::rc::Rc;

use futures::lock::Mutex;
use gio_compat::CompatWrite;
use gtk::{glib, glib::clone, prelude::*, subclass::prelude::*};
use nvim::types::{
    uievents::{
        GridClear, GridCursorGoto, GridDestroy, GridLine, GridResize, GridScroll, WinClose,
        WinFloatPos, WinHide, WinPos,
    },
    ModeInfo,
};

use crate::{colors::Colors, font::Font, nvim_unlock, spawn_local};

use super::Grid;

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

    fn find_grid(&self, id: i64) -> Option<Grid> {
        self.imp()
            .grids
            .borrow()
            .iter()
            .find(|grid| grid.id() == id)
            .cloned()
    }

    fn find_grid_must(&self, id: i64) -> Grid {
        self.find_grid(id).expect(&format!("grid {} not found", id))
    }

    pub fn busy_start(&self) {
        self.imp().root_grid.hide_cursor(true);
    }

    pub fn busy_stop(&self) {
        self.imp().root_grid.hide_cursor(false);
    }

    pub fn handle_grid_line(&self, event: GridLine) {
        self.find_grid_must(event.grid).put(event);
    }

    pub fn handle_grid_resize(&self, event: GridResize) {
        self.find_grid(event.grid)
            .unwrap_or_else(|| {
                let grid = Grid::new(event.grid);
                self.imp().grids.borrow_mut().push(grid.clone());
                grid
            })
            .resize(event);
    }

    pub fn handle_flush(&self, colors: &Colors, font: &Font) {
        self.imp()
            .grids
            .borrow()
            .iter()
            .for_each(|grid| grid.flush(colors, font))
    }

    pub fn handle_grid_clear(&self, event: GridClear) {
        self.find_grid_must(event.grid).clear();
    }

    pub fn handle_grid_cursor_goto(&self, event: GridCursorGoto) {
        self.find_grid_must(event.grid)
            .cursor_goto(event.col, event.row);
    }

    pub fn handle_grid_scroll(&self, event: GridScroll) {
        self.find_grid_must(event.grid).scroll(event);
    }

    pub fn handle_mode_change(&self, mode: &ModeInfo) {
        self.imp()
            .grids
            .borrow()
            .iter()
            .for_each(|grid| grid.mode_change(mode))
    }

    pub fn handle_grid_destroy(&self, event: GridDestroy) {
        assert!(event.grid != 1, "cant do grid_destroy for grid 1");

        let mut grids = self.imp().grids.borrow_mut();
        let index = grids
            .iter()
            .position(|grid| grid.id() == event.grid)
            .expect("grid_destroy: bad grid id");

        // Remove the grid from our list, and unparent it. This will cause
        // it to be dropped because all the references to the grid will be
        // released.
        let grid = grids.remove(index);
        grid.unparent();
    }

    pub fn handle_win_pos(&self, event: WinPos, font: &Font) {
        assert!(event.grid != 1, "cant do win_pos for grid 1");

        /* NOTE(ville): The reported width and height in this event _might_
         * be different from the actual size of the window when/if at somepoint
         * neovim is not controlling the window's/grid's size.
         */

        // Mask the win_pos event as win_float_pos, since it does the same trick.
        let synthetic_float_pos = WinFloatPos {
            grid: event.grid,
            win: event.win,
            anchor: String::new(),
            anchor_grid: 1,
            anchor_col: event.startcol as f64,
            anchor_row: event.startrow as f64,
            focusable: true,
            zindex: 0,
        };

        self.handle_float_pos(synthetic_float_pos, font);
    }

    pub fn handle_float_pos(&self, event: WinFloatPos, font: &Font) {
        let grid = self.find_grid_must(event.grid);
        grid.set_nvim_window(Some(event.win));

        let anchor_grid = self.find_grid_must(event.anchor_grid);
        let x = font.col_to_x(event.anchor_col);
        let y = font.row_to_y(event.anchor_row);

        // TODO(ville): Implement layout that support the zindex.
        let fixed = anchor_grid.fixed().clone();

        // TODO(ville): Adjust x and y based on event.anchor. For this we need
        // to implement the "measure" virtual method for the grid (in order
        // to get its actual size).
        if grid.parent().map(|parent| parent == fixed).unwrap_or(false) {
            fixed.move_(&grid, x, y);
        } else {
            grid.unparent();
            fixed.put(&grid, x, y);
        }
    }

    pub fn handle_win_hide(&self, event: WinHide) {
        assert!(event.grid != 1, "cant do win_hide for grid 1");

        let grid = self.find_grid_must(event.grid);
        grid.unparent();
    }

    pub fn handle_win_close(&self, event: WinClose) {
        assert!(event.grid != 1, "cant do win_close for grid 1");

        let grid = self.find_grid_must(event.grid);
        grid.set_nvim_window(None);
        grid.unparent();
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
