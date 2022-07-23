use std::ops::Deref;

#[derive(Debug, Default, Clone, glib::Boxed)]
#[boxed_type(name = "ModeInfo")]
pub struct ModeInfo(pub nvim::types::ModeInfo);

impl Deref for ModeInfo {
    type Target = nvim::types::ModeInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<nvim::types::ModeInfo> for ModeInfo {
    fn from(m: nvim::types::ModeInfo) -> Self {
        Self(m)
    }
}

#[derive(Debug, Default, Clone, glib::Boxed)]
#[boxed_type(name = "TablineShow")]
pub struct ShowTabline(pub nvim::types::ShowTabline);

impl Deref for ShowTabline {
    type Target = nvim::types::ShowTabline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<nvim::types::ShowTabline> for ShowTabline {
    fn from(s: nvim::types::ShowTabline) -> Self {
        Self(s)
    }
}
