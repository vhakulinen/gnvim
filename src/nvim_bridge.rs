use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};

use neovim_lib::{neovim_api::Tabpage, Handler, RequestHandler, Value};

use ui::color::{Color, Highlight};

macro_rules! unwrap_str {
    ($val:expr) => {
        $val.as_str().unwrap();
    };
}

macro_rules! unwrap_u64 {
    ($val:expr) => {
        $val.as_u64().unwrap();
    };
}

macro_rules! unwrap_i64 {
    ($val:expr) => {
        $val.as_i64().unwrap();
    };
}

macro_rules! unwrap_array {
    ($val:expr) => {
        $val.as_array().unwrap();
    };
}

macro_rules! unwrap_map {
    ($val:expr) => {
        $val.as_map().unwrap();
    };
}

macro_rules! unwrap_bool {
    ($val:expr) => {
        $val.as_bool().unwrap();
    };
}

macro_rules! try_str {
    ($val:expr, $msg:expr) => {
        $val.as_str()
            .ok_or(format!("Value is not an str: {}", $msg))?
    };
}

macro_rules! try_u64 {
    ($val:expr, $msg:expr) => {
        $val.as_u64()
            .ok_or(format!("Value is not an u64: {}", $msg))?
    };
}

macro_rules! try_map {
    ($val:expr, $msg:expr) => {
        $val.as_map()
            .ok_or(format!("Value is not an map: {}", $msg))?
    };
}

impl Highlight {
    fn from_map_val(map: &Vec<(Value, Value)>) -> Self {
        let mut hl = Highlight::default();
        for (prop, val) in map {
            hl.set(unwrap_str!(prop), val.clone());
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
                self.reverse = unwrap_bool!(val);
            }
            "italic" => {
                self.italic = unwrap_bool!(val);
            }
            "bold" => {
                self.bold = unwrap_bool!(val);
            }
            "underline" => {
                self.underline = unwrap_bool!(val);
            }
            "undercurl" => {
                self.undercurl = unwrap_bool!(val);
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
    /// Redraw event will always get parsed. If something goes wrong there,
    /// we'll panic. Messages are coming from nvim so we should always be
    /// able to parse them.
    RedrawEvent(Vec<RedrawEvent>),
    /// Gnvim event might fail parsing, because user can send basically
    /// anything to the ('Gnvim') channel.
    GnvimEvent(Result<GnvimEvent, String>),
}

#[derive(Clone)]
pub enum CursorShape {
    Block,
    Horizontal,
    Vertical,
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
                self.cursor_shape = CursorShape::from_string(unwrap_str!(val))
            }
            "cell_percentage" => {
                let mut val = unwrap_u64!(val);

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
    pub double_width: bool,
}

pub struct GridLineSegment {
    pub grid: u64,
    pub row: u64,
    pub col_start: u64,
    pub cells: Vec<Cell>,
}

pub enum OptionSet {
    /// Font name.
    GuiFont(String),
    /// Space between lines.
    LineSpace(i64),
    /// Event name.
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
    SetTitle(String),

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
            RedrawEvent::SetTitle(..) => write!(fmt, "SetTitle"),
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

    CursorTooltipLoadStyle(String),
    CursorTooltipShow(String, u64, u64),
    CursorTooltipHide,
    CursorTooltipSetStyle(String),

    PopupmenuWidth(u64),
    PopupmenuWidthDetails(u64),

    Unknown(String),
}

#[derive(Default, Clone, Copy)]
pub struct WildmenuColors {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub sel_bg: Option<Color>,
    pub sel_fg: Option<Color>,
}

#[derive(Default, Clone, Copy)]
pub struct PmenuColors {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub sel_bg: Option<Color>,
    pub sel_fg: Option<Color>,
}

#[derive(Default, Clone, Copy)]
pub struct TablineColors {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub fill_fg: Option<Color>,
    pub fill_bg: Option<Color>,
    pub sel_bg: Option<Color>,
    pub sel_fg: Option<Color>,
}

#[derive(Default, Clone, Copy)]
pub struct CmdlineColors {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub border: Option<Color>,
}

#[derive(Default)]
pub struct SetGuiColors {
    pub pmenu: PmenuColors,
    pub tabline: TablineColors,
    pub cmdline: CmdlineColors,
    pub wildmenu: WildmenuColors,
}

pub enum Request {
    CursorTooltipStyles,
}

/// Message type that we are sending to the UI.
pub enum Message {
    /// RPC notify (see `:h rpcnotify()`).
    Notify(Notify),
    /// RPC Request (see `: rpcrequest()`).
    Request(Sender<Result<Value, Value>>, Request),
}

pub struct NvimBridge {
    /// Channel to send messages to the ui.
    tx: Sender<Message>,

    /// Channel to pass to the UI when we receive a request from nvim.
    /// The UI should send values to this channel when ever it gets a message
    /// Message::Request on its receiving end of `tx`.
    request_tx: Sender<Result<Value, Value>>,
    /// Receiving end of `request_tx`.
    request_rx: Receiver<Result<Value, Value>>,
}

impl NvimBridge {
    pub fn new(tx: Sender<Message>) -> Self {
        let (request_tx, request_rx) = channel();

        NvimBridge {
            tx,
            request_tx,
            request_rx,
        }
    }
}

impl RequestHandler for NvimBridge {
    fn handle_request(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        match name {
            "Gnvim" => match parse_request(args) {
                Ok(msg) => {
                    self.tx
                        .send(Message::Request(self.request_tx.clone(), msg))
                        .unwrap();
                    self.request_rx.recv().unwrap()
                }
                Err(_) => Err("Failed to parse request".into()),
            },
            _ => {
                println!("Unknown request: {}", name);
                Err("Unkown request".into())
            }
        }
    }
}

impl Handler for NvimBridge {
    fn handle_notify(&mut self, name: &str, args: Vec<Value>) {
        if let Some(notify) = parse_notify(name, args) {
            self.tx.send(Message::Notify(notify)).unwrap();
        } else {
            println!("Unknown notify: {}", name);
        }
    }
}

fn parse_request(args: Vec<Value>) -> Result<Request, ()> {
    let cmd = unwrap_str!(args[0]);

    match cmd {
        "CursorTooltipGetStyles" => Ok(Request::CursorTooltipStyles),
        _ => Err(()),
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
            let cmd = unwrap_str!(args[0]);
            match cmd {
                "set_title" => {
                    let args = unwrap_array!(args[1]);
                    let title = unwrap_str!(args[0]);
                    RedrawEvent::SetTitle(title.to_string())
                }
                "grid_line" => {
                    let mut lines = vec![];

                    for entry in unwrap_array!(args)[1..].into_iter() {
                        let entry = unwrap_array!(entry);
                        let grid = unwrap_u64!(entry[0]);
                        let row = unwrap_u64!(entry[1]);
                        let col_start = unwrap_u64!(entry[2]);
                        let mut cells: Vec<Cell> = vec![];

                        for entry in unwrap_array!(entry[3]) {
                            let entry = unwrap_array!(entry);
                            let text = unwrap_str!(entry[0]);
                            let hl_id = if entry.len() >= 2 {
                                entry[1].as_u64()
                            } else {
                                None
                            };
                            let repeat = if entry.len() >= 3 {
                                unwrap_u64!(entry[2])
                            } else {
                                1
                            };

                            let hl_id = if let Some(hl_id) = hl_id {
                                hl_id
                            } else {
                                cells.last().unwrap().hl_id
                            };

                            if text == "" {
                                if let Some(prev) = cells.last_mut() {
                                    prev.double_width = true;
                                }
                            }

                            cells.push(Cell {
                                hl_id,
                                repeat,
                                text: String::from(text),
                                double_width: false,
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
                    let args = unwrap_array!(args[1]);
                    RedrawEvent::GridCursorGoto(
                        unwrap_u64!(args[0]),
                        unwrap_u64!(args[1]),
                        unwrap_u64!(args[2]),
                    )
                }
                "grid_resize" => {
                    let args = unwrap_array!(args[1]);
                    let grid = unwrap_u64!(args[0]);
                    let width = unwrap_u64!(args[1]);
                    let height = unwrap_u64!(args[2]);

                    RedrawEvent::GridResize(grid, width, height)
                }
                "grid_clear" => {
                    let args = unwrap_array!(args[1]);
                    let id = unwrap_u64!(args[0]);
                    RedrawEvent::GridClear(id)
                }
                "grid_scroll" => {
                    let args = unwrap_array!(args[1]);

                    let id = unwrap_u64!(args[0]);
                    let top = unwrap_u64!(args[1]);
                    let bot = unwrap_u64!(args[2]);
                    let left = unwrap_u64!(args[3]);
                    let right = unwrap_u64!(args[4]);
                    let rows = unwrap_i64!(args[5]);
                    let cols = unwrap_i64!(args[6]);

                    RedrawEvent::GridScroll(
                        id,
                        [top, bot, left, right],
                        rows,
                        cols,
                    )
                }
                "default_colors_set" => {
                    let args = unwrap_array!(args[1]);

                    let fg = Color::from_u64(args[0].as_u64().unwrap_or(0));
                    let bg = Color::from_u64(
                        args[1].as_u64().unwrap_or(std::u64::MAX),
                    );
                    // Default to red.
                    let sp =
                        Color::from_u64(args[2].as_u64().unwrap_or(16711680));

                    RedrawEvent::DefaultColorsSet(fg, bg, sp)
                }
                "hl_attr_define" => {
                    let mut hls = vec![];

                    for args in unwrap_array!(args)[1..].into_iter() {
                        let args = unwrap_array!(args);
                        let id = unwrap_u64!(args[0]);
                        let map = unwrap_map!(args[1]);

                        let hl = Highlight::from_map_val(map);

                        hls.push((id, hl));
                    }

                    RedrawEvent::HlAttrDefine(hls)
                }
                "option_set" => {
                    let mut opts = vec![];
                    for arg in unwrap_array!(args)[1..].into_iter() {
                        let name = unwrap_str!(arg[0]);
                        let opt = match name {
                            "guifont" => {
                                let val = unwrap_str!(arg[1]);
                                OptionSet::GuiFont(String::from(val))
                            }
                            "linespace" => {
                                let val = unwrap_i64!(arg[1]);
                                OptionSet::LineSpace(val)
                            }
                            _ => OptionSet::NotSupported(String::from(name)),
                        };

                        opts.push(opt);
                    }

                    RedrawEvent::OptionSet(opts)
                }
                "mode_info_set" => {
                    let args = unwrap_array!(args[1]);
                    let cursor_style_enabled = unwrap_bool!(args[0]);

                    let mut infos = vec![];
                    for info in unwrap_array!(args[1]).into_iter() {
                        let map = unwrap_map!(info);

                        let mut mode = ModeInfo::default();
                        for (prop, val) in map {
                            mode.set(unwrap_str!(prop), val.clone());
                        }

                        infos.push(mode);
                    }

                    RedrawEvent::ModeInfoSet(cursor_style_enabled, infos)
                }
                "mode_change" => {
                    let args = unwrap_array!(args[1]);
                    let name = unwrap_str!(args[0]);
                    let idx = unwrap_u64!(args[1]);
                    RedrawEvent::ModeChange(String::from(name), idx)
                }
                "busy_start" => RedrawEvent::SetBusy(true),
                "busy_stop" => RedrawEvent::SetBusy(false),
                "flush" => RedrawEvent::Flush(),
                "popupmenu_show" => {
                    let args = unwrap_array!(args[1]);
                    let selected = unwrap_i64!(args[1]);
                    let row = unwrap_u64!(args[2]);
                    let col = unwrap_u64!(args[3]);

                    let mut items = vec![];
                    for item in unwrap_array!(args[0]) {
                        let item = unwrap_array!(item);
                        let word = unwrap_str!(item[0]).to_owned();
                        let kind = unwrap_str!(item[1]).to_owned();
                        let menu = unwrap_str!(item[2]).to_owned();
                        let info = unwrap_str!(item[3]).to_owned();

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
                    let args = unwrap_array!(args[1]);
                    let selected = unwrap_i64!(args[0]);
                    RedrawEvent::PopupmenuSelect(selected)
                }
                "tabline_update" => {
                    let args = unwrap_array!(args[1]);
                    let cur_tab = Tabpage::new(args[0].clone());
                    let tabs = unwrap_array!(args[1])
                        .iter()
                        .map(|item| {
                            let m = map_to_hash(&item);
                            (
                                Tabpage::new((*m.get("tab").unwrap()).clone()),
                                unwrap_str!(m.get("name").unwrap()).to_string(),
                            )
                        })
                        .collect();

                    RedrawEvent::TablineUpdate(cur_tab, tabs)
                }
                "cmdline_show" => {
                    let args = unwrap_array!(args[1]);
                    let content: Vec<(u64, String)> = unwrap_array!(args[0])
                        .into_iter()
                        .map(|v| {
                            let hl_id = unwrap_u64!(v[0]);
                            let text = unwrap_str!(v[1]);

                            (hl_id, String::from(text))
                        })
                        .collect();
                    let pos = unwrap_u64!(args[1]);
                    let firstc = String::from(unwrap_str!(args[2]));
                    let prompt = String::from(unwrap_str!(args[3]));
                    let indent = unwrap_u64!(args[4]);
                    let level = unwrap_u64!(args[5]);

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
                    let args = unwrap_array!(args[1]);
                    let pos = unwrap_u64!(args[0]);
                    let level = unwrap_u64!(args[1]);
                    RedrawEvent::CmdlinePos(pos, level)
                }
                "cmdline_special_char" => {
                    let args = unwrap_array!(args[1]);
                    let c = unwrap_str!(args[0]);
                    let shift = unwrap_bool!(args[1]);
                    let level = unwrap_u64!(args[2]);
                    RedrawEvent::CmdlineSpecialChar(c.to_string(), shift, level)
                }
                "cmdline_block_show" => {
                    let args = unwrap_array!(args[1]);
                    let args = unwrap_array!(args[0]);

                    let lines: Vec<(u64, String)> = unwrap_array!(args[0])
                        .iter()
                        .map(|v| {
                            let hl_id = unwrap_u64!(v[0]);
                            let text = unwrap_str!(v[1]);

                            (hl_id, text.to_string())
                        })
                        .collect();

                    RedrawEvent::CmdlineBlockShow(lines)
                }
                "cmdline_block_append" => {
                    let args = unwrap_array!(args[1]);
                    let args = unwrap_array!(args[0]);
                    let line_raw = unwrap_array!(args[0]);

                    RedrawEvent::CmdlineBlockAppend((
                        unwrap_u64!(line_raw[0]),
                        unwrap_str!(line_raw[1]).to_string(),
                    ))
                }
                "cmdline_block_hide" => RedrawEvent::CmdlineBlockHide(),
                "wildmenu_show" => {
                    let args = unwrap_array!(args[1]);
                    let items: Vec<String> = unwrap_array!(args[0])
                        .iter()
                        .map(|v| unwrap_str!(v).to_string())
                        .collect();

                    RedrawEvent::WildmenuShow(items)
                }
                "wildmenu_hide" => RedrawEvent::WildmenuHide(),
                "wildmenu_select" => {
                    let args = unwrap_array!(args[1]);
                    let item = unwrap_i64!(args[0]);
                    RedrawEvent::WildmenuSelect(item)
                }
                _ => RedrawEvent::Unknown(cmd.to_string()),
            }
        })
        .collect()
}

fn parse_gnvim_event(args: Vec<Value>) -> Result<GnvimEvent, String> {
    let cmd = try_str!(args.get(0).ok_or("No command given")?, "cmd");
    let res = match cmd {
        "SetGuiColors" => {
            let mut colors = SetGuiColors::default();

            for e in try_map!(
                args.get(1).ok_or("No data for SetGuiColors")?,
                "colors"
            ) {
                let color = Color::from_hex_string(String::from(try_str!(
                    e.1,
                    "color hex value"
                )))
                .ok();
                match try_str!(e.0, "color name") {
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
                        println!(
                            "Unknown SetGuiColor: {}",
                            try_str!(e.0, "color name")
                        );
                    }
                }
            }

            GnvimEvent::SetGuiColors(colors)
        }
        "CompletionMenuToggleInfo" => GnvimEvent::CompletionMenuToggleInfo,
        "CursorTooltipLoadStyle" => {
            let path =
                try_str!(args.get(1).ok_or("path missing")?, "style file path");
            GnvimEvent::CursorTooltipLoadStyle(path.to_string())
        }
        "CursorTooltipShow" => {
            let content = try_str!(
                args.get(1).ok_or("content missing")?,
                "tooltip content"
            );
            let row =
                try_u64!(args.get(2).ok_or("row missing")?, "tooltip row");
            let col =
                try_u64!(args.get(3).ok_or("col missing")?, "tooltip col");
            GnvimEvent::CursorTooltipShow(content.to_string(), row, col)
        }
        "CursorTooltipHide" => GnvimEvent::CursorTooltipHide,
        "CursorTooltipSetStyle" => {
            let style = try_str!(
                args.get(1).ok_or("path missing")?,
                "tooltip style path"
            );
            GnvimEvent::CursorTooltipSetStyle(style.to_string())
        }
        "PopupmenuSetWidth" => {
            let w =
                try_u64!(args.get(1).ok_or("width missing")?, "pmenu width");
            GnvimEvent::PopupmenuWidth(w)
        }
        "PopupmenuSetWidthDetails" => {
            let w =
                try_u64!(args.get(1).ok_or("width missing")?, "pmenu width");
            GnvimEvent::PopupmenuWidthDetails(w)
        }
        _ => GnvimEvent::Unknown(String::from(cmd)),
    };

    Ok(res)
}

fn map_to_hash<'a>(val: &'a Value) -> HashMap<&'a str, &'a Value> {
    let mut h = HashMap::new();
    for (prop, val) in unwrap_map!(val) {
        h.insert(unwrap_str!(prop), val);
    }

    h
}
