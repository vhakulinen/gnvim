use std::{collections::HashMap, ops::Deref};

use gtk::gdk;

use nvim::types::HlAttr;

#[derive(Debug, Default)]
pub struct HlGroups {
    msg_separator: Option<i64>,
}

#[derive(Debug, Default)]
pub struct Colors {
    pub fg: Color,
    pub bg: Color,
    pub sp: Color,

    pub hls: HashMap<i64, HlAttr>,
    pub hl_groups: HlGroups,
}

impl Colors {
    pub fn get_hl(&self, hl: i64) -> Option<&HlAttr> {
        self.hls.get(&hl)
    }

    pub fn set_msg_separator(&mut self, hl: i64) {
        self.hl_groups.msg_separator = Some(hl);
    }

    pub fn msg_separator_fg(&self) -> Color {
        self.hl_groups
            .msg_separator
            .and_then(|id| {
                self.hls
                    .get(&id)
                    .and_then(|hl| hl.foreground)
                    .map(Color::from_i64)
            })
            .unwrap_or(self.fg)
    }

    pub fn get_hl_fg(&self, hl: i64) -> Color {
        if let Some(hl) = self.hls.get(&hl) {
            if hl.reverse.unwrap_or(false) {
                hl.background.map(Color::from_i64).unwrap_or(self.bg)
            } else {
                hl.foreground.map(Color::from_i64).unwrap_or(self.fg)
            }
        } else {
            self.fg
        }
    }

    pub fn get_hl_bg(&self, hl: i64) -> Color {
        if let Some(hl) = self.hls.get(&hl) {
            if hl.reverse.unwrap_or(false) {
                hl.foreground.map(Color::from_i64).unwrap_or(self.fg)
            } else {
                hl.background.map(Color::from_i64).unwrap_or(self.bg)
            }
        } else {
            self.bg
        }
    }

    pub fn get_hl_sp(&self, hl: i64) -> Color {
        if let Some(hl) = self.hls.get(&hl) {
            hl.special.map(Color::from_i64).unwrap_or(self.sp)
        } else {
            self.sp
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color(gdk::RGBA);

impl Default for Color {
    fn default() -> Self {
        Self(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
    }
}

impl Color {
    pub fn from_i64(v: i64) -> Self {
        Self(gdk::RGBA::new(
            ((v >> 16) & 255) as f32 / 255f32,
            ((v >> 8) & 255) as f32 / 255f32,
            (v & 255) as f32 / 255f32,
            1.0,
        ))
    }

    pub fn as_hex(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}",
            (self.red() * 255.0) as u8,
            (self.green() * 255.0) as u8,
            (self.blue() * 255.0) as u8
        )
    }
}

impl Deref for Color {
    type Target = gdk::RGBA;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
