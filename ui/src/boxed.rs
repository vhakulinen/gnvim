use std::ops::Deref;

use gtk::graphene;

use crate::SCALE;

#[derive(Debug, Default, Clone, glib::Boxed)]
#[boxed_type(name = "CursorShape")]
pub struct CursorShape(pub nvim::types::CursorShape);

impl CursorShape {
    /// Get the "size" rect for rendering.
    pub fn cell_rect(&self, height: f32, width: f32, percentage: f32) -> graphene::Rect {
        let width = (width / SCALE)
            * match self.0 {
                nvim::types::CursorShape::Vertical => percentage,
                _ => 1.0,
            };

        let height = height / SCALE;
        // For Horizontal cursor, we'll need to adjust our y so that we dont
        // render our cursor to the top of the cell,
        let (y, height) = match self.0 {
            nvim::types::CursorShape::Horizontal => {
                let h = height * percentage;
                (height - h, h)
            }
            _ => (0.0, height),
        };

        graphene::Rect::new(0.0, y, width, height)
    }
}

impl Deref for CursorShape {
    type Target = nvim::types::CursorShape;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<nvim::types::CursorShape> for CursorShape {
    fn from(value: nvim::types::CursorShape) -> Self {
        Self(value)
    }
}

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

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "Tabpage")]
pub struct Tabpage(pub nvim::types::Tabpage);

impl Deref for Tabpage {
    type Target = nvim::types::Tabpage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<nvim::types::Tabpage> for Tabpage {
    fn from(s: nvim::types::Tabpage) -> Self {
        Self(s)
    }
}
