use std::collections::HashMap;

use nvim::types::uievents::HlAttr;

#[derive(Debug, Default)]
pub struct Colors {
    pub fg: Color,
    pub bg: Color,
    pub sp: Color,

    pub hls: HashMap<i64, HlAttr>,
}

impl Colors {
    pub fn get_hl_fg(&self, hl: i64) -> Color {
        if let Some(hl) = self.hls.get(&hl) {
            if hl.reverse.unwrap_or(false) {
                hl.background
                    .map(Color::from_i64)
                    .unwrap_or(self.bg.clone())
            } else {
                hl.foreground
                    .map(Color::from_i64)
                    .unwrap_or(self.fg.clone())
            }
        } else {
            self.fg.clone()
        }
    }

    pub fn get_hl_bg(&self, hl: i64) -> Color {
        if let Some(hl) = self.hls.get(&hl) {
            if hl.reverse.unwrap_or(false) {
                hl.foreground
                    .map(Color::from_i64)
                    .unwrap_or(self.fg.clone())
            } else {
                hl.background
                    .map(Color::from_i64)
                    .unwrap_or(self.bg.clone())
            }
        } else {
            self.bg.clone()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn from_i64(v: i64) -> Self {
        Self {
            r: ((v >> 16) & 255) as f64 / 255f64,
            g: ((v >> 8) & 255) as f64 / 255f64,
            b: (v & 255) as f64 / 255f64,
        }
    }
}
