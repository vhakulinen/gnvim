use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use glib;
use gtk;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;

use gtk::prelude::*;

use nvim_bridge::{Notify, RedrawEventGrid, GridLineSegment};
use ui::color::{Color, Highlight};
use ui::grid::Grid;

type Grids = HashMap<u64, Grid>;
pub type HlDefs = HashMap<u64, Highlight>;

pub struct UI {
    nvim: Arc<Mutex<Neovim>>,
    rx: Receiver<Notify>,
    grids: Arc<Mutex<Grids>>,
    hl_defs: Arc<Mutex<HlDefs>>
}

impl UI {
    pub fn init(app: &gtk::Application, rx: Receiver<Notify>, nvim: Arc<Mutex<Neovim>>) -> Self {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("Neovim");
        window.set_default_size(1280, 720);

        let mut hl_defs = HlDefs::default();
        hl_defs.insert(0, Highlight::default());
        let hl_defs = Arc::new(Mutex::new(hl_defs));

        let grid = Grid::new(1, &window, hl_defs.clone());
        let nvim_ref = nvim.clone();
        grid.connect_resize(move |rows, cols| {
            let mut nvim = nvim_ref.lock().unwrap();
            nvim.ui_try_resize(cols, rows).unwrap();

            false
        });

        let mut grids = HashMap::new();
        grids.insert(1, grid);

        window.show_all();

        UI {
            nvim,
            rx,
            grids: Arc::new(Mutex::new(grids)),
            hl_defs,
        }
    }

    pub fn start(self) {
        let rx = self.rx;
        let grids = self.grids.clone();
        let hl_defs = self.hl_defs.clone();
        thread::spawn(move || loop {
            let notify = rx.recv().unwrap();
            let grids = grids.clone();
            let hl_defs = hl_defs.clone();

            glib::idle_add(move || {
                handle_notify(&notify, grids.clone(), hl_defs.clone());

                glib::Continue(false)
            });
        });
    }
}

fn handle_notify(notify: &Notify, grids: Arc<Mutex<Grids>>, hl_defs: Arc<Mutex<HlDefs>>) {
    match notify {
        Notify::RedrawEventGrid(events) => {
            handle_redraw_event(events, grids.clone(), hl_defs.clone());
        }
    }
}

fn handle_redraw_event(events: &Vec<RedrawEventGrid>, grids: Arc<Mutex<Grids>>, hl_defs: Arc<Mutex<HlDefs>>) {
    let grids = grids.lock().unwrap();

    for event in events {
        match event {
            RedrawEventGrid::Line(lines) => {
                println!("girdline");

                for line in lines {
                    let grid = grids.get(&line.grid).unwrap();
                    grid.put_line(line);
                }

                //grid_line(da, ctx, state, lines);
            }
            //RedrawEvent::Put(s) => {
                //println!("{}", s);
                //put(da, s, ctx, state);
            //}
            RedrawEventGrid::CursorGoto(grid, row, col) => {
                let grid = grids.get(grid).unwrap();
                grid.cursor_goto(*row, *col);
            }
            RedrawEventGrid::Resize(grid, width, height) => {
                let grid = grids.get(grid).unwrap();
                grid.resize(*width, *height);
                // TODO(ville): What else do we need to do here? Will there be a situtation where neovim
                // actually resizes it?
                //state.scroll_region = [ 0, cols - 1, 0, rows - 1 ];

                // After resize the screen is redrawn, and neovim doesn't tell us to put the cursor to the
                // "beginning" but acts like its there.
                //state.grid.cursor.0 = 0;
                //state.grid.cursor.1 = 0;
            }
            RedrawEventGrid::Clear(grid) => {
                let grid = grids.get(grid).unwrap();
                grid.clear();
            }
            RedrawEventGrid::DefaultColorsSet(fg, bg, sp) => {

                {
                    let mut hl_defs = hl_defs.lock().unwrap();
                    let hl = hl_defs.get_mut(&0).unwrap();
                    hl.foreground = Some(*fg);
                    hl.background = Some(*bg);
                    hl.special = Some(*sp);
                }

                for grid in grids.values() {
                    grid.set_default_colors(*fg, *bg, *sp);
                }
            }
            RedrawEventGrid::HlAttrDefine(defs) => {
                let mut hl_defs = hl_defs.lock().unwrap();

                for (id, hl) in defs {
                    hl_defs.insert(*id, *hl);
                }
            }
            RedrawEventGrid::Unknown(e) => {
                println!("Received unknow redraw event: {}", e);
            }
        }
    }
}
