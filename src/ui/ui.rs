use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use glib;
use gtk;
use gdk;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;

use gtk::prelude::*;

use nvim_bridge::{Notify, RedrawEvent, OptionSet, ModeInfo};
use ui::color::{Highlight, Color};
use ui::grid::Grid;
use thread_guard::ThreadGuard;

type Grids = HashMap<u64, Grid>;

#[derive(Default)]
pub struct HlDefs {
    hl_defs: HashMap<u64, Highlight>,

    pub default_fg: Color,
    pub default_bg: Color,
    pub default_sp: Color,
}

impl HlDefs {
    pub fn get_mut(&mut self, id: &u64) -> Option<&mut Highlight> {
        self.hl_defs.get_mut(id)
    }

    pub fn get(&self, id: &u64) -> Option<&Highlight> {
        self.hl_defs.get(id)
    }

    pub fn insert(&mut self, id: u64, hl: Highlight) -> Option<Highlight> {
        self.hl_defs.insert(id, hl)
    }
}

struct UIState {
    grids: Grids,
    hl_defs: Arc<Mutex<HlDefs>>,
    mode_infos: Vec<ModeInfo>,
    current_grid: u64,
}

pub struct UI {
    win: Arc<ThreadGuard<gtk::ApplicationWindow>>,
    nvim: Arc<Mutex<Neovim>>,
    rx: Receiver<Notify>,
    state: Arc<Mutex<UIState>>,
}

impl UI {
    pub fn init(app: &gtk::Application, rx: Receiver<Notify>, nvim: Arc<Mutex<Neovim>>) -> Self {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("Neovim");
        window.set_default_size(1280, 720);

        let mut hl_defs = HlDefs::default();
        hl_defs.insert(0, Highlight::default());
        let hl_defs = Arc::new(Mutex::new(hl_defs));

        let grid = Grid::new(1, &window.clone().upcast::<gtk::Widget>().downcast::<gtk::Container>().unwrap(), hl_defs.clone());
        let nvim_ref = nvim.clone();
        let source_id = Arc::new(Mutex::new(None));
        grid.connect_da_resize(move |rows, cols| {
            let nvim_ref = nvim_ref.clone();

            let source_id_moved = source_id.clone();
            let new = glib::timeout_add(30, move || {
                let mut nvim = nvim_ref.lock().unwrap();
                nvim.ui_try_resize(cols as u64, rows as u64).unwrap();

                let source_id = source_id_moved.clone();
                let mut source_id = source_id.lock().unwrap();
                *source_id = None;

                Continue(false)
            });

            let source_id = source_id.clone();
            let mut source_id = source_id.lock().unwrap();
            {
                let old = source_id.take();
                if let Some(old) = old {
                    glib::source::source_remove(old);
                }
            }

            *source_id = Some(new);

            false
        });

        // Mouse button click events.
        let nvim_ref = nvim.clone();
        grid.connect_mouse_button_events(move |button, row, col| {
            let mut nvim = nvim_ref.lock().unwrap();
            let input = format!("<{}Mouse><{},{}>", button, col, row);
            nvim.input(&input).expect("Couldn't send mouse input");

            Inhibit(false)
        });

        // Mouse drag events.
        let nvim_ref = nvim.clone();
        grid.connect_motion_events(move |button, row, col| {
            let mut nvim = nvim_ref.lock().unwrap();
            let input = format!("<{}Drag><{},{}>", button, col, row);
            nvim.input(&input).expect("Couldn't send mouse input");

            Inhibit(false)
        });

        // Scrolling events.
        let nvim_ref = nvim.clone();
        grid.connect_scroll_events(move |dir, row, col| {
            let mut nvim = nvim_ref.lock().unwrap();
            let input = format!("<{}><{},{}>", dir, col, row);
            nvim.input(&input).expect("Couldn't send mouse input");

            Inhibit(false)
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
            win: Arc::new(ThreadGuard::new(window)),
            nvim,
            rx,
            state: Arc::new(Mutex::new(UIState {
                grids: grids,
                hl_defs,
                mode_infos: vec!(),
                current_grid: 1,
            })),
        }
    }

    pub fn start(self) {
        let rx = self.rx;
        let state = self.state.clone();
        let win = self.win.clone();
        let nvim = self.nvim.clone();

        thread::spawn(move || {
            let timeout = time::Duration::from_millis(33);

            loop {
                // Use timeout, so we can use this loop to "tick" the current
                // grid (mainly to just blink the cursor).
                let notify = rx.recv_timeout(timeout);

                if let Err(RecvTimeoutError::Disconnected) = notify {
                    // Neovim closed and the sender is disconnected
                    // so we need to exit too.
                    break;
                }

                let state = state.clone();
                let nvim = nvim.clone();
                glib::idle_add(move || {

                    let mut state = state.lock().unwrap();

                    if let Ok(ref notify) = notify {
                        handle_notify(notify, &mut state, nvim.clone());
                    }

                    let grid = state.grids.get(&state.current_grid).unwrap();
                    grid.tick();

                    glib::Continue(false)
                });
            }

            // Close the window once the recv loop exits.
            glib::idle_add(move || {
                win.borrow().close();
                glib::Continue(false)
            });
        });
    }
}

fn handle_notify(notify: &Notify, state: &mut UIState, nvim: Arc<Mutex<Neovim>>) {
    match notify {
        Notify::RedrawEvent(events) => {
            handle_redraw_event(events, state, nvim);
        }
    }
}

fn handle_redraw_event(events: &Vec<RedrawEvent>, state: &mut UIState, nvim: Arc<Mutex<Neovim>>) {
    for event in events {
        match event {
            RedrawEvent::GridLine(lines) => {
                for line in lines {
                    let grid = state.grids.get(&line.grid).unwrap();
                    grid.put_line(line);
                }
            }
            RedrawEvent::GridCursorGoto(grid_id, row, col) => {

                let grid = if *grid_id != state.current_grid {
                    state.grids.get(&state.current_grid).unwrap().set_active(false);
                    state.current_grid = *grid_id;

                    let grid = state.grids.get(grid_id).unwrap();
                    grid.set_active(true);
                    grid
                } else {
                    state.grids.get(grid_id).unwrap()
                };

                grid.cursor_goto(*row, *col);
            }
            RedrawEvent::GridResize(grid, width, height) => {
                let grid = state.grids.get(grid).unwrap();
                grid.resize(*width, *height);
            }
            RedrawEvent::GridClear(grid) => {
                let grid = state.grids.get(grid).unwrap();
                grid.clear();
            }
            RedrawEvent::GridScroll(grid, reg, rows, cols) => {
                let grid = state.grids.get(grid).unwrap();
                grid.scroll(*reg, *rows, *cols);
            }
            RedrawEvent::DefaultColorsSet(fg, bg, sp) => {

                let mut hl_defs = state.hl_defs.lock().unwrap();
                hl_defs.default_fg = *fg;
                hl_defs.default_bg = *bg;
                hl_defs.default_sp = *sp;

                let hl = hl_defs.get_mut(&0).unwrap();
                hl.foreground = Some(*fg);
                hl.background = Some(*bg);
                hl.special = Some(*sp);
            }
            RedrawEvent::HlAttrDefine(defs) => {
                let mut hl_defs = state.hl_defs.lock().unwrap();

                for (id, hl) in defs {
                    hl_defs.insert(*id, *hl);
                }
            }
            RedrawEvent::OptionSet(opt) => {
                match opt {
                    OptionSet::GuiFont(font) => {
                        for grid in (state.grids).values() {
                            grid.set_font(font.to_string());
                        }

                        let grid = state.grids.get(&1).unwrap();
                        let (rows, cols) = grid.calc_size();
                        let mut nvim = nvim.lock().unwrap();
                        nvim.ui_try_resize(cols as u64, rows as u64).unwrap();
                    }
                    OptionSet::NotSupported(name) => {
                        println!("Not supported option set: {}", name);
                    }
                }
            }
            RedrawEvent::ModeInfoSet(_cursor_shape_enabled, infos) => {
                state.mode_infos = infos.clone();
            }
            RedrawEvent::ModeChange(name, idx) => {
                let mode = state.mode_infos.get(*idx as usize).unwrap();
                for grid in state.grids.values() {
                    grid.set_mode(mode);
                }
            }
            RedrawEvent::SetBusy(busy) => {
                for grid in state.grids.values() {
                    grid.set_busy(*busy);
                }
            }
            RedrawEvent::Unknown(e) => {
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
