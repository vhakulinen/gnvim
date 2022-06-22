use std::{collections::HashMap, ops::Deref};

use gtk::gdk;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HlGroup {
    MsgSeparator,
    Pmenu,
    PmenuSel,
    PmenuSbar,
    PmenuThumb,
}

#[derive(Debug, Default)]
pub struct Colors {
    pub fg: Color,
    pub bg: Color,
    pub sp: Color,

    pub hls: HashMap<i64, HlAttr>,
    pub hl_groups: HashMap<HlGroup, i64>,
}

impl Colors {
    pub fn get_hl(&self, hl: &i64) -> Option<&HlAttr> {
        self.hls.get(hl)
    }

    pub fn set_hl_group(&mut self, group: HlGroup, hl_id: i64) {
        self.hl_groups.insert(group, hl_id);
    }

    pub fn get_hl_group_fg(&self, group: &HlGroup) -> &Color {
        self.hl_groups
            .get(group)
            .map(|hl| self.get_hl_fg(hl))
            .unwrap_or(&self.fg)
    }

    pub fn get_hl_group_bg(&self, group: &HlGroup) -> &Color {
        self.hl_groups
            .get(group)
            .map(|hl| self.get_hl_bg(hl))
            .unwrap_or(&self.bg)
    }

    pub fn get_hl_fg(&self, hl: &i64) -> &Color {
        self.hls
            .get(hl)
            .map(|hl| {
                if hl.reverse.unwrap_or(false) {
                    hl.background.as_ref().unwrap_or(&self.bg)
                } else {
                    hl.foreground.as_ref().unwrap_or(&self.fg)
                }
            })
            .unwrap_or(&self.fg)
    }

    pub fn get_hl_bg(&self, hl: &i64) -> &Color {
        self.hls
            .get(hl)
            .map(|hl| {
                if hl.reverse.unwrap_or(false) {
                    hl.foreground.as_ref().unwrap_or(&self.fg)
                } else {
                    hl.background.as_ref().unwrap_or(&self.bg)
                }
            })
            .unwrap_or(&self.bg)
    }

    pub fn get_hl_sp(&self, hl: &i64) -> &Color {
        self.hls
            .get(hl)
            .and_then(|hl| hl.special.as_ref())
            .unwrap_or(&self.sp)
    }
}

#[derive(Debug)]
pub struct HlAttr {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub special: Option<Color>,
    pub reverse: Option<bool>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub underlineline: Option<bool>,
    pub undercurl: Option<bool>,
    pub underdot: Option<bool>,
    pub underdash: Option<bool>,
    pub blend: Option<Color>,
}

impl From<nvim::types::HlAttr> for HlAttr {
    fn from(from: nvim::types::HlAttr) -> Self {
        HlAttr {
            foreground: from.foreground.map(From::from),
            background: from.background.map(From::from),
            special: from.special.map(From::from),
            reverse: from.reverse,
            italic: from.italic,
            bold: from.bold,
            strikethrough: from.strikethrough,
            underline: from.underline,
            underlineline: from.underline,
            undercurl: from.undercurl,
            underdot: from.underdot,
            underdash: from.underdash,
            blend: from.blend.map(From::from),
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

impl From<i64> for Color {
    fn from(from: i64) -> Self {
        Self::from_i64(from)
    }
}

impl Deref for Color {
    type Target = gdk::RGBA;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
