use glib;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Highlight {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub special: Option<Color>,

    pub reverse: bool,
    pub italic: bool,
    pub bold: bool,
    pub underline: bool,
    pub undercurl: bool,
}

impl Highlight {
    pub fn pango_markup(
        &self,
        text: &str,
        default_fg: &Color,
        default_bg: &Color,
        default_sp: &Color,
    ) -> String {
        let fg = self.foreground.as_ref().unwrap_or(default_fg);
        let bg = self.background.as_ref().unwrap_or(default_bg);
        let sp = self.special.as_ref().unwrap_or(default_sp);

        let weight = if self.bold { "bold" } else { "normal" };
        let underline = if self.undercurl {
            "error"
        } else if self.underline {
            "underline"
        } else {
            "none"
        };

        let fontstyle = if self.italic { "italic" } else { "normal" };

        format!(
            "<span
            foreground=\"#{fg}\"
            background=\"#{bg}\"
            underline_color=\"#{sp}\"
            weight=\"{weight}\"
            font_style=\"{fontstyle}\"
            underline=\"{underline}\">{text}</span>",
            fg = fg.to_hex(),
            bg = bg.to_hex(),
            sp = sp.to_hex(),
            weight = weight,
            fontstyle = fontstyle,
            underline = underline,
            text = glib::markup_escape_text(text)
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn from_hex_string(mut hex: String) -> Result<Color, String> {
        let l = hex.chars().count();
        if l == 7 {
            hex = hex.chars().skip(1).collect();
        } else if l != 6 {
            return Err(String::from("hex string has invalid length"));
        }

        let res = u64::from_str_radix(hex.as_str(), 16);

        if let Ok(res) = res {
            return Ok(Color::from_u64(res));
        } else {
            return Err(format!(
                "Failed to parse hex string '{}': {:?}",
                hex,
                res.err()
            ));
        }
    }

    pub fn from_u64(v: u64) -> Color {
        Color {
            r: ((v >> 16) & 255) as f64 / 255f64,
            g: ((v >> 8) & 255) as f64 / 255f64,
            b: (v & 255) as f64 / 255f64,
        }
    }

    pub fn to_hex(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }
}
