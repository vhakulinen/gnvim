
#[derive(Debug, Clone, Copy, Default)]
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

#[derive(Debug, Clone, Copy, Default)]
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
            return Err(format!("Failed to parse hex string '{}': {:?}", hex, res.err()));
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
        format!("{:02x}{:02x}{:02x}",
                (self.r * 255.0) as u8,
                (self.g * 255.0) as u8,
                (self.b * 255.0) as u8)
    }
}
