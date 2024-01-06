use std::ops::Deref;

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
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct Cursor {
    #[serde(default)]
    pub blink_transition: CursorBlinkTransition,
    #[serde(default)]
    pub position_transition: CursorPositionTransition,
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
