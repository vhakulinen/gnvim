use std::{collections::HashMap, ops::Deref};

use gtk::glib;
use nvim::serde;

#[derive(Debug, serde::Deserialize)]
#[serde(
    crate = "nvim::serde",
    rename_all = "snake_case",
    tag = "fn",
    content = "args"
)]
pub enum GnvimEvent {
    // NOTE(ville): Demo event.
    EchoRepeat(EchoRepeat),
    GtkDebugger,
    Setup(Setup),

    FontSize(FontSize),
}

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct FontSize {
    pub increment: f32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct EchoRepeat {
    pub msg: String,
    pub times: usize,
}

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct Setup {
    #[serde(default)]
    pub cursor: Cursor,
    #[serde(default)]
    pub scroll_transition: ScrollTransition,
    #[serde(default)]
    pub popupmenu: Popupmenu,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct Popupmenu {
    #[serde(default)]
    pub kinds: HashMap<String, PopupmenuKind>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct PopupmenuKind {
    pub label: Option<String>,
    pub hl: Option<HlAttr>,
    pub sel_hl: Option<HlAttr>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct Cursor {
    #[serde(default)]
    pub blink_transition: CursorBlinkTransition,
    #[serde(default)]
    pub position_transition: CursorPositionTransition,
}

/// Limited custom HlAttr that the gnvim API supports.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct HlAttr {
    #[serde(default)]
    pub fg: Option<i64>,
    #[serde(default)]
    pub bg: Option<i64>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
}

macro_rules! defaulted_f64 {
    ($name:ident, $default:literal) => {
        #[derive(Debug, serde::Deserialize, glib::ValueDelegate)]
        #[serde(crate = "nvim::serde")]
        pub struct $name(f64);

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        impl Deref for $name {
            type Target = f64;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

defaulted_f64!(CursorBlinkTransition, 160.0);
defaulted_f64!(CursorPositionTransition, 150.0);
defaulted_f64!(ScrollTransition, 300.0);
