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
    if state.contains(gdk::ModifierType::ALT_MASK) || state.contains(gdk::ModifierType::META_MASK) {
        modifier.push_str("M-");
    }
    if state.contains(gdk::ModifierType::SUPER_MASK) {
        modifier.push_str("D-");
    }

    modifier
}

pub fn keyname_to_nvim_key(s: &str) -> Option<&str> {
    // Originally sourced from python-gui.
    match s {
        "asciicircum" => Some("^"), // fix #137
        "semicolon" => Some(";"),
        "equal" => Some("="),
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

/// Turns a gtk key event into a nvim input string, i.e. something that can
/// be passed to `nvim_input`.
///
/// Returns `None` when the event doesn't translate to any nvim input (e.g.
/// unmapped function keys, IM void symbols).
pub fn event_to_nvim_input(keyval: gdk::Key, state: gdk::ModifierType) -> Option<String> {
    let mut modifiers = modifier_to_nvim(&state);
    let keyname = keyval.name()?;

    let key = if keyname.chars().count() > 1 {
        // Named key: try the table first, then fall back to the unicode
        // value. The fallback matters e.g. on macOS, where the IM context
        // doesn't consume plain printable keys, so names like "colon"
        // reach us here.
        match keyname_to_nvim_key(keyname.as_str()) {
            Some(n) => String::from(n),
            None => keyval.to_unicode()?.to_string(),
        }
    } else {
        keyval.to_unicode()?.to_string()
    };

    // Single-char keys are printable characters, multi-char ones are
    // special keys (CR, Tab, Space, F1, BS, ...).
    let is_special = key.chars().count() > 1;

    if !is_special {
        // A printable character already encodes shift (':' vs ';', 'A' vs
        // 'a'); sending <S-:> and the like confuses nvim.
        modifiers = modifiers.replace("S-", "");
    }

    if modifiers.is_empty() && !is_special {
        // Bare printable character: send as-is, mirroring the IM commit
        // path. "<" needs to be escaped for nvim_input (see `:h nvim_input`).
        return Some(if key == "<" {
            String::from("<lt>")
        } else {
            key
        });
    }

    let key = if key == "<" { String::from("lt") } else { key };
    Some(format!("<{}{}>", modifiers, key))
}

#[cfg(test)]
mod tests {
    use gtk::gdk;

    use super::{event_to_nvim_input, modifier_to_nvim};

    #[test]
    fn test_modifier_to_nvim() {
        assert_eq!(&modifier_to_nvim(&gdk::ModifierType::empty()), "");
        assert_eq!(&modifier_to_nvim(&gdk::ModifierType::ALT_MASK), "M-");
        assert_eq!(&modifier_to_nvim(&gdk::ModifierType::META_MASK), "M-");
        assert_eq!(&modifier_to_nvim(&gdk::ModifierType::CONTROL_MASK), "C-");
        assert_eq!(&modifier_to_nvim(&gdk::ModifierType::SUPER_MASK), "D-");

        let mut m = gdk::ModifierType::empty();
        m.set(gdk::ModifierType::META_MASK, true);
        m.set(gdk::ModifierType::ALT_MASK, true);
        assert_eq!(&modifier_to_nvim(&m), "M-");

        let mut m = gdk::ModifierType::empty();
        m.set(gdk::ModifierType::SUPER_MASK, true);
        m.set(gdk::ModifierType::ALT_MASK, true);
        assert_eq!(&modifier_to_nvim(&m), "M-D-");

        let mut m = gdk::ModifierType::empty();
        m.set(gdk::ModifierType::SHIFT_MASK, true);
        m.set(gdk::ModifierType::ALT_MASK, true);
        assert_eq!(&modifier_to_nvim(&m), "S-M-");
    }

    #[test]
    fn test_event_to_nvim_input() {
        let key = |name: &str| gdk::Key::from_name(name).expect("bad key name");
        let none = gdk::ModifierType::empty();
        let shift = gdk::ModifierType::SHIFT_MASK;
        let ctrl = gdk::ModifierType::CONTROL_MASK;

        // Bare printable characters are sent as-is.
        assert_eq!(event_to_nvim_input(key("a"), none).as_deref(), Some("a"));
        // Named printable keys missing from the table fall back to their
        // unicode value (e.g. "colon" on macOS).
        assert_eq!(
            event_to_nvim_input(key("colon"), none).as_deref(),
            Some(":")
        );
        // Shift is dropped for printable characters (it's already encoded
        // in the character itself)...
        assert_eq!(
            event_to_nvim_input(key("colon"), shift).as_deref(),
            Some(":")
        );
        assert_eq!(
            event_to_nvim_input(key("underscore"), shift).as_deref(),
            Some("_")
        );
        assert_eq!(event_to_nvim_input(key("A"), shift).as_deref(), Some("A"));
        // ...but kept for special keys.
        assert_eq!(
            event_to_nvim_input(key("Tab"), shift).as_deref(),
            Some("<S-Tab>")
        );
        // Other modifiers are kept.
        assert_eq!(
            event_to_nvim_input(key("a"), ctrl).as_deref(),
            Some("<C-a>")
        );
        assert_eq!(
            event_to_nvim_input(key("colon"), ctrl).as_deref(),
            Some("<C-:>")
        );
        // "<" is escaped.
        assert_eq!(
            event_to_nvim_input(key("less"), none).as_deref(),
            Some("<lt>")
        );
        assert_eq!(
            event_to_nvim_input(key("less"), ctrl).as_deref(),
            Some("<C-lt>")
        );
        // Special keys go through the table.
        assert_eq!(
            event_to_nvim_input(key("Return"), none).as_deref(),
            Some("<CR>")
        );
        assert_eq!(
            event_to_nvim_input(key("space"), none).as_deref(),
            Some("<Space>")
        );
    }
}
