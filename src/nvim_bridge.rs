use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::Sender;

use neovim_lib::{neovim_api::Tabpage, Handler, Value};

use ui::color::{Color, Highlight};

macro_rules! try_str {
    ($val:expr) => {
        $val.as_str().unwrap();
    };
}

macro_rules! try_u64 {
    ($val:expr) => {
        $val.as_u64().unwrap();
    };
}

macro_rules! try_i64 {
    ($val:expr) => {
        $val.as_i64().unwrap();
    };
}

macro_rules! try_array {
    ($val:expr) => {
        $val.as_array().unwrap();
    };
}

macro_rules! try_map {
    ($val:expr) => {
        $val.as_map().unwrap();
    };
}

macro_rules! try_bool {
    ($val:expr) => {
        $val.as_bool().unwrap();
    };
}

impl Highlight {
    fn from_map_val(map: &Vec<(Value, Value)>) -> Self {
        let mut hl = Highlight::default();
        for (prop, val) in map {
            hl.set(try_str!(prop), val.clone());
        }
        hl
    }

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
                println!("Unknown highligh property: {}", prop);
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
            "block" => CursorShape::Block,
            "horizontal" => CursorShape::Horizontal,
            "vertical" => CursorShape::Vertical,
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
            _ => {}
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

#[derive(Debug)]
pub struct CmdlineShow {
    pub content: Vec<(u64, String)>,
    pub pos: u64,
    pub firstc: String,
    pub prompt: String,
    pub indent: u64,
    pub level: u64,
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
    GridScroll(u64, [u64; 4], i64, i64),

    /// fg, bg, sp
    DefaultColorsSet(Color, Color, Color),
    /// id, hl
    HlAttrDefine(Vec<(u64, Highlight)>),
    OptionSet(Vec<OptionSet>),
    /// cusror shape enabled, mode info
    ModeInfoSet(bool, Vec<ModeInfo>),
    /// name, index
    ModeChange(String, u64),
    SetBusy(bool),

    Flush(),

    PopupmenuShow(PopupmenuShow),
    PopupmenuHide(),
    PopupmenuSelect(i64),

    TablineUpdate(Tabpage, Vec<(Tabpage, String)>),

    CmdlineShow(CmdlineShow),
    CmdlineHide(),
    CmdlinePos(u64, u64),
    CmdlineSpecialChar(String, bool, u64),
    CmdlineBlockShow(Vec<(u64, String)>),
    CmdlineBlockAppend((u64, String)),
    CmdlineBlockHide(),

    WildmenuShow(Vec<String>),
    WildmenuHide(),
    WildmenuSelect(i64),

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
            RedrawEvent::DefaultColorsSet(..) => {
                write!(fmt, "DefaultColorsSet")
            }
            RedrawEvent::HlAttrDefine(..) => write!(fmt, "HlAttrDefine"),
            RedrawEvent::OptionSet(..) => write!(fmt, "OptionSet"),
            RedrawEvent::ModeInfoSet(..) => write!(fmt, "ModeInfoSet"),
            RedrawEvent::ModeChange(..) => write!(fmt, "ModeChange"),
            RedrawEvent::SetBusy(..) => write!(fmt, "SetBusy"),
            RedrawEvent::Flush(..) => write!(fmt, "Flush"),
            RedrawEvent::PopupmenuShow(..) => write!(fmt, "PopupmenuShow"),
            RedrawEvent::PopupmenuHide(..) => write!(fmt, "PopupmenuHide"),
            RedrawEvent::PopupmenuSelect(..) => write!(fmt, "PopupmenuSelect"),
            RedrawEvent::TablineUpdate(..) => write!(fmt, "TablineUpdate"),
            RedrawEvent::CmdlineShow(..) => write!(fmt, "CmdlineShow"),
            RedrawEvent::CmdlineHide(..) => write!(fmt, "CmdlineHide"),
            RedrawEvent::CmdlinePos(..) => write!(fmt, "CmdlinePos"),
            RedrawEvent::CmdlineSpecialChar(..) => {
                write!(fmt, "CmdlineSpecialChar")
            }
            RedrawEvent::CmdlineBlockShow(..) => {
                write!(fmt, "CmdlineBlockShow")
            }
            RedrawEvent::CmdlineBlockAppend(..) => {
                write!(fmt, "CmdlineBlockAppend")
            }
            RedrawEvent::CmdlineBlockHide(..) => {
                write!(fmt, "CmdlineBlockHide")
            }
            RedrawEvent::WildmenuShow(..) => write!(fmt, "WildmenuShow"),
            RedrawEvent::WildmenuHide(..) => write!(fmt, "WildmenuHide"),
            RedrawEvent::WildmenuSelect(..) => write!(fmt, "WildmenuSelect"),
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
pub struct WildmenuColors {
    pub bg: Color,
    pub fg: Color,
    pub sel_bg: Color,
    pub sel_fg: Color,
}

#[derive(Default)]
pub struct PmenuColors {
    pub bg: Color,
    pub fg: Color,
    pub sel_bg: Color,
    pub sel_fg: Color,
}

#[derive(Default)]
pub struct TablineColors {
    pub fg: Color,
    pub bg: Color,
    pub fill_fg: Color,
    pub fill_bg: Color,
    pub sel_bg: Color,
    pub sel_fg: Color,
}

#[derive(Default)]
pub struct CmdlineColors {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
}

#[derive(Default)]
pub struct SetGuiColors {
    pub pmenu: PmenuColors,
    pub tabline: TablineColors,
    pub cmdline: CmdlineColors,
    pub wildmenu: WildmenuColors,
}

pub struct NvimBridge {
    tx: Sender<Notify>,
}

impl NvimBridge {
    pub fn new(tx: Sender<Notify>) -> Self {
        NvimBridge { tx }
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
        "redraw" => Some(Notify::RedrawEvent(parse_redraw_event(args))),
        "Gnvim" => Some(Notify::GnvimEvent(parse_gnvim_event(args))),
        _ => None,
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
    args.into_iter()
        .map(|args| {
            let cmd = try_str!(args[0]);
            match cmd {
                "grid_line" => {
                    let mut lines = vec![];

                    for entry in try_array!(args)[1..].into_iter() {
                        let entry = try_array!(entry);
                        let grid = try_u64!(entry[0]);
                        let row = try_u64!(entry[1]);
                        let col_start = try_u64!(entry[2]);
                        let mut cells: Vec<Cell> = vec![];

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

                            cells.push(Cell {
                                hl_id,
                                repeat,
                                text: String::from(text),
                            });
                        }

                        lines.push(GridLineSegment {
                            grid,
                            row,
                            col_start,
                            cells,
                        });
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

                    RedrawEvent::GridScroll(
                        id,
                        [top, bot, left, right],
                        rows,
                        cols,
                    )
                }
                "default_colors_set" => {
                    let args = try_array!(args[1]);

                    let fg = Color::from_u64(try_i64!(args[0]) as u64);
                    let bg = Color::from_u64(try_i64!(args[1]) as u64);
                    let sp = Color::from_u64(try_i64!(args[2]) as u64);

                    RedrawEvent::DefaultColorsSet(fg, bg, sp)
                }
                "hl_attr_define" => {
                    let mut hls = vec![];

                    for args in try_array!(args)[1..].into_iter() {
                        let args = try_array!(args);
                        let id = try_u64!(args[0]);
                        let map = try_map!(args[1]);

                        let hl = Highlight::from_map_val(map);

                        hls.push((id, hl));
                    }

                    RedrawEvent::HlAttrDefine(hls)
                }
                "option_set" => {
                    let mut opts = vec![];
                    for arg in try_array!(args)[1..].into_iter() {
                        let name = try_str!(arg[0]);
                        let opt = match name {
                            "guifont" => {
                                let val = try_str!(arg[1]);
                                OptionSet::GuiFont(String::from(val))
                            }
                            _ => OptionSet::NotSupported(String::from(name)),
                        };

                        opts.push(opt);
                    }

                    RedrawEvent::OptionSet(opts)
                }
                "mode_info_set" => {
                    let args = try_array!(args[1]);
                    let cursor_style_enabled = try_bool!(args[0]);

                    let mut infos = vec![];
                    for info in try_array!(args[1]).into_iter() {
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
                "busy_start" => RedrawEvent::SetBusy(true),
                "busy_stop" => RedrawEvent::SetBusy(false),
                "flush" => RedrawEvent::Flush(),
                "popupmenu_show" => {
                    let args = try_array!(args[1]);
                    let selected = try_i64!(args[1]);
                    let row = try_u64!(args[2]);
                    let col = try_u64!(args[3]);

                    let mut items = vec![];
                    for item in try_array!(args[0]) {
                        let item = try_array!(item);
                        let word = try_str!(item[0]).to_owned();
                        let kind = try_str!(item[1]).to_owned();
                        let menu = try_str!(item[2]).to_owned();
                        let info = try_str!(item[3]).to_owned();

                        items.push(CompletionItem {
                            word,
                            kind,
                            menu,
                            info,
                        });
                    }

                    RedrawEvent::PopupmenuShow(PopupmenuShow {
                        items,
                        selected,
                        row,
                        col,
                    })
                }
                "popupmenu_hide" => RedrawEvent::PopupmenuHide(),
                "popupmenu_select" => {
                    let args = try_array!(args[1]);
                    let selected = try_i64!(args[0]);
                    RedrawEvent::PopupmenuSelect(selected)
                }
                "tabline_update" => {
                    let args = try_array!(args[1]);
                    let cur_tab = Tabpage::new(args[0].clone());
                    let tabs = try_array!(args[1])
                        .iter()
                        .map(|item| {
                            let m = map_to_hash(&item);
                            (
                                Tabpage::new((*m.get("tab").unwrap()).clone()),
                                try_str!(m.get("name").unwrap()).to_string(),
                            )
                        })
                        .collect();

                    RedrawEvent::TablineUpdate(cur_tab, tabs)
                }
                "cmdline_show" => {
                    let args = try_array!(args[1]);
                    let content: Vec<(u64, String)> = try_array!(args[0])
                        .into_iter()
                        .map(|v| {
                            let hl_id = try_u64!(v[0]);
                            let text = try_str!(v[1]);

                            (hl_id, String::from(text))
                        })
                        .collect();
                    let pos = try_u64!(args[1]);
                    let firstc = String::from(try_str!(args[2]));
                    let prompt = String::from(try_str!(args[3]));
                    let indent = try_u64!(args[4]);
                    let level = try_u64!(args[5]);

                    RedrawEvent::CmdlineShow(CmdlineShow {
                        content,
                        pos,
                        firstc,
                        prompt,
                        indent,
                        level,
                    })
                }
                "cmdline_hide" => RedrawEvent::CmdlineHide(),
                "cmdline_pos" => {
                    let args = try_array!(args[1]);
                    let pos = try_u64!(args[0]);
                    let level = try_u64!(args[1]);
                    RedrawEvent::CmdlinePos(pos, level)
                }
                "cmdline_special_char" => {
                    let args = try_array!(args[1]);
                    let c = try_str!(args[0]);
                    let shift = try_bool!(args[1]);
                    let level = try_u64!(args[2]);
                    RedrawEvent::CmdlineSpecialChar(c.to_string(), shift, level)
                }
                "cmdline_block_show" => {
                    let args = try_array!(args[1]);
                    let args = try_array!(args[0]);

                    let lines: Vec<(u64, String)> = try_array!(args[0])
                        .iter()
                        .map(|v| {
                            let hl_id = try_u64!(v[0]);
                            let text = try_str!(v[1]);

                            (hl_id, text.to_string())
                        })
                        .collect();

                    RedrawEvent::CmdlineBlockShow(lines)
                }
                "cmdline_block_append" => {
                    let args = try_array!(args[1]);
                    let args = try_array!(args[0]);
                    let line_raw = try_array!(args[0]);

                    RedrawEvent::CmdlineBlockAppend((
                        try_u64!(line_raw[0]),
                        try_str!(line_raw[1]).to_string(),
                    ))
                }
                "cmdline_block_hide" => RedrawEvent::CmdlineBlockHide(),
                "wildmenu_show" => {
                    let args = try_array!(args[1]);
                    let items: Vec<String> = try_array!(args[0])
                        .iter()
                        .map(|v| try_str!(v).to_string())
                        .collect();

                    RedrawEvent::WildmenuShow(items)
                }
                "wildmenu_hide" => RedrawEvent::WildmenuHide(),
                "wildmenu_select" => {
                    let args = try_array!(args[1]);
                    let item = try_i64!(args[0]);
                    RedrawEvent::WildmenuSelect(item)
                }
                _ => RedrawEvent::Unknown(cmd.to_string()),
            }
        })
        .collect()
}

fn parse_gnvim_event(args: Vec<Value>) -> GnvimEvent {
    let cmd = try_str!(args[0]);
    match cmd {
        "SetGuiColors" => {
            let mut colors = SetGuiColors::default();

            for e in try_map!(args[1]) {
                let color = Color::from_hex_string(String::from(try_str!(e.1)))
                    .unwrap_or(Color::default());
                match try_str!(e.0) {
                    "pmenu_bg" => colors.pmenu.bg = color,
                    "pmenu_fg" => colors.pmenu.fg = color,
                    "pmenusel_bg" => colors.pmenu.sel_bg = color,
                    "pmenusel_fg" => colors.pmenu.sel_fg = color,

                    "tabline_fg" => colors.tabline.fg = color,
                    "tabline_bg" => colors.tabline.bg = color,
                    "tablinefill_fg" => colors.tabline.fill_fg = color,
                    "tablinefill_bg" => colors.tabline.fill_bg = color,
                    "tablinesel_fg" => colors.tabline.sel_fg = color,
                    "tablinesel_bg" => colors.tabline.sel_bg = color,

                    "cmdline_fg" => colors.cmdline.fg = color,
                    "cmdline_bg" => colors.cmdline.bg = color,
                    "cmdline_border" => colors.cmdline.border = color,

                    "wildmenu_bg" => colors.wildmenu.bg = color,
                    "wildmenu_fg" => colors.wildmenu.fg = color,
                    "wildmenusel_bg" => colors.wildmenu.sel_bg = color,
                    "wildmenusel_fg" => colors.wildmenu.sel_fg = color,
                    _ => {
                        println!("Unknown SetGuiColor: {}", try_str!(e.0));
                    }
                }
            }

            GnvimEvent::SetGuiColors(colors)
        }
        "CompletionMenuToggleInfo" => GnvimEvent::CompletionMenuToggleInfo,
        _ => GnvimEvent::Unknown(String::from("UGH")),
    }
}

fn map_to_hash<'a>(val: &'a Value) -> HashMap<&'a str, &'a Value> {
    let mut h = HashMap::new();
    for (prop, val) in try_map!(val) {
        h.insert(try_str!(prop), val);
    }

    h
}
