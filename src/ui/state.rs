use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glib;
use gtk;
use gtk::prelude::*;

use log::{debug, error};
use nvim_rs::Tabpage;

use crate::nvim_bridge::{
    CmdlinePos, CmdlineSpecialChar, DefaultColorsSet, GnvimEvent,
    GridCursorGoto, GridResize, HlAttrDefine, ModeChange, ModeInfo,
    ModeInfoSet, Notify, OptionSet, RedrawEvent, TablineUpdate,
};
use crate::nvim_gio::GioNeovim;
use crate::ui::cmdline::Cmdline;
use crate::ui::color::HlDefs;
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
}

impl UIState {
    pub fn handle_notify(
        &mut self,
        window: &gtk::ApplicationWindow,
        notify: &Notify,
        nvim: &GioNeovim,
    ) {
        match notify {
            Notify::RedrawEvent(events) => {
                events.iter().for_each(|e| {
                    self.handle_redraw_event(window, e, &nvim);
                });
            }
            Notify::GnvimEvent(event) => match event {
                Ok(event) => self.handle_gnvim_event(event, nvim),
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

    fn handle_gnvim_event(&mut self, event: &GnvimEvent, nvim: &GioNeovim) {
        match event {
            GnvimEvent::SetGuiColors(colors) => {
                self.popupmenu.set_colors(colors.pmenu, &self.hl_defs);
                self.tabline.set_colors(colors.tabline, &self.hl_defs);
                self.cmdline.set_colors(colors.cmdline, &self.hl_defs);
                self.cmdline
                    .wildmenu_set_colors(&colors.wildmenu, &self.hl_defs);
            }
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

    fn handle_redraw_event(
        &mut self,
        window: &gtk::ApplicationWindow,
        event: &RedrawEvent,
        nvim: &GioNeovim,
    ) {
        match event {
            RedrawEvent::SetTitle(evt) => {
                evt.iter().for_each(|title| {
                    window.set_title(title);
                });
            }
            RedrawEvent::GridLine(evt) => {
                evt.iter().for_each(|line| {
                    let grid = self.grids.get(&line.grid).unwrap();
                    grid.put_line(line, &self.hl_defs);
                });
            }
            RedrawEvent::GridCursorGoto(evt) => {
                evt.iter().for_each(
                    |GridCursorGoto {
                         grid: grid_id,
                         row,
                         col,
                     }| {
                        // Gird cursor goto sets the current cursor to grid_id,
                        // so we'll need to handle that here...
                        let grid = if *grid_id != self.current_grid {
                            // ...so if the grid_id is not same as the self tells us,
                            // set the previous current grid to inactive self.
                            self.grids
                                .get(&self.current_grid)
                                .unwrap()
                                .set_active(false);
                            self.current_grid = *grid_id;

                            // And set the new current grid to active.
                            let grid = self.grids.get(grid_id).unwrap();
                            grid.set_active(true);
                            grid
                        } else {
                            self.grids.get(grid_id).unwrap()
                        };

                        // And after all that, set the current grid's cursor position.
                        grid.cursor_goto(*row, *col);
                    },
                );
            }
            RedrawEvent::GridResize(evt) => {
                evt.iter().for_each(
                    |GridResize {
                         grid,
                         width,
                         height,
                     }| {
                        let grid = self.grids.get(grid).unwrap();
                        grid.resize(*width, *height);
                    },
                );
            }
            RedrawEvent::GridClear(evt) => {
                evt.iter().for_each(|grid| {
                    let grid = self.grids.get(grid).unwrap();
                    grid.clear(&self.hl_defs);
                });
            }
            RedrawEvent::GridScroll(evt) => {
                evt.iter().for_each(|info| {
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
                });
            }
            RedrawEvent::DefaultColorsSet(evt) => {
                evt.iter().for_each(|DefaultColorsSet { fg, bg, sp }| {
                    self.hl_defs.default_fg = *fg;
                    self.hl_defs.default_bg = *bg;
                    self.hl_defs.default_sp = *sp;

                    {
                        // NOTE(ville): Not sure if these are actually needed.
                        let hl = self.hl_defs.get_mut(&0).unwrap();
                        hl.foreground = Some(*fg);
                        hl.background = Some(*bg);
                        hl.special = Some(*sp);
                    }

                    for grid in self.grids.values() {
                        grid.redraw(&self.hl_defs);
                    }

                    #[cfg(feature = "libwebkit2gtk")]
                    self.cursor_tooltip.set_colors(*fg, *bg);
                });
            }
            RedrawEvent::HlAttrDefine(evt) => {
                evt.iter().for_each(|HlAttrDefine { id, hl }| {
                    self.hl_defs.insert(*id, *hl);
                });
            }
            RedrawEvent::OptionSet(evt) => {
                evt.iter().for_each(|opt| match opt {
                    OptionSet::GuiFont(font) => {
                        let font =
                            Font::from_guifont(font).unwrap_or(Font::default());

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

                        opts.line_space = *val;

                        self.resize_on_flush = Some(opts);
                    }
                    OptionSet::NotSupported(name) => {
                        debug!("Not supported option set: {}", name);
                    }
                });
            }
            RedrawEvent::ModeInfoSet(evt) => {
                evt.iter().for_each(|ModeInfoSet { mode_info, .. }| {
                    self.mode_infos = mode_info.clone();
                });
            }
            RedrawEvent::ModeChange(evt) => {
                evt.iter().for_each(|ModeChange { index, .. }| {
                    let mode = self.mode_infos.get(*index as usize).unwrap();
                    // Broadcast the mode change to all grids.
                    // TODO(ville): It might be enough to just set the mode to the
                    //              current active grid.
                    for grid in self.grids.values() {
                        grid.set_mode(mode);
                    }
                });
            }
            RedrawEvent::SetBusy(busy) => {
                for grid in self.grids.values() {
                    grid.set_busy(*busy);
                }
            }
            RedrawEvent::Flush() => {
                for grid in self.grids.values() {
                    grid.flush(&self.hl_defs);
                }

                if let Some(opts) = self.resize_on_flush.take() {
                    for grid in self.grids.values() {
                        grid.update_cell_metrics(
                            opts.font.clone(),
                            opts.line_space,
                        );
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
            }
            RedrawEvent::PopupmenuShow(evt) => {
                evt.iter().for_each(|popupmenu| {
                    self.popupmenu
                        .set_items(popupmenu.items.clone(), &self.hl_defs);

                    let grid = self.grids.get(&self.current_grid).unwrap();
                    let rect =
                        grid.get_rect_for_cell(popupmenu.row, popupmenu.col);

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
                                self.cursor_tooltip
                                    .force_gravity(Some(Gravity::Down));
                            } else {
                                self.cursor_tooltip
                                    .force_gravity(Some(Gravity::Up));
                            }

                            self.cursor_tooltip.refresh_position();
                        }
                    }
                });
            }
            RedrawEvent::PopupmenuHide() => {
                self.popupmenu.hide();

                // Undo any force positioning of cursor tool tip that might
                // have occured on popupmenu show.
                #[cfg(feature = "libwebkit2gtk")]
                {
                    self.cursor_tooltip.force_gravity(None);
                    self.cursor_tooltip.refresh_position();
                }
            }
            RedrawEvent::PopupmenuSelect(evt) => {
                evt.iter().for_each(|selected| {
                    self.popupmenu.select(*selected as i32, &self.hl_defs);
                });
            }
            RedrawEvent::TablineUpdate(evt) => {
                evt.iter().for_each(|TablineUpdate { current, tabs }| {
                    let current = Tabpage::new(current.clone(), nvim.clone());
                    let tabs = tabs
                        .iter()
                        .map(|v| {
                            (
                                Tabpage::new(v.0.clone(), nvim.clone()),
                                v.1.clone(),
                            )
                        })
                        .collect();
                    self.tabline.update(current, tabs);
                });
            }
            RedrawEvent::CmdlineShow(evt) => {
                evt.iter().for_each(|cmdline_show| {
                    self.cmdline.show(cmdline_show, &self.hl_defs);
                });
            }
            RedrawEvent::CmdlineHide() => {
                self.cmdline.hide();
            }
            RedrawEvent::CmdlinePos(evt) => {
                evt.iter().for_each(|CmdlinePos { pos, level }| {
                    self.cmdline.set_pos(*pos, *level);
                });
            }
            RedrawEvent::CmdlineSpecialChar(evt) => {
                evt.iter().for_each(
                    |CmdlineSpecialChar {
                         character: ch,
                         shift,
                         level,
                     }| {
                        self.cmdline.show_special_char(
                            ch.clone(),
                            *shift,
                            *level,
                        );
                    },
                );
            }
            RedrawEvent::CmdlineBlockShow(evt) => {
                evt.iter().for_each(|show| {
                    self.cmdline.show_block(show, &self.hl_defs);
                });
            }
            RedrawEvent::CmdlineBlockAppend(evt) => {
                evt.iter().for_each(|line| {
                    self.cmdline.block_append(line, &self.hl_defs);
                });
            }
            RedrawEvent::CmdlineBlockHide() => {
                self.cmdline.hide_block();
            }
            RedrawEvent::WildmenuShow(evt) => {
                evt.iter().for_each(|items| {
                    self.cmdline.wildmenu_show(&items.0);
                });
            }
            RedrawEvent::WildmenuHide() => {
                self.cmdline.wildmenu_hide();
            }
            RedrawEvent::WildmenuSelect(evt) => {
                evt.iter().for_each(|item| {
                    self.cmdline.wildmenu_select(*item);
                });
            }
            RedrawEvent::Ignored(_) => (),
            RedrawEvent::Unknown(e) => {
                debug!("Received unknown redraw event: {}", e);
            }
        }
    }
}
