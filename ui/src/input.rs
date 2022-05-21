use gtk::{gdk, prelude::*};

/// Mapping between gtk mouse events and nvim mouse events.
#[derive(Debug, Clone, Copy)]
pub enum Mouse {
    Left,
    Right,
    Middle,
    Wheel,
}

impl Mouse {
    pub fn as_nvim_input(&self) -> &'static str {
        match self {
            Mouse::Left => "left",
            Mouse::Right => "right",
            Mouse::Middle => "middle",
            Mouse::Wheel => "wheel",
        }
    }
}

impl<T> From<&T> for Mouse
where
    T: IsA<gtk::GestureSingle>,
{
    fn from(gst: &T) -> Self {
        match gst.current_button() {
            gdk::BUTTON_PRIMARY => Mouse::Left,
            gdk::BUTTON_SECONDARY => Mouse::Right,
            gdk::BUTTON_MIDDLE => Mouse::Middle,
            _ => {
                println!("unknown button, defaulting to primary");
                Mouse::Left
            }
        }
    }
}

/// Mapping between gtk mouse actions and nvim mouse actions.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Pressed,
    Released,
    Drag,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

impl Action {
    pub fn as_nvim_action(&self) -> &'static str {
        match self {
            Action::Pressed => "press",
            Action::Released => "release",
            Action::Drag => "drag",
            Action::ScrollUp => "up",
            Action::ScrollDown => "down",
            Action::ScrollLeft => "left",
            Action::ScrollRight => "right",
        }
    }
}

/// Turns gtk modifier to nvim input's modifier prefix.
pub fn modifier_to_nvim(state: &gdk::ModifierType) -> String {
    let mut modifier = String::new();

    if state.contains(gdk::ModifierType::SHIFT_MASK) {
        modifier.push_str("S-");
    }
    if state.contains(gdk::ModifierType::CONTROL_MASK) {
        modifier.push_str("C-");
    }
    if state.contains(gdk::ModifierType::ALT_MASK) {
        modifier.push_str("A-");
    }
    if state.contains(gdk::ModifierType::META_MASK) {
        modifier.push_str("M-");
    }

    modifier
}

pub fn keyname_to_nvim_key(s: &str) -> Option<&str> {
    // Originally sourced from python-gui.
    match s {
        "asciicircum" => Some("^"), // fix #137
        "slash" => Some("/"),
        "backslash" => Some("\\"),
        "dead_circumflex" => Some("^"),
        "at" => Some("@"),
        "numbersign" => Some("#"),
        "dollar" => Some("$"),
        "percent" => Some("%"),
        "ampersand" => Some("&"),
        "asterisk" => Some("*"),
        "parenleft" => Some("("),
        "parenright" => Some(")"),
        "underscore" => Some("_"),
        "plus" => Some("+"),
        "minus" => Some("-"),
        "bracketleft" => Some("["),
        "bracketright" => Some("]"),
        "braceleft" => Some("{"),
        "braceright" => Some("}"),
        "dead_diaeresis" => Some("\""),
        "dead_acute" => Some("\'"),
        "less" => Some("<"),
        "greater" => Some(">"),
        "comma" => Some(","),
        "period" => Some("."),
        "space" => Some("Space"),
        "BackSpace" => Some("BS"),
        "Insert" => Some("Insert"),
        "Return" => Some("CR"),
        "Escape" => Some("Esc"),
        "Delete" => Some("Del"),
        "Page_Up" => Some("PageUp"),
        "Page_Down" => Some("PageDown"),
        "Enter" => Some("CR"),
        "ISO_Left_Tab" => Some("Tab"),
        "Tab" => Some("Tab"),
        "Up" => Some("Up"),
        "Down" => Some("Down"),
        "Left" => Some("Left"),
        "Right" => Some("Right"),
        "Home" => Some("Home"),
        "End" => Some("End"),
        "F1" => Some("F1"),
        "F2" => Some("F2"),
        "F3" => Some("F3"),
        "F4" => Some("F4"),
        "F5" => Some("F5"),
        "F6" => Some("F6"),
        "F7" => Some("F7"),
        "F8" => Some("F8"),
        "F9" => Some("F9"),
        "F10" => Some("F10"),
        "F11" => Some("F11"),
        "F12" => Some("F12"),
        _ => None,
    }
}
