use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};
use nvim::types::uievents::{
    GridClear, GridCursorGoto, GridDestroy, GridLine, GridResize, GridScroll, MsgSetPos,
    PopupmenuSelect, PopupmenuShow, WinClose, WinExternalPos, WinFloatPos, WinHide, WinPos,
};

use crate::{colors::Colors, font::Font, mode_info::ModeInfo, warn, SCALE};

use super::Grid;

macro_rules! find_grid_or_return {
    ($self:expr, $grid:expr) => {
        crate::some_or_return!(
            $self.find_grid($grid),
            "grid {} not found in {}:{}",
            $grid,
            file!(),
            line!()
        )
    };
}

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

    pub fn set_cursor_blink_transition(&self, t: f64) {
        self.set_property("cursor-blink-transition", t);
    }

    pub fn set_cursor_position_transition(&self, t: f64) {
        self.set_property("cursor-position-transition", t);
    }

    pub fn set_scroll_transition(&self, t: f64) {
        self.set_property("scroll-transition", t);
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
        find_grid_or_return!(self, event.grid).put(event);
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
                self.bind_property("current-mode-info", &grid, "mode-info")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                self.bind_property("cursor-blink-transition", &grid, "cursor-blink-transition")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                self.bind_property(
                    "cursor-position-transition",
                    &grid,
                    "cursor-position-transition",
                )
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
                self.bind_property("scroll-transition", &grid, "scroll-transition")
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
        find_grid_or_return!(self, event.grid).clear();
    }

    pub fn handle_grid_cursor_goto(&self, event: GridCursorGoto) {
        let mut current_grid = self.imp().current_grid.borrow_mut();
        current_grid.set_active(false);

        // NOTE(ville): In some situations, neovim sends `grid_cursor_goto`
        // message for a grid that already got destroyed.
        if let Some(grid) = self.find_grid(event.grid) {
            grid.cursor_goto(event.col, event.row);
            grid.set_active(true);

            *current_grid = grid;
        } else {
            println!("invalid grid for grid_cursor_goto: {}", event.grid);
        }
    }

    pub fn handle_grid_scroll(&self, event: GridScroll) {
        find_grid_or_return!(self, event.grid).scroll(event);
    }

    pub fn handle_mode_change(&self, mode: &ModeInfo) {
        self.set_property("current-mode-info", mode);
    }

    pub fn handle_grid_destroy(&self, event: GridDestroy) {
        assert!(event.grid != 1, "cant do grid_destroy for grid 1");

        let mut grids = self.imp().grids.borrow_mut();
        if let Some(index) = grids.iter().position(|grid| grid.id() == event.grid) {
            // Remove the grid from our list, and unparent it. This will cause
            // it to be dropped because all the references to the grid will be
            // released.
            let grid = grids.remove(index);
            grid.unparent();
        } else {
            warn!("grid {} not found in {}:{}", event.grid, file!(), line!());
        }
    }

    pub fn handle_win_pos(&self, event: WinPos, font: &Font) {
        assert!(event.grid != 1, "cant do win_pos for grid 1");

        /* NOTE(ville): The reported width and height in this event _might_
         * be different from the actual size of the window when/if at somepoint
         * neovim is not controlling the window's/grid's size.
         */

        let grid = find_grid_or_return!(self, event.grid);
        grid.set_nvim_window(Some(event.win));

        let x = font.col_to_x(event.startcol as f64) as f32;
        let y = font.row_to_y(event.startrow as f64) as f32;

        let fixed = self.imp().fixed.clone();
        if grid.parent().map(|parent| parent == fixed).unwrap_or(false) {
            fixed.move_(&grid, x, y);
        } else {
            grid.unparent();
            fixed.put(&grid, x, y);
        }
    }

    pub fn handle_float_pos(&self, event: WinFloatPos, font: &Font) {
        let grid = find_grid_or_return!(self, event.grid);
        grid.set_nvim_window(Some(event.win));

        let east = event.anchor == "NE" || event.anchor == "SE";
        let south = event.anchor == "SE" || event.anchor == "SW";

        // Adjust position based on anchor.
        let (cols, rows) = grid.grid_size();
        let col = event.anchor_col - if east { cols as f64 } else { 0.0 };
        let row = event.anchor_row - if south { rows as f64 } else { 0.0 };

        let fixed = self.imp().fixed.clone();

        let pos = if event.anchor_grid == 1 {
            gsk::Transform::new()
        } else {
            fixed.child_position(&find_grid_or_return!(self, event.anchor_grid))
        }
        .transform_point(&graphene::Point::new(
            font.col_to_x(col) as f32,
            font.row_to_y(row) as f32,
        ));

        let (_, req) = self.imp().root_grid.preferred_size();
        let (max_x, max_y) = (req.width(), req.height());

        let (req, _) = grid.preferred_size();
        // TODO(ville): Resize the grid if it doesn't fit the screen?
        let x = pos.x().min((max_x - req.width()) as f32).max(0.0);
        let y = pos
            .y()
            // NOTE(ville): Not 100% the substraction of one cell height is
            // required.
            .min((max_y - req.height()) as f32 - font.height() / SCALE)
            .max(0.0);

        if grid.parent().map(|parent| parent == fixed).unwrap_or(false) {
            fixed.move_(&grid, x, y);
        } else {
            grid.unparent();
            fixed.put(&grid, x, y);
        }

        fixed.set_zindex(&grid, event.zindex);
    }

    pub fn handle_win_hide(&self, event: WinHide) {
        assert!(event.grid != 1, "cant do win_hide for grid 1");

        let grid = find_grid_or_return!(self, event.grid);
        grid.unparent();
    }

    pub fn handle_win_close(&self, event: WinClose) {
        assert!(event.grid != 1, "cant do win_close for grid 1");

        let grid = find_grid_or_return!(self, event.grid);
        grid.set_nvim_window(None);
        grid.unparent();
    }

    pub fn handle_win_external_pos(&self, event: WinExternalPos, parent: &gtk::Window) {
        assert!(event.grid != 1, "cant do win_external_pos for grid 1");

        let grid = find_grid_or_return!(self, event.grid);
        grid.set_nvim_window(Some(event.win));
        grid.make_external(parent);
    }

    pub fn handle_msg_set_pos(&self, event: MsgSetPos, font: &Font) {
        assert!(event.grid != 1, "cant do msg_set_pos for grid 1");

        let grid = find_grid_or_return!(self, event.grid);
        let imp = self.imp();
        let win = imp.msg_win.clone();

        let h = imp.root_grid.grid_size().1 - event.row as usize;
        win.set_height(font.row_to_y(h as f64).ceil() as i32);

        let y = font.row_to_y(event.row as f64);
        imp.fixed.move_(&win, 0.0, y as f32);

        if grid.parent().map(|parent| parent != win).unwrap_or(true) {
            grid.unparent();
            grid.set_parent(&win);
        }

        if event.scrolled {
            win.add_css_class("scrolled");
        } else {
            win.remove_css_class("scrolled");
        }
    }

    pub fn handle_popupmenu_show(&self, event: PopupmenuShow) {
        let imp = self.imp();

        // NOTE(ville): Need to set the content and visibility before checking
        // the size.
        imp.popupmenu.set_items(event.items);
        imp.popupmenu.set_visible(true);

        // TODO(ville): Would be nice to make the popupmenu to retain its
        // placement (e.g. above vs. below) when the popupmenu is already
        // shown and displayed in a way where it has enough space.

        let font = imp.font.borrow();

        let pos = if event.grid == 1 {
            gsk::Transform::new()
        } else {
            imp.fixed
                .child_position(&find_grid_or_return!(self, event.grid))
        }
        .transform_point(&graphene::Point::new(
            font.col_to_x(event.col as f64) as f32,
            font.row_to_y(event.row as f64 + 1.0) as f32,
        ));

        let (_, req) = imp.root_grid.preferred_size();
        let max_w = req.width() as f32;
        // Make sure the msg window and the popupmenu won't overlap.
        let max_h = (req.height() - imp.msg_win.height()) as f32;
        let (x, y) = (pos.x(), pos.y());
        let below = max_h - y;
        let above = max_h - below - font.height() / SCALE;

        let (_, req) = imp.popupmenu.listview_preferred_size();
        let (pmenu_w, pmenu_h) = (req.width() as f32, req.height() as f32);

        let (y, max_h) = if pmenu_h > below && above > below {
            // Place above.
            ((y - font.height() / SCALE - pmenu_h).max(0.0), above)
        } else {
            // Place below.
            (y, below)
        };

        let x = if x + pmenu_w > max_w {
            (max_w - pmenu_w).max(0.0)
        } else {
            // Adjust for padding when not overflowing to the right.
            x - imp.popupmenu.get_padding_x()
        };

        imp.popupmenu.set_max_width(max_w.floor() as i32);
        imp.popupmenu.set_max_height(max_h.floor() as i32);
        imp.fixed.move_(&*imp.popupmenu, x, y);

        imp.popupmenu.report_pum_bounds(&imp.nvim.borrow(), x, y);
    }

    pub fn handle_popupmenu_select(&self, event: PopupmenuSelect) {
        self.imp().popupmenu.select(event.selected as u32);
    }

    pub fn handle_popupmenu_hide(&self) {
        let imp = self.imp();
        imp.popupmenu.set_visible(false);
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
