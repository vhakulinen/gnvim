use std::fmt::Display;

use super::manual::*;

#[derive(Debug, serde::Deserialize)]
pub struct ModeInfoSet {
    pub enabled: bool,
    pub cursor_styles: Vec<ModeInfo>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ModeChange {
    pub mode: String,
    pub mode_idx: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetTitle {
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetIcon {
    pub icon: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Screenshot {
    pub path: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateFg {
    pub fg: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateBg {
    pub bg: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateSp {
    pub sp: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Resize {
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CursorGoto {
    pub row: i64,
    pub col: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct HighlightSet {
    pub attrs: rmpv::Value, /* Dictionary */
}

#[derive(Debug, serde::Deserialize)]
pub struct Put {
    pub str: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetScrollRegion {
    pub top: i64,
    pub bot: i64,
    pub left: i64,
    pub right: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Scroll {
    pub count: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct DefaultColorsSet {
    pub rgb_fg: i64,
    pub rgb_bg: i64,
    pub rgb_sp: i64,
    pub cterm_fg: i64,
    pub cterm_bg: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct HlAttrDefine {
    pub id: i64,
    pub rgb_attrs: HlAttr,
    pub cterm_attrs: HlAttr,
    pub info: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct HlGroupSet {
    pub name: String,
    pub id: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridResize {
    pub grid: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridClear {
    pub grid: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridCursorGoto {
    pub grid: i64,
    pub row: i64,
    pub col: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridLine {
    pub grid: i64,
    pub row: i64,
    pub col_start: i64,
    pub data: Vec<GridLineData>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridScroll {
    pub grid: i64,
    pub top: i64,
    pub bot: i64,
    pub left: i64,
    pub right: i64,
    pub rows: i64,
    pub cols: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GridDestroy {
    pub grid: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinPos {
    pub grid: i64,
    pub win: Window,
    pub startrow: i64,
    pub startcol: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinFloatPos {
    pub grid: i64,
    pub win: Window,
    pub anchor: String,
    pub anchor_grid: i64,
    pub anchor_row: f64,
    pub anchor_col: f64,
    pub focusable: bool,
    pub zindex: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinExternalPos {
    pub grid: i64,
    pub win: Window,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinHide {
    pub grid: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinClose {
    pub grid: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgSetPos {
    pub grid: i64,
    pub row: i64,
    pub scrolled: bool,
    pub sep_char: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct WinViewport {
    pub grid: i64,
    pub win: Window,
    pub topline: i64,
    pub botline: i64,
    pub curline: i64,
    pub curcol: i64,
    pub line_count: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct PopupmenuShow {
    pub items: Vec<PopupmenuItem>,
    pub selected: i64,
    pub row: i64,
    pub col: i64,
    pub grid: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct PopupmenuSelect {
    pub selected: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct TablineUpdate {
    pub current: Tabpage,
    pub tabs: Vec<TablineTab>,
    pub current_buffer: Buffer,
    pub buffers: Vec<TablineBuffer>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlineShow {
    pub content: rmpv::Value, /* Array */
    pub pos: i64,
    pub firstc: String,
    pub prompt: String,
    pub indent: i64,
    pub level: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlinePos {
    pub pos: i64,
    pub level: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlineSpecialChar {
    pub c: String,
    pub shift: bool,
    pub level: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlineHide {
    pub level: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlineBlockShow {
    pub lines: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct CmdlineBlockAppend {
    pub lines: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct WildmenuShow {
    pub items: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct WildmenuSelect {
    pub selected: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgShow {
    pub kind: String,
    pub content: rmpv::Value, /* Array */
    pub replace_last: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgShowcmd {
    pub content: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgShowmode {
    pub content: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgRuler {
    pub content: rmpv::Value, /* Array */
}

#[derive(Debug, serde::Deserialize)]
pub struct MsgHistoryShow {
    pub entries: rmpv::Value, /* Array */
}

#[derive(Debug)]
pub enum UiEvent {
    ModeInfoSet(Vec<ModeInfoSet>),
    UpdateMenu,
    BusyStart,
    BusyStop,
    MouseOn,
    MouseOff,
    ModeChange(Vec<ModeChange>),
    Bell,
    VisualBell,
    Flush,
    Suspend,
    SetTitle(Vec<SetTitle>),
    SetIcon(Vec<SetIcon>),
    Screenshot(Vec<Screenshot>),
    OptionSet(Vec<OptionSet>),
    UpdateFg(Vec<UpdateFg>),
    UpdateBg(Vec<UpdateBg>),
    UpdateSp(Vec<UpdateSp>),
    Resize(Vec<Resize>),
    Clear,
    EolClear,
    CursorGoto(Vec<CursorGoto>),
    HighlightSet(Vec<HighlightSet>),
    Put(Vec<Put>),
    SetScrollRegion(Vec<SetScrollRegion>),
    Scroll(Vec<Scroll>),
    DefaultColorsSet(Vec<DefaultColorsSet>),
    HlAttrDefine(Vec<HlAttrDefine>),
    HlGroupSet(Vec<HlGroupSet>),
    GridResize(Vec<GridResize>),
    GridClear(Vec<GridClear>),
    GridCursorGoto(Vec<GridCursorGoto>),
    GridLine(Vec<GridLine>),
    GridScroll(Vec<GridScroll>),
    GridDestroy(Vec<GridDestroy>),
    WinPos(Vec<WinPos>),
    WinFloatPos(Vec<WinFloatPos>),
    WinExternalPos(Vec<WinExternalPos>),
    WinHide(Vec<WinHide>),
    WinClose(Vec<WinClose>),
    MsgSetPos(Vec<MsgSetPos>),
    WinViewport(Vec<WinViewport>),
    PopupmenuShow(Vec<PopupmenuShow>),
    PopupmenuHide,
    PopupmenuSelect(Vec<PopupmenuSelect>),
    TablineUpdate(Vec<TablineUpdate>),
    CmdlineShow(Vec<CmdlineShow>),
    CmdlinePos(Vec<CmdlinePos>),
    CmdlineSpecialChar(Vec<CmdlineSpecialChar>),
    CmdlineHide(Vec<CmdlineHide>),
    CmdlineBlockShow(Vec<CmdlineBlockShow>),
    CmdlineBlockAppend(Vec<CmdlineBlockAppend>),
    CmdlineBlockHide,
    WildmenuShow(Vec<WildmenuShow>),
    WildmenuSelect(Vec<WildmenuSelect>),
    WildmenuHide,
    MsgShow(Vec<MsgShow>),
    MsgClear,
    MsgShowcmd(Vec<MsgShowcmd>),
    MsgShowmode(Vec<MsgShowmode>),
    MsgRuler(Vec<MsgRuler>),
    MsgHistoryShow(Vec<MsgHistoryShow>),
}

impl Display for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModeInfoSet(_) => write!(f, "mode_info_set"),
            Self::UpdateMenu => write!(f, "update_menu"),
            Self::BusyStart => write!(f, "busy_start"),
            Self::BusyStop => write!(f, "busy_stop"),
            Self::MouseOn => write!(f, "mouse_on"),
            Self::MouseOff => write!(f, "mouse_off"),
            Self::ModeChange(_) => write!(f, "mode_change"),
            Self::Bell => write!(f, "bell"),
            Self::VisualBell => write!(f, "visual_bell"),
            Self::Flush => write!(f, "flush"),
            Self::Suspend => write!(f, "suspend"),
            Self::SetTitle(_) => write!(f, "set_title"),
            Self::SetIcon(_) => write!(f, "set_icon"),
            Self::Screenshot(_) => write!(f, "screenshot"),
            Self::OptionSet(_) => write!(f, "option_set"),
            Self::UpdateFg(_) => write!(f, "update_fg"),
            Self::UpdateBg(_) => write!(f, "update_bg"),
            Self::UpdateSp(_) => write!(f, "update_sp"),
            Self::Resize(_) => write!(f, "resize"),
            Self::Clear => write!(f, "clear"),
            Self::EolClear => write!(f, "eol_clear"),
            Self::CursorGoto(_) => write!(f, "cursor_goto"),
            Self::HighlightSet(_) => write!(f, "highlight_set"),
            Self::Put(_) => write!(f, "put"),
            Self::SetScrollRegion(_) => write!(f, "set_scroll_region"),
            Self::Scroll(_) => write!(f, "scroll"),
            Self::DefaultColorsSet(_) => write!(f, "default_colors_set"),
            Self::HlAttrDefine(_) => write!(f, "hl_attr_define"),
            Self::HlGroupSet(_) => write!(f, "hl_group_set"),
            Self::GridResize(_) => write!(f, "grid_resize"),
            Self::GridClear(_) => write!(f, "grid_clear"),
            Self::GridCursorGoto(_) => write!(f, "grid_cursor_goto"),
            Self::GridLine(_) => write!(f, "grid_line"),
            Self::GridScroll(_) => write!(f, "grid_scroll"),
            Self::GridDestroy(_) => write!(f, "grid_destroy"),
            Self::WinPos(_) => write!(f, "win_pos"),
            Self::WinFloatPos(_) => write!(f, "win_float_pos"),
            Self::WinExternalPos(_) => write!(f, "win_external_pos"),
            Self::WinHide(_) => write!(f, "win_hide"),
            Self::WinClose(_) => write!(f, "win_close"),
            Self::MsgSetPos(_) => write!(f, "msg_set_pos"),
            Self::WinViewport(_) => write!(f, "win_viewport"),
            Self::PopupmenuShow(_) => write!(f, "popupmenu_show"),
            Self::PopupmenuHide => write!(f, "popupmenu_hide"),
            Self::PopupmenuSelect(_) => write!(f, "popupmenu_select"),
            Self::TablineUpdate(_) => write!(f, "tabline_update"),
            Self::CmdlineShow(_) => write!(f, "cmdline_show"),
            Self::CmdlinePos(_) => write!(f, "cmdline_pos"),
            Self::CmdlineSpecialChar(_) => write!(f, "cmdline_special_char"),
            Self::CmdlineHide(_) => write!(f, "cmdline_hide"),
            Self::CmdlineBlockShow(_) => write!(f, "cmdline_block_show"),
            Self::CmdlineBlockAppend(_) => write!(f, "cmdline_block_append"),
            Self::CmdlineBlockHide => write!(f, "cmdline_block_hide"),
            Self::WildmenuShow(_) => write!(f, "wildmenu_show"),
            Self::WildmenuSelect(_) => write!(f, "wildmenu_select"),
            Self::WildmenuHide => write!(f, "wildmenu_hide"),
            Self::MsgShow(_) => write!(f, "msg_show"),
            Self::MsgClear => write!(f, "msg_clear"),
            Self::MsgShowcmd(_) => write!(f, "msg_showcmd"),
            Self::MsgShowmode(_) => write!(f, "msg_showmode"),
            Self::MsgRuler(_) => write!(f, "msg_ruler"),
            Self::MsgHistoryShow(_) => write!(f, "msg_history_show"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for UiEvent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let redraw = rmpv::Value::deserialize(d)?;

        let name = redraw[0].as_str();
        // TODO(ville): Would be nice if this was possible to do with the derilization it self...
        let params = redraw.as_array().and_then(|v| {
            if v[1].as_array().map(|v| v.is_empty()) == Some(true) {
                None
            } else {
                Some(v[1..].to_vec())
            }
        });

        // TODO(ville): Error handling.
        Ok(match (name, params) {
            (Some("mode_info_set"), Some(params)) => UiEvent::ModeInfoSet({
                params
                    .into_iter()
                    .map(ModeInfoSet::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("update_menu"), None) => UiEvent::UpdateMenu,

            (Some("busy_start"), None) => UiEvent::BusyStart,

            (Some("busy_stop"), None) => UiEvent::BusyStop,

            (Some("mouse_on"), None) => UiEvent::MouseOn,

            (Some("mouse_off"), None) => UiEvent::MouseOff,

            (Some("mode_change"), Some(params)) => UiEvent::ModeChange({
                params
                    .into_iter()
                    .map(ModeChange::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("bell"), None) => UiEvent::Bell,

            (Some("visual_bell"), None) => UiEvent::VisualBell,

            (Some("flush"), None) => UiEvent::Flush,

            (Some("suspend"), None) => UiEvent::Suspend,

            (Some("set_title"), Some(params)) => UiEvent::SetTitle({
                params
                    .into_iter()
                    .map(SetTitle::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("set_icon"), Some(params)) => UiEvent::SetIcon({
                params
                    .into_iter()
                    .map(SetIcon::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("screenshot"), Some(params)) => UiEvent::Screenshot({
                params
                    .into_iter()
                    .map(Screenshot::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("option_set"), Some(params)) => UiEvent::OptionSet({
                params
                    .into_iter()
                    .map(OptionSet::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("update_fg"), Some(params)) => UiEvent::UpdateFg({
                params
                    .into_iter()
                    .map(UpdateFg::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("update_bg"), Some(params)) => UiEvent::UpdateBg({
                params
                    .into_iter()
                    .map(UpdateBg::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("update_sp"), Some(params)) => UiEvent::UpdateSp({
                params
                    .into_iter()
                    .map(UpdateSp::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("resize"), Some(params)) => UiEvent::Resize({
                params
                    .into_iter()
                    .map(Resize::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("clear"), None) => UiEvent::Clear,

            (Some("eol_clear"), None) => UiEvent::EolClear,

            (Some("cursor_goto"), Some(params)) => UiEvent::CursorGoto({
                params
                    .into_iter()
                    .map(CursorGoto::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("highlight_set"), Some(params)) => UiEvent::HighlightSet({
                params
                    .into_iter()
                    .map(HighlightSet::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("put"), Some(params)) => UiEvent::Put({
                params
                    .into_iter()
                    .map(Put::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("set_scroll_region"), Some(params)) => UiEvent::SetScrollRegion({
                params
                    .into_iter()
                    .map(SetScrollRegion::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("scroll"), Some(params)) => UiEvent::Scroll({
                params
                    .into_iter()
                    .map(Scroll::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("default_colors_set"), Some(params)) => UiEvent::DefaultColorsSet({
                params
                    .into_iter()
                    .map(DefaultColorsSet::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("hl_attr_define"), Some(params)) => UiEvent::HlAttrDefine({
                params
                    .into_iter()
                    .map(HlAttrDefine::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("hl_group_set"), Some(params)) => UiEvent::HlGroupSet({
                params
                    .into_iter()
                    .map(HlGroupSet::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_resize"), Some(params)) => UiEvent::GridResize({
                params
                    .into_iter()
                    .map(GridResize::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_clear"), Some(params)) => UiEvent::GridClear({
                params
                    .into_iter()
                    .map(GridClear::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_cursor_goto"), Some(params)) => UiEvent::GridCursorGoto({
                params
                    .into_iter()
                    .map(GridCursorGoto::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_line"), Some(params)) => UiEvent::GridLine({
                params
                    .into_iter()
                    .map(GridLine::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_scroll"), Some(params)) => UiEvent::GridScroll({
                params
                    .into_iter()
                    .map(GridScroll::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("grid_destroy"), Some(params)) => UiEvent::GridDestroy({
                params
                    .into_iter()
                    .map(GridDestroy::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_pos"), Some(params)) => UiEvent::WinPos({
                params
                    .into_iter()
                    .map(WinPos::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_float_pos"), Some(params)) => UiEvent::WinFloatPos({
                params
                    .into_iter()
                    .map(WinFloatPos::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_external_pos"), Some(params)) => UiEvent::WinExternalPos({
                params
                    .into_iter()
                    .map(WinExternalPos::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_hide"), Some(params)) => UiEvent::WinHide({
                params
                    .into_iter()
                    .map(WinHide::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_close"), Some(params)) => UiEvent::WinClose({
                params
                    .into_iter()
                    .map(WinClose::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("msg_set_pos"), Some(params)) => UiEvent::MsgSetPos({
                params
                    .into_iter()
                    .map(MsgSetPos::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("win_viewport"), Some(params)) => UiEvent::WinViewport({
                params
                    .into_iter()
                    .map(WinViewport::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("popupmenu_show"), Some(params)) => UiEvent::PopupmenuShow({
                params
                    .into_iter()
                    .map(PopupmenuShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("popupmenu_hide"), None) => UiEvent::PopupmenuHide,

            (Some("popupmenu_select"), Some(params)) => UiEvent::PopupmenuSelect({
                params
                    .into_iter()
                    .map(PopupmenuSelect::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("tabline_update"), Some(params)) => UiEvent::TablineUpdate({
                params
                    .into_iter()
                    .map(TablineUpdate::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_show"), Some(params)) => UiEvent::CmdlineShow({
                params
                    .into_iter()
                    .map(CmdlineShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_pos"), Some(params)) => UiEvent::CmdlinePos({
                params
                    .into_iter()
                    .map(CmdlinePos::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_special_char"), Some(params)) => UiEvent::CmdlineSpecialChar({
                params
                    .into_iter()
                    .map(CmdlineSpecialChar::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_hide"), Some(params)) => UiEvent::CmdlineHide({
                params
                    .into_iter()
                    .map(CmdlineHide::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_block_show"), Some(params)) => UiEvent::CmdlineBlockShow({
                params
                    .into_iter()
                    .map(CmdlineBlockShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_block_append"), Some(params)) => UiEvent::CmdlineBlockAppend({
                params
                    .into_iter()
                    .map(CmdlineBlockAppend::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("cmdline_block_hide"), None) => UiEvent::CmdlineBlockHide,

            (Some("wildmenu_show"), Some(params)) => UiEvent::WildmenuShow({
                params
                    .into_iter()
                    .map(WildmenuShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("wildmenu_select"), Some(params)) => UiEvent::WildmenuSelect({
                params
                    .into_iter()
                    .map(WildmenuSelect::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("wildmenu_hide"), None) => UiEvent::WildmenuHide,

            (Some("msg_show"), Some(params)) => UiEvent::MsgShow({
                params
                    .into_iter()
                    .map(MsgShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("msg_clear"), None) => UiEvent::MsgClear,

            (Some("msg_showcmd"), Some(params)) => UiEvent::MsgShowcmd({
                params
                    .into_iter()
                    .map(MsgShowcmd::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("msg_showmode"), Some(params)) => UiEvent::MsgShowmode({
                params
                    .into_iter()
                    .map(MsgShowmode::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("msg_ruler"), Some(params)) => UiEvent::MsgRuler({
                params
                    .into_iter()
                    .map(MsgRuler::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            (Some("msg_history_show"), Some(params)) => UiEvent::MsgHistoryShow({
                params
                    .into_iter()
                    .map(MsgHistoryShow::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
            }),

            v => panic!("failed to decode message {:?}", v),
        })
    }
}
