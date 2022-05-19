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
