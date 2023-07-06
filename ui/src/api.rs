use std::collections::HashMap;

use nvim::serde;

use crate::colors;

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
    CursorBlinkTransition(f64),
    CursorPositionTransition(f64),
    ScrollTransition(f64),
    MessageKinds(MessageKinds),
}

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct EchoRepeat {
    pub msg: String,
    pub times: usize,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(crate = "nvim::serde")]
pub struct MessageKinds {
    pub kinds: HashMap<String, MessageKind>,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(crate = "nvim::serde")]
pub struct MessageKind {
    pub label: String,
    pub hl_attr: Option<colors::HlAttr>,
}
