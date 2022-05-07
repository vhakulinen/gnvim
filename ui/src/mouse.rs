#[derive(Clone, Copy)]
pub enum Mouse {
    Left,
    Right,
    Middle,
}

impl Mouse {
    pub fn as_nvim_input(&self) -> &'static str {
        match self {
            Mouse::Left => "left",
            Mouse::Right => "right",
            Mouse::Middle => "middle",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    Pressed,
    Released,
    Drag,
}

impl Action {
    pub fn as_nvim_action(&self) -> &'static str {
        match self {
            Action::Pressed => "press",
            Action::Released => "release",
            Action::Drag => "drag",
        }
    }
}
