use std::sync::mpsc::Sender;

use neovim_lib::{Handler, Value};

use ui::color::{Color, Highlight};

macro_rules! try_str {
    ($val:expr) => (
        $val.as_str().unwrap();
    );
}

macro_rules! try_u64 {
    ($val:expr) => (
        $val.as_u64().unwrap();
    );
}

macro_rules! try_i64 {
    ($val:expr) => (
        $val.as_i64().unwrap();
    );
}

macro_rules! try_array {
    ($val:expr) => (
        $val.as_array().unwrap();
    );
}

macro_rules! try_map {
    ($val:expr) => (
        $val.as_map().unwrap();
    );
}

macro_rules! try_bool {
    ($val:expr) => (
        $val.as_bool().unwrap();
    );
}

impl Highlight {
    fn set(&mut self, prop: &str, val: Value) {
        match prop {
            "foreground" => {
                self.foreground = if let Some(val) = val.as_u64() {
                    Some(Color::from_u64(val))
                } else {
                    None
                }
            }
            "background" => {
                self.background = if let Some(val) = val.as_u64() {
                    Some(Color::from_u64(val))
                } else {
                    None
                }
            }
            "special" => {
                self.special = if let Some(val) = val.as_u64() {
                    Some(Color::from_u64(val))
                } else {
                    None
                }
            }
            "reverse" => {
                self.reverse = try_bool!(val);
            }
            "italic" => {
                self.italic = try_bool!(val);
            }
            "bold" => {
                self.bold = try_bool!(val);
            }
            "underline" => {
                self.underline = try_bool!(val);
            }
            "undercurl" => {
                self.undercurl = try_bool!(val);
            }
            "cterm_fg" => {}
            "cterm_bg" => {}
            _ => {
                panic!("Unknown highligh property: {}", prop);
            }
        }
    }
}

pub enum Notify {
    RedrawEvent(Vec<RedrawEvent>),
}

pub struct Cell {
    pub text: String,
    pub hl_id: Option<u64>,
    pub repeat: u64,
}

pub struct GridLineSegment {
    pub grid: u64,
    pub row: u64,
    pub col_start: u64,
    pub cells: Vec<Cell>,
}

pub enum RedrawEvent {
    GridLine(Vec<GridLineSegment>),
    // grid, width, height
    GridResize(u64, u64, u64),
    // grid, row, col
    GridCursorGoto(u64, u64, u64),
    // grid
    GridClear(u64),
    // grid, [top, bot, left, right], rows, cols
    GridScroll(u64, [u64;4], i64, i64),

    // fg, bg, sp
    DefaultColorsSet(Color, Color, Color),
    // id, hl
    HlAttrDefine(Vec<(u64, Highlight)>),
    Unknown(String),
}

#[derive(Default)]
struct DefaultColors {
    foreground: Color,
    background: Color,
    special: Color,
}

pub struct NvimBridge {
    tx: Sender<Notify>,
    default_colors: DefaultColors,
}

impl NvimBridge {
    pub fn new(tx: Sender<Notify>) -> Self {
        NvimBridge {
            tx,
            default_colors: DefaultColors::default(),
        }
    }
}

impl Handler for NvimBridge {
    fn handle_notify(&mut self, name: &str, args: Vec<Value>) {
        println!("{}", name);

        if let Some(notify) = parse_notify(name, args) {
            self.tx.send(notify).unwrap();
        } else {
            println!("Unknown notify: {}", name);
        }
    }
}

fn parse_notify(name: &str, args: Vec<Value>) -> Option<Notify> {
    match name {
        "redraw" => {
            Some(Notify::RedrawEvent(parse_redraw_event(args)))
        }
        _ => None
    }
}

/*
GLOBALS: 
    ["set_title", title]
    ["set_icon", icon]
    ["mode_info_set", cursor_style_enabled, mode_info]
    ["option_set", name, value]
    ["mode_change", mode, mode_idx]
    ["mouse_on"]
    ["mouse_off"]
    ["busy_on"]
    ["busy_off"]
    ["suspend"]
    ["update_menu"]
    ["bell"]
    ["visual_bell"]
 */

fn parse_redraw_event(args: Vec<Value>) -> Vec<RedrawEvent> {

    args.into_iter().map(|args| {
        let cmd = try_str!(args[0]);
        match cmd {
            "grid_line" => {
                let mut lines = vec!();

                for entry in try_array!(args)[1..].into_iter() {
                    let entry = try_array!(entry);
                    let grid = try_u64!(entry[0]);
                    let row = try_u64!(entry[1]);
                    let col_start = try_u64!(entry[2]);
                    let mut cells = vec!();

                    for entry in try_array!(entry[3]) {
                        let entry = try_array!(entry);
                        let text = try_str!(entry[0]);
                        let hl_id = if entry.len() >= 2 {
                            entry[1].as_u64()
                        } else {
                            None
                        };
                        let repeat = if entry.len() >= 3 {
                            try_u64!(entry[2])
                        } else {
                            1
                        };

                        cells.push(Cell{hl_id, repeat, text: String::from(text)});
                    }

                    lines.push(GridLineSegment{grid, row, col_start, cells});
                }

                RedrawEvent::GridLine(lines)
            }
            "grid_cursor_goto" => {
                let args = try_array!(args[1]);
                RedrawEvent::GridCursorGoto(
                    try_u64!(args[0]),
                    try_u64!(args[1]),
                    try_u64!(args[2]),
                )
            }
            "grid_resize" => {
                let args = try_array!(args[1]);
                let grid = try_u64!(args[0]);
                let width = try_u64!(args[1]);
                let height = try_u64!(args[2]);

                RedrawEvent::GridResize(grid, width, height)
            }
            "grid_clear" => {
                let args = try_array!(args[1]);
                let id = try_u64!(args[0]);
                RedrawEvent::GridClear(id)
            }
            "grid_scroll" => {
                let args = try_array!(args[1]);

                let id = try_u64!(args[0]);
                let top = try_u64!(args[1]);
                let bot = try_u64!(args[2]);
                let left = try_u64!(args[3]);
                let right = try_u64!(args[4]);
                let rows = try_i64!(args[5]);
                let cols = try_i64!(args[6]);

                //RedrawEvent::Unknown(cmd.to_string())
                RedrawEvent::GridScroll(id, [top, bot, left, right], rows, cols)
            }
            "default_colors_set" => {
                let args = try_array!(args[1]);

                let fg = Color::from_u64(try_i64!(args[0]) as u64);
                let bg = Color::from_u64(try_i64!(args[1]) as u64);
                let sp = Color::from_u64(try_i64!(args[2]) as u64);

                RedrawEvent::DefaultColorsSet(fg, bg, sp)
            }
            "hl_attr_define" => {
                let mut hls = vec!();

                for args in try_array!(args)[1..].into_iter() {
                    let args = try_array!(args);
                    let id = try_u64!(args[0]);
                    let map = try_map!(args[1]);

                    let mut hl = Highlight::default();
                    for (prop, val) in map {
                        hl.set(try_str!(prop), val.clone());
                    }

                    hls.push((id, hl));
                }
                
                RedrawEvent::HlAttrDefine(hls)
            }
            _ => {
                //println!("Unknown redraw event: {}", cmd);
                RedrawEvent::Unknown(cmd.to_string())
            }
        }
    }).collect()
}
