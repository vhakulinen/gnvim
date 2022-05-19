use gtk::{glib, prelude::*, subclass::prelude::*};
use nvim::types::{
    uievents::{
        GridClear, GridCursorGoto, GridDestroy, GridLine, GridResize, GridScroll, MsgSetPos,
        WinClose, WinExternalPos, WinFloatPos, WinHide, WinPos,
    },
    ModeInfo,
};

use crate::{colors::Colors, font::Font};

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

    fn set_busy(&self, busy: bool) {
        self.set_property("busy", busy);
    }

    pub fn busy_start(&self) {
        self.set_busy(true);
    }

    pub fn busy_stop(&self) {
        self.set_busy(false);
    }

    pub fn handle_grid_line(&self, event: GridLine) {
        self.find_grid_must(event.grid).put(event);
    }

    pub fn font(&self) -> Font {
        self.imp().root_grid.font().clone()
    }

    pub fn handle_grid_resize(&self, event: GridResize) {
        self.find_grid(event.grid)
            .unwrap_or_else(|| {
                let grid = Grid::new(event.grid, &self.font());

                // Bind the properties.
                self.bind_property("font", &grid, "font")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                self.bind_property("nvim", &grid, "nvim")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                self.bind_property("busy", &grid, "busy")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();

                self.imp().grids.borrow_mut().push(grid.clone());
                grid
            })
            .resize(event);
    }

    pub fn handle_flush(&self, colors: &Colors) {
        self.imp()
            .grids
            .borrow()
            .iter()
            .for_each(|grid| grid.flush(colors));
    }

    pub fn handle_grid_clear(&self, event: GridClear) {
        self.find_grid_must(event.grid).clear();
    }

    pub fn handle_grid_cursor_goto(&self, event: GridCursorGoto) {
        let mut current_grid = self.imp().current_grid.borrow_mut();
        current_grid.set_active(false);

        let grid = self.find_grid_must(event.grid);
        grid.cursor_goto(event.col, event.row);
        grid.set_active(true);

        *current_grid = grid;
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

        let grid = self.find_grid_must(event.grid);
        grid.set_nvim_window(Some(event.win));

        let x = font.col_to_x(event.startcol as f64);
        let y = font.row_to_y(event.startrow as f64);

        let fixed = self.imp().root_grid.fixed().clone();
        if grid.parent().map(|parent| parent == fixed).unwrap_or(false) {
            fixed.move_(&grid, x, y);
        } else {
            grid.unparent();
            fixed.put(&grid, x, y);
        }
    }

    pub fn handle_float_pos(&self, event: WinFloatPos, font: &Font) {
        let grid = self.find_grid_must(event.grid);
        grid.set_nvim_window(Some(event.win));

        let anchor_grid = self.find_grid_must(event.anchor_grid);

        let east = event.anchor == "NE" || event.anchor == "SE";
        let south = event.anchor == "SE" || event.anchor == "SW";

        // Adjust position based on anchor.
        let (cols, rows) = grid.grid_size();
        let col = event.anchor_col - if east { cols as f64 } else { 0.0 };
        let row = event.anchor_row - if south { rows as f64 } else { 0.0 };

        // Adjust position if the floating grid overflows.
        //
        // NOTE(ville): It _seems_ like neovim clamps the floating window's
        // size to the size of the anchor grid, but doesn't adjust position
        // accordingly.
        // TODO(ville): The current solution doesn't allow the floating grid
        // overflow on anyway even if there would be space, e.g. when a floating
        // window in middle of the screen overflows the anchor grid.
        let (max_cols, max_rows) = anchor_grid.grid_size();
        let col = col.min(col.min((max_cols - cols) as f64)).max(0.0);
        let row = row.min(row.min((max_rows - rows + 1) as f64)).max(0.0);

        let x = font.col_to_x(col);
        let y = font.row_to_y(row);

        // TODO(ville): Implement layout that support the zindex.
        let fixed = anchor_grid.fixed().clone();

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

    pub fn handle_win_external_pos(&self, event: WinExternalPos, parent: &gtk::Window) {
        assert!(event.grid != 1, "cant do win_external_pos for grid 1");

        let grid = self.find_grid_must(event.grid);
        grid.set_nvim_window(Some(event.win));
        grid.make_external(parent);
    }

    pub fn handle_msg_set_pos(&self, event: MsgSetPos, font: &Font) {
        assert!(event.grid != 1, "cant do msg_set_pos for grid 1");

        let grid = self.find_grid_must(event.grid);
        let fixed = self.imp().msg_fixed.clone();

        let x = 0.0;
        let y = font.row_to_y(event.row as f64);

        if grid.parent().map(|parent| parent == fixed).unwrap_or(false) {
            fixed.move_(&grid, x, y);
        } else {
            grid.unparent();
            fixed.put(&grid, x, y);
        }

        // TODO(ville): Draw the separator.
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
