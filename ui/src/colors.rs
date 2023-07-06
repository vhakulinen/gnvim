use std::{collections::HashMap, ops::Deref};

use gtk::gdk;
use nvim::serde;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HlGroup {
    MsgSeparator,
    Pmenu,
    PmenuSel,
    PmenuSbar,
    PmenuThumb,
    TabLine,
    TabLineFill,
    TabLineSel,
    Menu,
}

#[derive(Clone, Debug, Default)]
pub struct Colors {
    pub fg: Color,
    pub bg: Color,
    pub sp: Color,

    pub hls: HashMap<i64, HlAttr>,
    pub hl_groups: HashMap<HlGroup, i64>,
}

impl Colors {
    pub fn get_hl<'a>(&'a self, hl: &i64) -> Highlight<'a> {
        let hl_attr = self.hls.get(hl);

        Highlight {
            colors: self,
            hl_attr,
        }
    }

    pub fn set_hl_group(&mut self, group: HlGroup, hl_id: i64) {
        self.hl_groups.insert(group, hl_id);
    }

    pub fn get_hl_group<'a>(&'a self, group: &HlGroup) -> Highlight<'a> {
        let hl_attr = self.hl_groups.get(group).and_then(|hl| self.hls.get(hl));

        Highlight {
            colors: self,
            hl_attr,
        }
    }
}

pub struct Highlight<'a> {
    colors: &'a Colors,
    hl_attr: Option<&'a HlAttr>,
}

impl<'a> Highlight<'a> {
    pub fn new(colors: &'a Colors, hl_attr: Option<&'a HlAttr>) -> Self {
        Self { colors, hl_attr }
    }

    pub fn fg(&self) -> &Color {
        if self.hl_attr.and_then(|hl| hl.reverse).unwrap_or(false) {
            self.hl_attr
                .and_then(|hl| hl.background.as_ref())
                .unwrap_or(&self.colors.bg)
        } else {
            self.hl_attr
                .and_then(|hl| hl.foreground.as_ref())
                .unwrap_or(&self.colors.fg)
        }
    }

    pub fn bg(&self) -> &Color {
        if self.hl_attr.and_then(|hl| hl.reverse).unwrap_or(false) {
            self.hl_attr
                .and_then(|hl| hl.foreground.as_ref())
                .unwrap_or(&self.colors.fg)
        } else {
            self.hl_attr
                .and_then(|hl| hl.background.as_ref())
                .unwrap_or(&self.colors.bg)
        }
    }

    pub fn sp(&self) -> &Color {
        self.hl_attr
            .and_then(|hl| hl.special.as_ref())
            .unwrap_or(&self.colors.sp)
    }

    pub fn hl_attr(&self) -> Option<&HlAttr> {
        self.hl_attr
    }

    pub fn pango_markup(&self, text: &str) -> String {
        let weight = if self.hl_attr.and_then(|hl| hl.bold).unwrap_or(false) {
            "bold"
        } else {
            "normal"
        };
        let underline = if self.hl_attr.and_then(|hl| hl.undercurl).unwrap_or(false) {
            "error"
        } else if self.hl_attr.and_then(|hl| hl.underline).unwrap_or(false) {
            "single"
        } else if self
            .hl_attr
            .and_then(|hl| hl.underlineline)
            .unwrap_or(false)
        {
            "double"
        } else {
            "none"
        };

        let strikethrough = if self
            .hl_attr
            .and_then(|hl| hl.strikethrough)
            .unwrap_or(false)
        {
            "true"
        } else {
            "false"
        };

        let fontstyle = if self.hl_attr.and_then(|hl| hl.italic).unwrap_or(false) {
            "italic"
        } else {
            "normal"
        };

        format!(
            "<span
            foreground=\"#{fg}\"
            background=\"#{bg}\"
            underline_color=\"#{sp}\"
            strikethrough_color=\"#{sp}\"
            weight=\"{weight}\"
            font_style=\"{fontstyle}\"
            strikethrough=\"{strikethrough}\"
            underline=\"{underline}\">{text}</span>",
            fg = self.fg().as_hex(),
            bg = self.bg().as_hex(),
            sp = self.sp().as_hex(),
            text = glib::markup_escape_text(text)
        )
    }
}

/// Mapping from `nvim::HlAttr` that has the color fields converted to `Color`.
#[derive(Default, Clone, Copy, Debug, serde::Deserialize)]
#[serde(crate = "nvim::serde")]
pub struct HlAttr {
    #[serde(alias = "fg")]
    pub foreground: Option<Color>,
    #[serde(alias = "bg")]
    pub background: Option<Color>,
    #[serde(alias = "sp")]
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
        Self::from(&from)
    }
}
impl From<&nvim::types::HlAttr> for HlAttr {
    fn from(from: &nvim::types::HlAttr) -> Self {
        HlAttr {
            foreground: from.foreground.map(From::from),
            background: from.background.map(From::from),
            special: from.special.map(From::from),
            reverse: from.reverse,
            italic: from.italic,
            bold: from.bold,
            strikethrough: from.strikethrough,
            underline: from.underline,
            underlineline: from.underdouble,
            undercurl: from.undercurl,
            underdot: from.underdotted,
            underdash: from.underdashed,
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

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        i64::deserialize(deserializer).map(|v| Color::from(v))
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
