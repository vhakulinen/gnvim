use std::fmt;
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
    GnvimEvent(GnvimEvent),
}

#[derive(Clone)]
pub enum CursorShape {
    Block,
    Horizontal,
    Vertical,
    Unknown,
}

impl CursorShape {
    fn from_string(name: &str) -> Self {
        match String::from(name).to_lowercase().as_str() {
            "block" => {
                CursorShape::Block
            }
            "horizontal" => {
                CursorShape::Horizontal
            }
            "vertical" => {
                CursorShape::Vertical
            }
            _ => {
                panic!("Unknown cursor shape: {}", name);
            }
        }
    }
}

impl Default for CursorShape {
    fn default() -> Self {
        CursorShape::Block
    }
}

#[derive(Default, Clone)]
pub struct ModeInfo {
    pub cursor_shape: CursorShape,
    /// The cursor's width (in percentages, from 0..1).
    pub cell_percentage: f64,

    // TODO(ville): Implement the rest.
}

impl ModeInfo {
    fn set(&mut self, prop: &str, val: Value) {
        match prop {
            "cursor_shape" => {
                self.cursor_shape = CursorShape::from_string(try_str!(val))
            }
            "cell_percentage" => {
                let mut val = try_u64!(val);

                // Ensure that the val is not zero.
                if val == 0 {
                    val = 100;
                }
                self.cell_percentage = val as f64 / 100.0;
            }
            _ => {
            }
        }
    }
}

pub struct Cell {
    pub text: String,
    pub hl_id: u64,
    pub repeat: u64,
}

pub struct GridLineSegment {
    pub grid: u64,
    pub row: u64,
    pub col_start: u64,
    pub cells: Vec<Cell>,
}

pub enum OptionSet {
    /// font name
    GuiFont(String),
    /// event name
    NotSupported(String),
}

#[derive(Clone)]
pub struct CompletionItem {
    pub word: String,
    pub kind: String,
    pub menu: String,
    pub info: String,
}

pub struct PopupmenuShow {
    pub items: Vec<CompletionItem>,
    pub selected: i64,
    pub row: u64,
    pub col: u64,
}

pub enum RedrawEvent {
    GridLine(Vec<GridLineSegment>),
    /// grid, width, height
    GridResize(u64, u64, u64),
    /// grid, row, col
    GridCursorGoto(u64, u64, u64),
    /// grid
    GridClear(u64),
    /// grid, [top, bot, left, right], rows, cols
    GridScroll(u64, [u64;4], i64, i64),

    /// fg, bg, sp
    DefaultColorsSet(Color, Color, Color),
    /// id, hl
    HlAttrDefine(Vec<(u64, Highlight)>),
    OptionSet(OptionSet),
    /// cusror shape enabled, mode info
    ModeInfoSet(bool, Vec<ModeInfo>),
    /// name, index
    ModeChange(String, u64),
    SetBusy(bool),

    PopupmenuShow(PopupmenuShow),
    PopupmenuHide(),
    PopupmenuSelect(i64),

    Unknown(String),
}

impl fmt::Display for RedrawEvent {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RedrawEvent::GridLine(..) => write!(fmt, "GridLine"),
            RedrawEvent::GridResize(..) => write!(fmt, "GridResize"),
            RedrawEvent::GridCursorGoto(..) => write!(fmt, "GridCursorGoto"),
            RedrawEvent::GridClear(..) => write!(fmt, "GridClear"),
            RedrawEvent::GridScroll(..) => write!(fmt, "GridScroll"),
            RedrawEvent::DefaultColorsSet(..) => write!(fmt, "DefaultColorsSet"),
            RedrawEvent::HlAttrDefine(..) => write!(fmt, "HlAttrDefine"),
            RedrawEvent::OptionSet(..) => write!(fmt, "OptionSet"),
            RedrawEvent::ModeInfoSet(..) => write!(fmt, "ModeInfoSet"),
            RedrawEvent::ModeChange(..) => write!(fmt, "ModeChange"),
            RedrawEvent::SetBusy(..) => write!(fmt, "SetBusy"),
            RedrawEvent::PopupmenuShow(..) => write!(fmt, "PopupmenuShow"),
            RedrawEvent::PopupmenuHide(..) => write!(fmt, "PopupmenuHide"),
            RedrawEvent::PopupmenuSelect(..) => write!(fmt, "PopupmenuSelect"),
            RedrawEvent::Unknown(..) => write!(fmt, "Unknown"),
        }
    }
}

pub enum GnvimEvent {
    SetGuiColors(SetGuiColors),
    CompletionMenuToggleInfo,
    Unknown(String),
}

#[derive(Default)]
pub struct SetGuiColors {
    pub pmenu_bg: Color,
    pub pmenu_fg: Color,
    pub pmenusel_bg: Color,
    pub pmenusel_fg: Color,
}

pub struct NvimBridge {
    tx: Sender<Notify>,
}

impl NvimBridge {
    pub fn new(tx: Sender<Notify>) -> Self {
        NvimBridge {
            tx,
        }
    }
}

impl Handler for NvimBridge {
    fn handle_notify(&mut self, name: &str, args: Vec<Value>) {
        //println!("{}", name);

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
        "Gnvim" => {
            Some(Notify::GnvimEvent(parse_gnvim_event(args)))
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
                    let mut cells: Vec<Cell> = vec!();

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

                        let hl_id = if let Some(hl_id) = hl_id {
                            hl_id
                        } else {
                            cells.last().unwrap().hl_id
                        };

                        cells.push(Cell{
                            hl_id,
                            repeat,
                            text: String::from(text)});
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
            "option_set" => {
                let args = try_array!(args[1]);
                let name = try_str!(args[0]);

                let opt = match name {
                    "guifont" => {
                        let val = try_str!(args[1]);
                        OptionSet::GuiFont(String::from(val))
                    }
                    _ => OptionSet::NotSupported(String::from(name))
                };

                RedrawEvent::OptionSet(opt)
            }
            "mode_info_set" => {
                let args = try_array!(args[1]);
                let cursor_style_enabled = try_bool!(args[0]);

                let mut infos = vec!();
                for info in try_array!(args[1]).into_iter() {
                    //let args = try_array!(args);
                    //let id = try_u64!(args[0]);
                    let map = try_map!(info);

                    let mut mode = ModeInfo::default();
                    for (prop, val) in map {
                        mode.set(try_str!(prop), val.clone());
                    }

                    infos.push(mode);
                }

                RedrawEvent::ModeInfoSet(cursor_style_enabled, infos)
            }
            "mode_change" => {
                let args = try_array!(args[1]);
                let name = try_str!(args[0]);
                let idx = try_u64!(args[1]);
                RedrawEvent::ModeChange(String::from(name), idx)
            }
            "busy_start" => {
                RedrawEvent::SetBusy(true)
            }
            "busy_stop" => {
                RedrawEvent::SetBusy(false)
            }
            "popupmenu_show" => {
                let args = try_array!(args[1]);
                let selected = try_i64!(args[1]);
                let row = try_u64!(args[2]);
                let col = try_u64!(args[3]);

                let mut items = vec!();
                for item in try_array!(args[0]) {
                    let item = try_array!(item);
                    let word = try_str!(item[0]).to_owned();
                    let kind = try_str!(item[1]).to_owned();
                    let menu = try_str!(item[2]).to_owned();
                    let info = try_str!(item[3]).to_owned();

                    items.push(CompletionItem {
                        word, kind, menu, info,
                    });
                }

                RedrawEvent::PopupmenuShow(PopupmenuShow {
                    items, selected, row, col,
                })
            }
            "popupmenu_hide" => {
                RedrawEvent::PopupmenuHide()
            }
            "popupmenu_select"  => {
                let args = try_array!(args[1]);
                let selected = try_i64!(args[0]);
                RedrawEvent::PopupmenuSelect(selected)
            }
            _ => {
                //println!("Unknown redraw event: {}", cmd);
                RedrawEvent::Unknown(cmd.to_string())
            }
        }
    }).collect()
}

fn parse_gnvim_event(args: Vec<Value>) -> GnvimEvent {
    let cmd = try_str!(args[0]);
    match cmd {
        "SetGuiColors" => {
            let mut colors = SetGuiColors::default();

            for e in try_map!(args[1]) {
                let color = Color::from_hex_string(
                        String::from(try_str!(e.1))
                    ).unwrap_or(Color::default());
                match try_str!(e.0) {
                    "pmenu_bg" => colors.pmenu_bg = color,
                    "pmenu_fg" => colors.pmenu_fg = color,
                    "pmenusel_bg" => colors.pmenusel_bg = color,
                    "pmenusel_fg" => colors.pmenusel_fg = color,
                    _ => {
                        println!("Unknown SetGuiColor: {}", try_str!(e.0));
                    }
                }
            }

            GnvimEvent::SetGuiColors(colors)
        }
        "CompletionMenuToggleInfo" => {
            GnvimEvent::CompletionMenuToggleInfo
        }
        _ => {
            GnvimEvent::Unknown(String::from("UGH"))
        }
    }
}
