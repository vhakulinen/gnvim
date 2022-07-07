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
    CursorBlinkTransition(f64),
}

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct EchoRepeat {
    pub msg: String,
    pub times: usize,
}
