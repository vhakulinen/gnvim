use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glib;
use gtk;
use gtk::prelude::*;

use log::{debug, error};
use nvim_rs::Tabpage;

use crate::nvim_bridge::{
    CmdlineBlockAppend, CmdlineBlockShow, CmdlinePos, CmdlineShow,
    CmdlineSpecialChar, DefaultColorsSet, GnvimEvent, GridCursorGoto,
    GridLineSegment, GridResize, GridScroll, HlAttrDefine, HlGroupSet,
    ModeChange, ModeInfo, ModeInfoSet, Notify, OptionSet, PopupmenuShow,
    RedrawEvent, TablineUpdate, WildmenuShow,
};
use crate::nvim_gio::GioNeovim;
use crate::ui::cmdline::Cmdline;
use crate::ui::color::{HlDefs, HlGroup};
use crate::ui::common::spawn_local;
#[cfg(feature = "libwebkit2gtk")]
use crate::ui::cursor_tooltip::{CursorTooltip, Gravity};
use crate::ui::font::Font;
use crate::ui::grid::Grid;
use crate::ui::popupmenu::Popupmenu;
use crate::ui::tabline::Tabline;

pub(crate) type Grids = HashMap<u64, Grid>;

pub(crate) struct ResizeOptions {
    pub font: Font,
    pub line_space: i64,
}

/// Internal structure for `UI` to work on.
pub(crate) struct UIState {
    /// All grids currently in the UI.
    pub grids: Grids,
    /// Highlight definitions.
    pub hl_defs: HlDefs,
    /// Mode infos. When a mode is activated, the activated mode is passed
    /// to the gird(s).
    pub mode_infos: Vec<ModeInfo>,
    /// Id of the current active grid.
    pub current_grid: u64,

    pub popupmenu: Popupmenu,
    pub cmdline: Cmdline,
    pub tabline: Tabline,
    #[cfg(feature = "libwebkit2gtk")]
    pub cursor_tooltip: CursorTooltip,

    /// Overlay contains our grid(s) and popupmenu.
    #[allow(unused)]
    pub overlay: gtk::Overlay,

    /// Source id for delayed call to ui_try_resize.
    pub resize_source_id: Rc<RefCell<Option<glib::SourceId>>>,
    /// Resize options that is some if a resize should be send to nvim on flush.
    pub resize_on_flush: Option<ResizeOptions>,

    /// Flag for flush to update GUI colors on components that depend on
    /// hl gruops.
    pub hl_groups_changed: bool,
}

impl UIState {
    pub fn handle_notify(
        &mut self,
        window: &gtk::ApplicationWindow,
        notify: Notify,
        nvim: &GioNeovim,
    ) {
        match notify {
            Notify::RedrawEvent(events) => {
                events.into_iter().for_each(|e| {
                    self.handle_redraw_event(window, e, &nvim);
                });
            }
            Notify::GnvimEvent(event) => match event {
                Ok(event) => self.handle_gnvim_event(&event, nvim),
                Err(err) => {
                    let nvim = nvim.clone();
                    let msg = format!(
                        "echom \"Failed to parse gnvim notify: '{}'\"",
                        err
                    );
                    spawn_local(async move {
                        if let Err(err) = nvim.command(&msg).await {
                            error!("Failed to execute nvim command: {}", err)
                        }
                    });
                }
            },
        }
    }

    fn set_title(&mut self, window: &gtk::ApplicationWindow, title: &str) {
        window.set_title(title);
    }

    fn grid_cursor_goto(
        &mut self,
        GridCursorGoto {
            grid: grid_id,
            row,
            col,
        }: GridCursorGoto,
    ) {
        // Gird cursor goto sets the current cursor to grid_id,
        // so we'll need to handle that here...
        let grid = if grid_id != self.current_grid {
            // ...so if the grid_id is not same as the self tells us,
            // set the previous current grid to inactive self.
            self.grids
                .get(&self.current_grid)
                .unwrap()
                .set_active(false);
            self.current_grid = grid_id;

            // And set the new current grid to active.
            let grid = self.grids.get(&grid_id).unwrap();
            grid.set_active(true);
            grid
        } else {
            self.grids.get(&grid_id).unwrap()
        };

        // And after all that, set the current grid's cursor position.
        grid.cursor_goto(row, col);
    }

    fn grid_resize(&mut self, e: GridResize) {
        let grid = self.grids.get(&e.grid).unwrap();
        grid.resize(e.width, e.height);
    }

    fn grid_line(&mut self, line: GridLineSegment) {
        let grid = self.grids.get(&line.grid).unwrap();
        grid.put_line(line, &self.hl_defs);
    }

    fn grid_clear(&mut self, grid: &u64) {
        let grid = self.grids.get(grid).unwrap();
        grid.clear(&self.hl_defs);
    }

    fn grid_scroll(&mut self, info: GridScroll, nvim: &GioNeovim) {
        let grid = self.grids.get(&info.grid).unwrap();
        grid.scroll(info.reg, info.rows, info.cols, &self.hl_defs);

        // Since nvim doesn't have its own 'scroll' autocmd, we'll
        // have to do it on our own. This use useful for the cursor tooltip.
        let nvim = nvim.clone();
        spawn_local(async move {
            if let Err(err) = nvim.command("if exists('#User#GnvimScroll') | doautocmd User GnvimScroll | endif").await {
                error!("GnvimScroll error: {:?}", err);
            }
        });
    }

    fn default_colors_set(
        &mut self,
        DefaultColorsSet { fg, bg, sp }: DefaultColorsSet,
    ) {
        self.hl_defs.default_fg = fg;
        self.hl_defs.default_bg = bg;
        self.hl_defs.default_sp = sp;

        {
            // NOTE(ville): Not sure if these are actually needed.
            let hl = self.hl_defs.get_mut(&0).unwrap();
            hl.foreground = Some(fg);
            hl.background = Some(bg);
            hl.special = Some(sp);
        }

        for grid in self.grids.values() {
            grid.redraw(&self.hl_defs);
        }

        #[cfg(feature = "libwebkit2gtk")]
        self.cursor_tooltip.set_colors(fg, bg);
    }

    fn hl_attr_define(&mut self, HlAttrDefine { id, hl }: HlAttrDefine) {
        self.hl_defs.insert(id, hl);
    }

    fn hl_group_set(&mut self, evt: HlGroupSet) {
        match evt.name.as_str() {
            "Pmenu" => {
                self.hl_defs.set_hl_group(HlGroup::Pmenu, evt.hl_id);
                self.hl_defs.set_hl_group(HlGroup::Wildmenu, evt.hl_id)
            }
            "PmenuSel" => {
                self.hl_defs.set_hl_group(HlGroup::PmenuSel, evt.hl_id);
                self.hl_defs.set_hl_group(HlGroup::WildmenuSel, evt.hl_id)
            }
            "TabLine" => self.hl_defs.set_hl_group(HlGroup::Tabline, evt.hl_id),
            "TabLineSel" => {
                self.hl_defs.set_hl_group(HlGroup::TablineSel, evt.hl_id);
                self.hl_defs.set_hl_group(HlGroup::CmdlineBorder, evt.hl_id)
            }
            "TabLineFill" => {
                self.hl_defs.set_hl_group(HlGroup::TablineFill, evt.hl_id)
            }
            "Normal" => self.hl_defs.set_hl_group(HlGroup::Cmdline, evt.hl_id),
            _ => None,
        };

        self.hl_groups_changed = true;
    }

    fn option_set(&mut self, opt: OptionSet) {
        match opt {
            OptionSet::GuiFont(font) => {
                let font = Font::from_guifont(&font).unwrap_or(Font::default());

                let mut opts =
                    self.resize_on_flush.take().unwrap_or_else(|| {
                        let grid = self.grids.get(&1).unwrap();
                        ResizeOptions {
                            font: grid.get_font(),
                            line_space: grid.get_line_space(),
                        }
                    });

                opts.font = font;

                self.resize_on_flush = Some(opts);
            }
            OptionSet::LineSpace(val) => {
                let mut opts =
                    self.resize_on_flush.take().unwrap_or_else(|| {
                        let grid = self.grids.get(&1).unwrap();
                        ResizeOptions {
                            font: grid.get_font(),
                            line_space: grid.get_line_space(),
                        }
                    });

                opts.line_space = val;

                self.resize_on_flush = Some(opts);
            }
            OptionSet::NotSupported(name) => {
                debug!("Not supported option set: {}", name);
            }
        }
    }

    fn mode_info_set(&mut self, ModeInfoSet { mode_info, .. }: ModeInfoSet) {
        self.mode_infos = mode_info.clone();
    }

    fn mode_change(&mut self, ModeChange { index, .. }: ModeChange) {
        let mode = self.mode_infos.get(index as usize).unwrap();
        // Broadcast the mode change to all grids.
        // TODO(ville): It might be enough to just set the mode to the
        //              current active grid.
        for grid in self.grids.values() {
            grid.set_mode(mode);
        }
    }

    fn set_busy(&mut self, busy: bool) {
        for grid in self.grids.values() {
            grid.set_busy(busy);
        }
    }

    fn flush(&mut self, nvim: &GioNeovim) {
        for grid in self.grids.values() {
            grid.flush(&self.hl_defs);
        }

        if let Some(opts) = self.resize_on_flush.take() {
            for grid in self.grids.values() {
                grid.update_cell_metrics(opts.font.clone(), opts.line_space);
            }

            let grid = self.grids.get(&1).unwrap();
            let (cols, rows) = grid.calc_size();

            // Cancel any possible delayed call for ui_try_resize.
            let mut id = self.resize_source_id.borrow_mut();
            if let Some(id) = id.take() {
                glib::source::source_remove(id);
            }

            let nvim = nvim.clone();
            spawn_local(async move {
                if let Err(err) =
                    nvim.ui_try_resize(cols as i64, rows as i64).await
                {
                    error!("Error: failed to resize nvim on line space change ({:?})", err);
                }
            });

            self.popupmenu.set_font(opts.font.clone(), &self.hl_defs);
            self.cmdline.set_font(opts.font.clone(), &self.hl_defs);
            self.tabline.set_font(opts.font.clone(), &self.hl_defs);
            #[cfg(feature = "libwebkit2gtk")]
            self.cursor_tooltip.set_font(opts.font.clone());

            self.cmdline.set_line_space(opts.line_space);
            self.popupmenu
                .set_line_space(opts.line_space, &self.hl_defs);
            self.tabline.set_line_space(opts.line_space, &self.hl_defs);
        }

        if self.hl_groups_changed {
            self.popupmenu.set_colors(&self.hl_defs);
            self.tabline.set_colors(&self.hl_defs);
            self.cmdline.set_colors(&self.hl_defs);
            self.cmdline.wildmenu_set_colors(&self.hl_defs);

            self.hl_groups_changed = false;
        }
    }

    fn popupmenu_show(&mut self, popupmenu: PopupmenuShow) {
        self.popupmenu.set_items(popupmenu.items, &self.hl_defs);

        let grid = self.grids.get(&self.current_grid).unwrap();
        let rect = grid.get_rect_for_cell(popupmenu.row, popupmenu.col);

        self.popupmenu.set_anchor(rect);
        self.popupmenu
            .select(popupmenu.selected as i32, &self.hl_defs);

        self.popupmenu.show();

        // If the cursor tooltip is visible at the same time, move
        // it out of our way.
        #[cfg(feature = "libwebkit2gtk")]
        {
            if self.cursor_tooltip.is_visible() {
                if self.popupmenu.is_above_anchor() {
                    self.cursor_tooltip.force_gravity(Some(Gravity::Down));
                } else {
                    self.cursor_tooltip.force_gravity(Some(Gravity::Up));
                }

                self.cursor_tooltip.refresh_position();
            }
        }
    }

    fn popupmenu_hide(&mut self) {
        self.popupmenu.hide();

        // Undo any force positioning of cursor tool tip that might
        // have occured on popupmenu show.
        #[cfg(feature = "libwebkit2gtk")]
        {
            self.cursor_tooltip.force_gravity(None);
            self.cursor_tooltip.refresh_position();
        }
    }

    fn popupmenu_select(&mut self, selected: i64) {
        self.popupmenu.select(selected as i32, &self.hl_defs);
    }

    fn tabline_update(
        &mut self,
        TablineUpdate { current, tabs }: TablineUpdate,
        nvim: &GioNeovim,
    ) {
        let current = Tabpage::new(current, nvim.clone());
        let tabs = tabs
            .into_iter()
            .map(|(value, name)| (Tabpage::new(value, nvim.clone()), name))
            .collect();
        self.tabline.update(current, tabs);
    }

    fn cmdline_show(&mut self, cmdline_show: CmdlineShow) {
        self.cmdline.show(cmdline_show, &self.hl_defs);
    }

    fn cmdline_hide(&mut self) {
        self.cmdline.hide();
    }

    fn cmdline_pos(&mut self, CmdlinePos { pos, level }: CmdlinePos) {
        self.cmdline.set_pos(pos, level);
    }

    fn cmdline_special_char(&mut self, s: CmdlineSpecialChar) {
        self.cmdline
            .show_special_char(s.character, s.shift, s.level);
    }

    fn cmdline_block_show(&mut self, show: CmdlineBlockShow) {
        self.cmdline.show_block(&show, &self.hl_defs);
    }

    fn cmdline_block_append(&mut self, line: CmdlineBlockAppend) {
        self.cmdline.block_append(line, &self.hl_defs);
    }

    fn cmdline_block_hide(&mut self) {
        self.cmdline.hide_block();
    }

    fn wildmenu_show(&mut self, items: WildmenuShow) {
        self.cmdline.wildmenu_show(&items.0);
    }

    fn wildmenu_hide(&mut self) {
        self.cmdline.wildmenu_hide();
    }

    fn wildmenu_select(&mut self, item: i64) {
        self.cmdline.wildmenu_select(item);
    }

    fn handle_redraw_event(
        &mut self,
        window: &gtk::ApplicationWindow,
        event: RedrawEvent,
        nvim: &GioNeovim,
    ) {
        match event {
            RedrawEvent::SetTitle(evt) => {
                evt.iter().for_each(|e| self.set_title(&window, e));
            }
            RedrawEvent::GridLine(evt) => {
                evt.into_iter().for_each(|line| self.grid_line(line))
            }
            RedrawEvent::GridCursorGoto(evt) => {
                evt.into_iter().for_each(|e| self.grid_cursor_goto(e))
            }
            RedrawEvent::GridResize(evt) => {
                evt.into_iter().for_each(|e| self.grid_resize(e))
            }
            RedrawEvent::GridClear(evt) => {
                evt.iter().for_each(|e| self.grid_clear(e))
            }
            RedrawEvent::GridScroll(evt) => {
                evt.into_iter().for_each(|e| self.grid_scroll(e, nvim))
            }
            RedrawEvent::DefaultColorsSet(evt) => {
                evt.into_iter().for_each(|e| self.default_colors_set(e))
            }
            RedrawEvent::HlAttrDefine(evt) => {
                evt.into_iter().for_each(|e| self.hl_attr_define(e))
            }
            RedrawEvent::HlGroupSet(evt) => {
                evt.into_iter().for_each(|e| self.hl_group_set(e))
            }
            RedrawEvent::OptionSet(evt) => {
                evt.into_iter().for_each(|e| self.option_set(e));
            }
            RedrawEvent::ModeInfoSet(evt) => {
                evt.into_iter().for_each(|e| self.mode_info_set(e));
            }
            RedrawEvent::ModeChange(evt) => {
                evt.into_iter().for_each(|e| self.mode_change(e));
            }
            RedrawEvent::SetBusy(busy) => self.set_busy(busy),
            RedrawEvent::Flush() => self.flush(nvim),
            RedrawEvent::PopupmenuShow(evt) => {
                evt.into_iter().for_each(|e| self.popupmenu_show(e));
            }
            RedrawEvent::PopupmenuHide() => self.popupmenu_hide(),
            RedrawEvent::PopupmenuSelect(evt) => {
                evt.into_iter().for_each(|e| self.popupmenu_select(e));
            }
            RedrawEvent::TablineUpdate(evt) => {
                evt.into_iter().for_each(|e| self.tabline_update(e, nvim));
            }
            RedrawEvent::CmdlineShow(evt) => {
                evt.into_iter().for_each(|e| self.cmdline_show(e));
            }
            RedrawEvent::CmdlineHide() => self.cmdline_hide(),
            RedrawEvent::CmdlinePos(evt) => {
                evt.into_iter().for_each(|e| self.cmdline_pos(e));
            }
            RedrawEvent::CmdlineSpecialChar(evt) => {
                evt.into_iter().for_each(|e| self.cmdline_special_char(e));
            }
            RedrawEvent::CmdlineBlockShow(evt) => {
                evt.into_iter().for_each(|e| self.cmdline_block_show(e));
            }
            RedrawEvent::CmdlineBlockAppend(evt) => {
                evt.into_iter().for_each(|e| self.cmdline_block_append(e));
            }
            RedrawEvent::CmdlineBlockHide() => self.cmdline_block_hide(),
            RedrawEvent::WildmenuShow(evt) => {
                evt.into_iter().for_each(|e| self.wildmenu_show(e));
            }
            RedrawEvent::WildmenuHide() => self.wildmenu_hide(),
            RedrawEvent::WildmenuSelect(evt) => {
                evt.into_iter().for_each(|e| self.wildmenu_select(e));
            }
            RedrawEvent::Ignored(_) => (),
            RedrawEvent::Unknown(e) => {
                debug!("Received unknown redraw event: {}", e);
            }
        }
    }

    fn handle_gnvim_event(&mut self, event: &GnvimEvent, nvim: &GioNeovim) {
        match event {
            GnvimEvent::CompletionMenuToggleInfo => {
                self.popupmenu.toggle_show_info()
            }
            GnvimEvent::PopupmenuWidth(width) => {
                self.popupmenu.set_width(*width as i32);
            }
            GnvimEvent::PopupmenuWidthDetails(width) => {
                self.popupmenu.set_width_details(*width as i32);
            }
            GnvimEvent::PopupmenuShowMenuOnAllItems(should_show) => {
                self.popupmenu.set_show_menu_on_all_items(*should_show);
            }
            GnvimEvent::Unknown(msg) => {
                debug!("Received unknown GnvimEvent: {}", msg);
            }

            #[cfg(not(feature = "libwebkit2gtk"))]
            GnvimEvent::CursorTooltipLoadStyle(..)
            | GnvimEvent::CursorTooltipShow(..)
            | GnvimEvent::CursorTooltipHide
            | GnvimEvent::CursorTooltipSetStyle(..) => {
                let nvim = nvim.clone();
                let msg =
                    "echom \"Cursor tooltip not supported in this build\"";
                spawn_local(async move {
                    if let Err(err) = nvim.command(&msg).await {
                        error!("Failed to execute nvim command: {}", err)
                    }
                });
            }

            #[cfg(feature = "libwebkit2gtk")]
            GnvimEvent::CursorTooltipLoadStyle(..)
            | GnvimEvent::CursorTooltipShow(..)
            | GnvimEvent::CursorTooltipHide
            | GnvimEvent::CursorTooltipSetStyle(..) => match event {
                GnvimEvent::CursorTooltipLoadStyle(path) => {
                    if let Err(err) =
                        self.cursor_tooltip.load_style(path.clone())
                    {
                        let msg = format!(
                            "echom \"Cursor tooltip load style failed: '{}'\"",
                            err
                        );
                        let nvim = nvim.clone();
                        spawn_local(async move {
                            if let Err(err) = nvim.command(&msg).await {
                                error!(
                                    "Failed to execute nvim command: {}",
                                    err
                                )
                            }
                        });
                    }
                }
                GnvimEvent::CursorTooltipShow(content, row, col) => {
                    self.cursor_tooltip.show(content.clone());

                    let grid = self.grids.get(&self.current_grid).unwrap();
                    let rect = grid.get_rect_for_cell(*row, *col);

                    self.cursor_tooltip.move_to(&rect);
                }
                GnvimEvent::CursorTooltipHide => self.cursor_tooltip.hide(),
                GnvimEvent::CursorTooltipSetStyle(style) => {
                    self.cursor_tooltip.set_style(style)
                }
                _ => unreachable!(),
            },
        }
    }
}
