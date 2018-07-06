use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use glib;
use gtk;
use gdk;
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

        let im_context = gtk::IMMulticontext::new();
        let nvim_ref = nvim.clone();
        im_context.connect_commit(move |_, mut input| {

            // Some quirk with gtk and/or neovim. The python-gui
            // does the same thing.
            if input == "<" {
                input = "<lt>"
            }

            let mut nvim = nvim_ref.lock().unwrap();
            nvim.input(input).expect("Couldn't send input");
        });

        let im_ref = im_context.clone();
        let nvim_ref = nvim.clone();
        window.connect_key_press_event(move |_, e| {

            if im_ref.filter_keypress(e) {
                Inhibit(true)
            } else {
                if let Some(input) = event_to_nvim_input(e) {
                    let mut nvim = nvim_ref.lock().unwrap();
                    nvim.input(input.as_str()).expect("Couldn't send input");
                    return Inhibit(true);
                }

                Inhibit(false)
            }
        });

        let im_ref = im_context.clone();
        window.connect_key_release_event(move |_, e| {
            im_ref.filter_keypress(e);
            Inhibit(false)
        });

        let im_ref = im_context.clone();
        window.connect_focus_in_event(move |_, _| {
            im_ref.focus_in();
            Inhibit(false)
        });

        let im_ref = im_context.clone();
        window.connect_focus_out_event(move |_, _| {
            im_ref.focus_out();
            Inhibit(false)
        });

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

fn keyname_to_nvim_key(s: &str) -> Option<&str> {
    // Sourced from python-gui.
    match s {
        "slash" => Some("/"),
        "backslash" => Some("\\"),
        "dead_circumflex" => Some("^"),
        "at" => Some("@"),
        "numbersign" => Some("#"),
        "dollar" => Some("$"),
        "percent" => Some("%"),
        "ampersand" => Some("&"),
        "asterisk" => Some("*"),
        "parenleft" => Some("("),
        "parenright" => Some(")"),
        "underscore" => Some("_"),
        "plus" => Some("+"),
        "minus" => Some("-"),
        "bracketleft" => Some("["),
        "bracketright" => Some("]"),
        "braceleft" => Some("{"),
        "braceright" => Some("}"),
        "dead_diaeresis" => Some("\""),
        "dead_acute" => Some("\'"),
        "less" => Some("<"),
        "greater" => Some(">"),
        "comma" => Some(","),
        "period" => Some("."),
        "BackSpace" => Some("BS"),
        "Return" => Some("CR"),
        "Escape" => Some("Esc"),
        "Delete" => Some("Del"),
        "Page_Up" => Some("PageUp"),
        "Page_Down" => Some("PageDown"),
        "Enter" => Some("CR"),
        "ISO_Left_Tab" => Some("Tab"),
        "Tab" => Some("Tab"),
        "Up" => Some("Up"),
        "Down" => Some("Down"),
        "Left" => Some("Left"),
        "Right" => Some("Right"),
        "Home" => Some("Home"),
        "End" => Some("End"),
    _ => None,
    }
}

fn event_to_nvim_input(e: &gdk::EventKey) -> Option<String> {
    let mut input = String::from("");

    let keyval = e.get_keyval();
    let keyname = gdk::keyval_name(keyval).unwrap();

    let state = e.get_state();

    if state.contains(gdk::ModifierType::SHIFT_MASK) {
        input.push_str("S-");
    }
    if state.contains(gdk::ModifierType::CONTROL_MASK) {
        input.push_str("C-");
    }
    if state.contains(gdk::ModifierType::MOD1_MASK) {
        input.push_str("A-");
    }

    if keyname.chars().count() > 1 {
        let n = keyname_to_nvim_key(keyname.as_str());

        if let Some(n) = n {
            input.push_str(n);
        } else {
            println!("NO KEY FOR NVIM ('{}')", keyname);
            return None;
        }
    } else {
        input.push(gdk::keyval_to_unicode(keyval).unwrap());
    }

    Some(format!("<{}>", input))
}
