use log::{debug, error};

use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::Future;
use gtk::glib;
use nvim_rs::{create::Spawner, neovim::Neovim, Handler, Value};

use crate::nvim_gio::GioWriter;
use crate::thread_guard::ThreadGuard;
use crate::ui::color::{Color, Highlight};

#[cfg(test)]
mod tests;

macro_rules! unwrap_str {
    ($val:expr) => {
        $val.as_str().unwrap()
    };
}

macro_rules! unwrap_u64 {
    ($val:expr) => {
        $val.as_u64().unwrap()
    };
}

macro_rules! unwrap_i64 {
    ($val:expr) => {
        $val.as_i64().unwrap()
    };
}

macro_rules! unwrap_f64 {
    ($val:expr) => {
        $val.as_f64().unwrap()
    };
}

macro_rules! unwrap_array {
    ($val:expr) => {
        $val.as_array().unwrap()
    };
}

macro_rules! unwrap_map {
    ($val:expr) => {
        $val.as_map().unwrap()
    };
}

macro_rules! unwrap_bool {
    ($val:expr) => {
        $val.as_bool().unwrap()
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

impl Highlight {
    fn from_map_val(map: &[(Value, Value)]) -> Self {
        let mut hl = Highlight::default();
        for (prop, val) in map {
            hl.set(unwrap_str!(prop), val.clone());
        }
        hl
    }

    fn set(&mut self, prop: &str, val: Value) {
        match prop {
            "foreground" => {
                self.foreground = val.as_u64().map(Color::from_u64);
            }
            "background" => {
                self.background = val.as_u64().map(Color::from_u64);
            }
            "special" => {
                self.special = val.as_u64().map(Color::from_u64);
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
            "blend" => {
                self.blend = unwrap_f64!(val) / 100.0;
            }
            "cterm_fg" => {}
            "cterm_bg" => {}
            _ => {
                debug!("Unknown highligh property: {}", prop);
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ModeInfo {
    /// The cursor blinking period (in ms)
    pub blink_on: u64,
    pub cursor_shape: CursorShape,
    /// The cursor's width (in percentages, from 0..1).
    pub cell_percentage: f64,
    // TODO(ville): Implement the rest.
}

impl ModeInfo {
    fn set(&mut self, prop: &str, val: Value) {
        match prop {
            "blinkon" => {
                self.blink_on = unwrap_u64!(val);
            }
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

#[derive(Debug, PartialEq)]
pub struct Cell {
    pub text: String,
    pub hl_id: u64,
    pub repeat: u64,
    pub double_width: bool,
}

#[derive(Debug, PartialEq)]
pub enum OptionSet {
    /// Font name.
    GuiFont(String),
    /// Space between lines.
    LineSpace(i64),
    ExtTabline(bool),
    ExtCmdline(bool),
    ExtPopupmenu(bool),
    /// Event name.
    NotSupported(String),
}

impl From<Value> for OptionSet {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let name = unwrap_str!(args[0]);
        match name {
            "guifont" => {
                let val = unwrap_str!(args[1]);
                OptionSet::GuiFont(String::from(val))
            }
            "linespace" => {
                let val = unwrap_i64!(args[1]);
                OptionSet::LineSpace(val)
            }
            "ext_tabline" => OptionSet::ExtTabline(unwrap_bool!(args[1])),
            "ext_cmdline" => OptionSet::ExtCmdline(unwrap_bool!(args[1])),
            "ext_popupmenu" => OptionSet::ExtPopupmenu(unwrap_bool!(args[1])),
            _ => OptionSet::NotSupported(String::from(name)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompletionItemKind {
    Class,
    Color,
    Constant,
    Constructor,
    Enum,
    EnumMember,
    Event,
    Function,
    File,
    Folder,
    Field,
    Interface,
    Keyword,
    Method,
    Module,
    Operator,
    Property,
    Reference,
    Snippet,
    Struct,
    Text,
    TypeParameter,
    Unit,
    Unknown,
    Value,
    Variable,
}

impl From<&str> for CompletionItemKind {
    // Returns CompletionItemKind from a string
    //
    // Lower case kinds are vim-lsp
    //   https://github.com/prabirshrestha/vim-lsp/blob/2b583fefa20b7b1a5e7481a93fb6f1fee67e0846/autoload/lsp/omni.vim#L4-L28
    // Single characters are coc.nvim
    //   https://github.com/neoclide/coc.nvim/blob/909710fddb04d383e5546b0f869c44f395a80d02/src/languages.ts#L143-L167
    // Pascal cased kinds are LanguageClient-neovim
    //   https://github.com/autozimu/LanguageClient-neovim/blob/0ac444affdff8db699684aa4cf04c2cb0daf0286/rplugin/python3/denite/lsp/protocol.py#L48-L55
    fn from(from: &str) -> Self {
        match from {
            "class" | "C" | "Class" => CompletionItemKind::Class,
            "color" => CompletionItemKind::Color,
            "constant" | "Constant" => CompletionItemKind::Constant,
            "constructor" | "Constructor" => CompletionItemKind::Constructor,
            "enum" | "Enum" => CompletionItemKind::Enum,
            "enum member" | "Enum Member" => CompletionItemKind::EnumMember,
            "event" | "E" | "Event" => CompletionItemKind::Event,
            "file" | "F" | "File" => CompletionItemKind::File,
            "field" | "m" | "Field" => CompletionItemKind::Field,
            "folder" | "Folder" => CompletionItemKind::Folder,
            "function" | "Function" => CompletionItemKind::Function,
            "interface" | "I" | "Interface" => CompletionItemKind::Interface,
            "keyword" | "k" | "Key" => CompletionItemKind::Keyword,
            "method" | "f" | "Method" => CompletionItemKind::Method,
            "module" | "M" | "Module" => CompletionItemKind::Module,
            "operator" | "O" | "Operator" => CompletionItemKind::Operator,
            "property" | "Property" => CompletionItemKind::Property,
            "reference" | "r" => CompletionItemKind::Reference,
            "snippet" => CompletionItemKind::Snippet,
            "struct" | "S" | "Struct" => CompletionItemKind::Struct,
            "text" => CompletionItemKind::Text,
            "type parameter" | "T" | "Type Parameter" => {
                CompletionItemKind::TypeParameter
            }
            "unit" | "U" => CompletionItemKind::Unit,
            "value" => CompletionItemKind::Value,
            "variable" | "v" | "Variable" => CompletionItemKind::Variable,
            _ => CompletionItemKind::Unknown,
        }
    }
}

impl CompletionItemKind {
    pub fn is_unknown(&self) -> bool {
        matches!(self, CompletionItemKind::Unknown)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompletionItem {
    pub word: String,
    pub kind: CompletionItemKind,
    pub kind_raw: String,
    pub menu: String,
    pub info: String,
}

#[derive(Debug, PartialEq)]
pub struct PopupmenuShow {
    pub items: Vec<CompletionItem>,
    pub selected: i64,
    pub row: u64,
    pub col: u64,
    pub grid: i64,
}

impl From<Value> for PopupmenuShow {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);

        let selected = unwrap_i64!(args[1]);
        let row = unwrap_u64!(args[2]);
        let col = unwrap_u64!(args[3]);
        let grid = unwrap_i64!(args[4]);

        let mut items = vec![];
        for item in unwrap_array!(args[0]) {
            let item = unwrap_array!(item);
            let word = unwrap_str!(item[0]).to_owned();
            let kind = CompletionItemKind::from(unwrap_str!(item[1]));

            let kind_raw = unwrap_str!(item[1]).to_owned();
            let menu = unwrap_str!(item[2]).to_owned();
            let info = unwrap_str!(item[3]).to_owned();

            items.push(CompletionItem {
                word,
                kind,
                menu,
                info,
                kind_raw,
            });
        }

        PopupmenuShow {
            items,
            selected,
            row,
            col,
            grid,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CmdlineShow {
    pub content: Vec<(u64, String)>,
    pub pos: u64,
    pub firstc: String,
    pub prompt: String,
    pub indent: u64,
    pub level: u64,
}

impl From<Value> for CmdlineShow {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let content: Vec<(u64, String)> = unwrap_array!(args[0])
            .iter()
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

        CmdlineShow {
            content,
            pos,
            firstc,
            prompt,
            indent,
            level,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridLineSegment {
    pub grid: i64,
    pub row: u64,
    pub col_start: u64,
    pub cells: Vec<Cell>,
}

impl From<Value> for GridLineSegment {
    fn from(args: Value) -> Self {
        let entry = unwrap_array!(args);

        let grid = unwrap_i64!(entry[0]);
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

            if text.is_empty() {
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

        GridLineSegment {
            grid,
            row,
            col_start,
            cells,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridResize {
    pub grid: i64,
    pub width: u64,
    pub height: u64,
}

impl From<Value> for GridResize {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        GridResize {
            grid: unwrap_i64!(args[0]),
            width: unwrap_u64!(args[1]),
            height: unwrap_u64!(args[2]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridCursorGoto {
    pub grid: i64,
    pub row: u64,
    pub col: u64,
}

impl From<Value> for GridCursorGoto {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        GridCursorGoto {
            grid: unwrap_i64!(args[0]),
            row: unwrap_u64!(args[1]),
            col: unwrap_u64!(args[2]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridScrollRegion(pub [u64; 4]);

pub struct GridScrollArea {
    pub src_top: f64,
    pub src_bot: f64,
    pub dst_top: f64,
    pub dst_bot: f64,
    pub clr_top: f64,
    pub clr_bot: f64,
}

impl GridScrollRegion {
    pub fn calc_area(&self, count: i64) -> GridScrollArea {
        let top = self.0[0];
        let bot = self.0[1];

        let (src_top, src_bot, dst_top, dst_bot, clr_top, clr_bot) = if count
            > 0
        {
            let (src_top, src_bot) = ((top as i64 + count) as f64, bot as f64);
            let (dst_top, dst_bot) = (top as f64, (bot as i64 - count) as f64);
            (src_top, src_bot, dst_top, dst_bot, dst_bot, src_bot)
        } else {
            let (src_top, src_bot) = (top as f64, (bot as i64 + count) as f64);
            let (dst_top, dst_bot) = ((top as i64 - count) as f64, bot as f64);
            (src_top, src_bot, dst_top, dst_bot, src_top, dst_top)
        };

        GridScrollArea {
            src_top,
            src_bot,
            dst_top,
            dst_bot,
            clr_top,
            clr_bot,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridScroll {
    pub grid: i64,
    pub reg: GridScrollRegion,
    pub rows: i64,
    pub cols: i64,
}

impl From<Value> for GridScroll {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let reg: Vec<u64> = args[1..5].iter().map(|v| unwrap_u64!(v)).collect();
        let reg = GridScrollRegion([reg[0], reg[1], reg[2], reg[3]]);
        GridScroll {
            grid: unwrap_i64!(args[0]),
            reg,
            rows: unwrap_i64!(args[5]),
            cols: unwrap_i64!(args[6]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DefaultColorsSet {
    pub fg: Color,
    pub bg: Color,
    pub sp: Color,
}

impl From<Value> for DefaultColorsSet {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);

        let fg = Color::from_u64(args[0].as_u64().unwrap_or(0));
        let bg = Color::from_u64(args[1].as_u64().unwrap_or(std::u64::MAX));
        // Default to red.
        let sp = Color::from_u64(args[2].as_u64().unwrap_or(16711680));

        DefaultColorsSet { fg, bg, sp }
    }
}

#[derive(Debug, PartialEq)]
pub struct HlAttrDefine {
    pub id: u64,
    pub hl: Highlight,
}

impl From<Value> for HlAttrDefine {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let id = unwrap_u64!(args[0]);
        let map = unwrap_map!(args[1]);

        let hl = Highlight::from_map_val(map);

        HlAttrDefine { id, hl }
    }
}

#[derive(Debug, PartialEq)]
pub struct HlGroupSet {
    pub name: String,
    pub hl_id: u64,
}

impl From<Value> for HlGroupSet {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let name = unwrap_str!(args[0]).to_string();
        let hl_id = unwrap_u64!(args[1]);

        HlGroupSet { name, hl_id }
    }
}

#[derive(Debug, PartialEq)]
pub struct ModeInfoSet {
    pub cursor_shape_enabled: bool,
    pub mode_info: Vec<ModeInfo>,
}

impl From<Value> for ModeInfoSet {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let cursor_shape_enabled = unwrap_bool!(args[0]);

        let mut mode_info = vec![];
        for info in unwrap_array!(args[1]).iter() {
            let map = unwrap_map!(info);

            let mut mode = ModeInfo::default();
            for (prop, val) in map {
                mode.set(unwrap_str!(prop), val.clone());
            }
            mode_info.push(mode);
        }

        ModeInfoSet {
            cursor_shape_enabled,
            mode_info,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ModeChange {
    pub name: String,
    pub index: u64,
}

impl From<Value> for ModeChange {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let name = unwrap_str!(args[0]).to_string();
        let index = unwrap_u64!(args[1]);

        ModeChange { name, index }
    }
}

#[derive(Debug, PartialEq)]
pub struct CmdlinePos {
    pub pos: u64,
    pub level: u64,
}

impl From<Value> for CmdlinePos {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let pos = unwrap_u64!(args[0]);
        let level = unwrap_u64!(args[1]);

        CmdlinePos { pos, level }
    }
}

#[derive(Debug, PartialEq)]
pub struct CmdlineSpecialChar {
    pub character: String,
    pub shift: bool,
    pub level: u64,
}

impl From<Value> for CmdlineSpecialChar {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        let c = unwrap_str!(args[0]);
        let shift = unwrap_bool!(args[1]);
        let level = unwrap_u64!(args[2]);

        CmdlineSpecialChar {
            character: c.to_string(),
            shift,
            level,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CmdlineBlockAppend {
    pub line: Vec<(u64, String)>,
}

impl From<Value> for CmdlineBlockAppend {
    fn from(args: Value) -> Self {
        let line = unwrap_array!(args[0])
            .iter()
            .map(|v| {
                let hl_id = unwrap_u64!(v[0]);
                let text = unwrap_str!(v[1]);

                (hl_id, String::from(text))
            })
            .collect();

        Self { line }
    }
}

#[derive(Debug, PartialEq)]
pub struct TablineUpdate {
    pub current: Value,
    pub tabs: Vec<(Value, String)>,
}

impl From<Value> for TablineUpdate {
    fn from(args: Value) -> Self {
        let current = args[0].clone();
        let tabs = unwrap_array!(args[1])
            .iter()
            .map(|item| {
                let m = map_to_hash(item);
                (
                    (*m.get("tab").unwrap()).clone(),
                    unwrap_str!(m.get("name").unwrap()).to_string(),
                )
            })
            .collect();

        Self { current, tabs }
    }
}

#[derive(Debug, PartialEq)]
pub struct CmdlineBlockShow {
    pub lines: Vec<Vec<(u64, String)>>,
}

impl From<Value> for CmdlineBlockShow {
    fn from(args: Value) -> Self {
        let lines = unwrap_array!(args)
            .iter()
            .map(|line| {
                unwrap_array!(line[0])
                    .iter()
                    .map(|v| {
                        let hl_id = unwrap_u64!(v[0]);
                        let text = unwrap_str!(v[1]);

                        (hl_id, String::from(text))
                    })
                    .collect()
            })
            .collect();

        Self { lines }
    }
}

#[derive(Debug, PartialEq)]
pub struct WindowPos {
    pub grid: i64,
    pub win: Value,
    pub start_row: u64,
    pub start_col: u64,
    pub width: u64,
    pub height: u64,
}

impl From<Value> for WindowPos {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        Self {
            grid: unwrap_i64!(args[0]),
            win: args[1].clone(),
            start_row: unwrap_u64!(args[2]),
            start_col: unwrap_u64!(args[3]),
            width: unwrap_u64!(args[4]),
            height: unwrap_u64!(args[5]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Anchor {
    NW,
    NE,
    SW,
    SE,
}

impl Anchor {
    pub fn is_west(&self) -> bool {
        matches!(self, Self::NW | Self::SW)
    }

    pub fn is_north(&self) -> bool {
        matches!(self, Self::NW | Self::NE)
    }
}

impl From<Value> for Anchor {
    fn from(args: Value) -> Self {
        let args = unwrap_str!(args);
        match args {
            "NW" => Self::NW,
            "NE" => Self::NE,
            "SW" => Self::SW,
            "SE" => Self::SE,
            _ => Self::NW,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WindowFloatPos {
    pub grid: i64,
    pub win: Value,
    pub anchor: Anchor,
    pub anchor_grid: i64,
    pub anchor_row: f64,
    pub anchor_col: f64,
    pub focusable: bool,
}

impl From<Value> for WindowFloatPos {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        Self {
            grid: unwrap_i64!(args[0]),
            win: args[1].clone(),
            anchor: Anchor::from(args[2].clone()),
            anchor_grid: unwrap_i64!(args[3]),
            anchor_row: unwrap_f64!(args[4]),
            anchor_col: unwrap_f64!(args[5]),
            focusable: unwrap_bool!(args[6]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WindowExternalPos {
    pub grid: i64,
    pub win: Value,
}

impl From<Value> for WindowExternalPos {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        Self {
            grid: unwrap_i64!(args[0]),
            win: args[1].clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MsgSetPos {
    pub grid: i64,
    pub row: u64,
    pub scrolled: bool,
    pub sep_char: String,
}

impl From<Value> for MsgSetPos {
    fn from(args: Value) -> Self {
        let args = unwrap_array!(args);
        Self {
            grid: unwrap_i64!(args[0]),
            row: unwrap_u64!(args[1]),
            scrolled: unwrap_bool!(args[2]),
            sep_char: unwrap_str!(args[3]).to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RedrawEvent {
    SetTitle(Vec<String>),

    GridLine(Vec<GridLineSegment>),
    GridResize(Vec<GridResize>),
    GridCursorGoto(Vec<GridCursorGoto>),
    GridClear(Vec<i64>),
    GridDestroy(Vec<i64>),
    GridScroll(Vec<GridScroll>),

    DefaultColorsSet(Vec<DefaultColorsSet>),
    HlAttrDefine(Vec<HlAttrDefine>),
    HlGroupSet(Vec<HlGroupSet>),
    OptionSet(Vec<OptionSet>),
    ModeInfoSet(Vec<ModeInfoSet>),
    ModeChange(Vec<ModeChange>),
    SetBusy(bool),

    Flush(),

    PopupmenuShow(Vec<PopupmenuShow>),
    PopupmenuHide(),
    PopupmenuSelect(Vec<i64>),

    TablineUpdate(Vec<TablineUpdate>),
    CmdlineShow(Vec<CmdlineShow>),
    CmdlineHide(),
    CmdlinePos(Vec<CmdlinePos>),
    CmdlineSpecialChar(Vec<CmdlineSpecialChar>),
    CmdlineBlockShow(Vec<CmdlineBlockShow>),
    CmdlineBlockAppend(Vec<CmdlineBlockAppend>),
    CmdlineBlockHide(),

    WindowPos(Vec<WindowPos>),
    WindowFloatPos(Vec<WindowFloatPos>),
    WindowExternalPos(Vec<WindowExternalPos>),
    WindowHide(Vec<i64>),
    WindowClose(Vec<i64>),
    MsgSetPos(Vec<MsgSetPos>),

    Ignored(String),
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
            RedrawEvent::GridDestroy(..) => write!(fmt, "GridDestroy"),
            RedrawEvent::GridScroll(..) => write!(fmt, "GridScroll"),
            RedrawEvent::DefaultColorsSet(..) => {
                write!(fmt, "DefaultColorsSet")
            }
            RedrawEvent::HlAttrDefine(..) => write!(fmt, "HlAttrDefine"),
            RedrawEvent::HlGroupSet(..) => write!(fmt, "HlGroupSet"),
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

            RedrawEvent::WindowPos(..) => write!(fmt, "WindowPos"),
            RedrawEvent::WindowFloatPos(..) => write!(fmt, "WindowFloatPos"),
            RedrawEvent::WindowExternalPos(..) => {
                write!(fmt, "WindowExternalPos")
            }
            RedrawEvent::WindowHide(..) => write!(fmt, "WindowHide"),
            RedrawEvent::WindowClose(..) => write!(fmt, "WindowClose"),
            RedrawEvent::MsgSetPos(..) => write!(fmt, "MsgSetPos"),

            RedrawEvent::Ignored(..) => write!(fmt, "Ignored"),
            RedrawEvent::Unknown(e) => write!(fmt, "Unknown({})", e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GnvimEvent {
    CompletionMenuToggleInfo,

    PopupmenuWidth(u64),
    PopupmenuWidthDetails(u64),
    PopupmenuShowMenuOnAllItems(bool),

    EnableCursorAnimations(bool),

    EnableExtTabline(bool),
    EnableExtCmdline(bool),
    EnableExtPopupmenu(bool),

    Unknown(String),
}

pub enum Request {}

/// Message type that we are sending to the UI.
pub enum Message {
    /// RPC notify (see `:h rpcnotify()`).
    Notify(Notify),
    /// RPC Request (see `: rpcrequest()`).
    Request(Sender<Result<Value, Value>>, Request),
    /// Nvim went away or reading from the rcp connection failed.
    Close,
}

#[derive(Clone)]
pub struct NvimBridge {
    /// Channel to send messages to the ui.
    tx: Arc<ThreadGuard<glib::Sender<Message>>>,

    /// Channel to pass to the UI when we receive a request from nvim.
    /// The UI should send values to this channel when ever it gets a message
    /// Message::Request on its receiving end of `tx`.
    request_tx: Arc<ThreadGuard<Sender<Result<Value, Value>>>>,
    /// Receiving end of `request_tx`.
    request_rx: Arc<ThreadGuard<Receiver<Result<Value, Value>>>>,
}

impl NvimBridge {
    pub fn new(tx: glib::Sender<Message>) -> Self {
        let (request_tx, request_rx) = channel();

        NvimBridge {
            tx: Arc::new(ThreadGuard::new(tx)),
            request_tx: Arc::new(ThreadGuard::new(request_tx)),
            request_rx: Arc::new(ThreadGuard::new(request_rx)),
        }
    }
}

#[async_trait]
impl Handler for NvimBridge {
    type Writer = GioWriter;

    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        match name.as_str() {
            "Gnvim" => match parse_request(args) {
                Ok(msg) => {
                    let tx = self.tx.borrow_mut();
                    tx.send(Message::Request(
                        self.request_tx.borrow_mut().clone(),
                        msg,
                    ))
                    .unwrap();
                    let rx = self.request_rx.borrow_mut();
                    rx.recv().unwrap()
                }
                Err(_) => Err("Failed to parse request".into()),
            },
            _ => {
                error!("Unknown request: {}", name);
                Err("Unkown request".into())
            }
        }
    }

    async fn handle_notify(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<<Self as Handler>::Writer>,
    ) {
        if let Some(notify) = parse_notify(&name, args) {
            let tx = self.tx.borrow_mut();
            tx.send(Message::Notify(notify)).unwrap();
        } else {
            error!("Unknown notify: {}", name);
        }
    }
}

impl Spawner for NvimBridge {
    type Handle = ();

    fn spawn<Fut>(&self, future: Fut) -> Self::Handle
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let c = glib::MainContext::default();

        c.spawn(future);
    }
}

fn parse_request(_args: Vec<Value>) -> Result<Request, ()> {
    //let cmd = unwrap_str!(args[0]);

    //match cmd {
    //_ => Err(()),
    //}
    Err(())
}

fn parse_notify(name: &str, args: Vec<Value>) -> Option<Notify> {
    match name {
        "redraw" => Some(Notify::RedrawEvent(parse_redraw_event(args))),
        "Gnvim" => Some(Notify::GnvimEvent(parse_gnvim_event(args))),
        _ => None,
    }
}

fn parse_single_redraw_event(cmd: &str, args: Vec<Value>) -> RedrawEvent {
    match cmd {
        "set_title" => RedrawEvent::SetTitle(
            args.into_iter()
                .map(|v| unwrap_str!(v[0]).to_string())
                .collect(),
        ),
        "grid_resize" => RedrawEvent::GridResize(
            args.into_iter().map(GridResize::from).collect(),
        ),
        "grid_cursor_goto" => RedrawEvent::GridCursorGoto(
            args.into_iter().map(GridCursorGoto::from).collect(),
        ),
        "grid_clear" => RedrawEvent::GridClear(
            args.into_iter().map(|v| unwrap_i64!(v[0])).collect(),
        ),
        "grid_destroy" => RedrawEvent::GridDestroy(
            args.into_iter().map(|v| unwrap_i64!(v[0])).collect(),
        ),
        "grid_scroll" => RedrawEvent::GridScroll(
            args.into_iter().map(GridScroll::from).collect(),
        ),
        "grid_line" => RedrawEvent::GridLine(
            args.into_iter().map(GridLineSegment::from).collect(),
        ),
        "default_colors_set" => RedrawEvent::DefaultColorsSet(
            args.into_iter().map(DefaultColorsSet::from).collect(),
        ),
        "hl_attr_define" => RedrawEvent::HlAttrDefine(
            args.into_iter().map(HlAttrDefine::from).collect(),
        ),
        "hl_group_set" => RedrawEvent::HlGroupSet(
            args.into_iter().map(HlGroupSet::from).collect(),
        ),
        "option_set" => RedrawEvent::OptionSet(
            args.into_iter().map(OptionSet::from).collect(),
        ),
        "mode_info_set" => RedrawEvent::ModeInfoSet(
            args.into_iter().map(ModeInfoSet::from).collect(),
        ),
        "mode_change" => RedrawEvent::ModeChange(
            args.into_iter().map(ModeChange::from).collect(),
        ),
        "busy_start" => RedrawEvent::SetBusy(true),
        "busy_stop" => RedrawEvent::SetBusy(false),
        "flush" => RedrawEvent::Flush(),
        "popupmenu_show" => RedrawEvent::PopupmenuShow(
            args.into_iter().map(PopupmenuShow::from).collect(),
        ),
        "popupmenu_hide" => RedrawEvent::PopupmenuHide(),
        "popupmenu_select" => RedrawEvent::PopupmenuSelect(
            args.into_iter().map(|s| unwrap_i64!(s[0])).collect(),
        ),
        "tabline_update" => RedrawEvent::TablineUpdate(
            args.into_iter().map(TablineUpdate::from).collect(),
        ),
        "cmdline_show" => RedrawEvent::CmdlineShow(
            args.into_iter().map(CmdlineShow::from).collect(),
        ),
        "cmdline_hide" => RedrawEvent::CmdlineHide(),
        "cmdline_pos" => RedrawEvent::CmdlinePos(
            args.into_iter().map(CmdlinePos::from).collect(),
        ),
        "cmdline_special_char" => RedrawEvent::CmdlineSpecialChar(
            args.into_iter().map(CmdlineSpecialChar::from).collect(),
        ),
        "cmdline_block_show" => RedrawEvent::CmdlineBlockShow(
            args.into_iter().map(CmdlineBlockShow::from).collect(),
        ),
        "cmdline_block_append" => RedrawEvent::CmdlineBlockAppend(
            args.into_iter().map(CmdlineBlockAppend::from).collect(),
        ),
        "cmdline_block_hide" => RedrawEvent::CmdlineBlockHide(),
        "win_pos" => RedrawEvent::WindowPos(
            args.into_iter().map(WindowPos::from).collect(),
        ),
        "win_float_pos" => RedrawEvent::WindowFloatPos(
            args.into_iter().map(WindowFloatPos::from).collect(),
        ),
        "win_external_pos" => RedrawEvent::WindowExternalPos(
            args.into_iter().map(WindowExternalPos::from).collect(),
        ),
        "win_hide" => RedrawEvent::WindowHide(
            args.into_iter()
                .map(|v| {
                    let v = unwrap_array!(v);
                    unwrap_i64!(v[0])
                })
                .collect(),
        ),
        "win_close" => RedrawEvent::WindowClose(
            args.into_iter()
                .map(|v| {
                    let v = unwrap_array!(v);
                    unwrap_i64!(v[0])
                })
                .collect(),
        ),
        "msg_set_pos" => RedrawEvent::MsgSetPos(
            args.into_iter().map(MsgSetPos::from).collect(),
        ),

        "mouse_on" | "mouse_off" => RedrawEvent::Ignored(cmd.to_string()),
        _ => RedrawEvent::Unknown(cmd.to_string()),
    }
}

pub(crate) fn parse_redraw_event(args: Vec<Value>) -> Vec<RedrawEvent> {
    args.into_iter()
        .map(|args| {
            let args = unwrap_array!(args);
            let cmd = unwrap_str!(args[0]);
            parse_single_redraw_event(cmd, args[1..].to_vec())
        })
        .collect()
}

pub(crate) fn parse_gnvim_event(
    args: Vec<Value>,
) -> Result<GnvimEvent, String> {
    let cmd = try_str!(args.get(0).ok_or("No command given")?, "cmd");
    let res = match cmd {
        "CompletionMenuToggleInfo" => GnvimEvent::CompletionMenuToggleInfo,
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
        "PopupmenuShowMenuOnAllItems" => {
            let b = try_u64!(
                args.get(1).ok_or("bool missing")?,
                "pmenu show menu on all items"
            );

            GnvimEvent::PopupmenuShowMenuOnAllItems(b != 0)
        }
        "EnableCursorAnimations" => GnvimEvent::EnableCursorAnimations(
            try_u64!(
                args.get(1).ok_or("argument missing")?,
                "failed to parse enable cursor animations argument"
            ) == 1,
        ),
        "EnableExtTabline" => GnvimEvent::EnableExtTabline(
            try_u64!(
                args.get(1).ok_or("argument missing")?,
                "failed to parse enable ext tabline argument"
            ) == 1,
        ),
        "EnableExtCmdline" => GnvimEvent::EnableExtCmdline(
            try_u64!(
                args.get(1).ok_or("argument missing")?,
                "failed to parse enable ext cmdline argument"
            ) == 1,
        ),
        "EnableExtPopupmenu" => GnvimEvent::EnableExtPopupmenu(
            try_u64!(
                args.get(1).ok_or("argument missing")?,
                "failed to parse enable ext popupmenu argument"
            ) == 1,
        ),
        _ => GnvimEvent::Unknown(String::from(cmd)),
    };

    Ok(res)
}

fn map_to_hash(val: &Value) -> HashMap<&str, &Value> {
    let mut h = HashMap::new();
    for (prop, val) in unwrap_map!(val) {
        h.insert(unwrap_str!(prop), val);
    }

    h
}
