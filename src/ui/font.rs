use std::fmt;
use std::fmt::Display;

const DEFAULT_HEIGHT: usize = 14;

pub enum FontUnit {
    Pixel,
    Point,
}

impl Display for FontUnit {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontUnit::Pixel => write!(fmt, "px"),
            FontUnit::Point => write!(fmt, "pt"),
        }
    }
}

#[derive(Clone)]
pub struct Font {
    name: String,
    height: usize,
}

impl Font {
    /// Parses nvim `guifont` option.
    ///
    /// If invalid height is specified, defaults to `DEFAULT_HEIGHT`.
    pub fn from_guifont(guifont: &str) -> Result<Self, ()> {
        let mut parts = guifont.split(":").into_iter();

        let name = parts.next().ok_or(())?;

        let mut font = Font {
            name: name.to_string(),
            height: DEFAULT_HEIGHT,
        };

        while let Some(part) = parts.next() {
            let mut chars = part.chars().into_iter();
            if let Some(ch) = chars.next() {
                match ch {
                    'h' => {
                        let rest = chars.collect::<String>();
                        let h = rest.parse::<usize>().or(Err(()))?;
                        if h == 0 {
                            // Ignore zero sized font.
                            continue;
                        }
                        font.height = h;
                    }
                    _ => {
                        println!("Not supported guifont option: {}", part);
                    }
                }
            }
        }

        Ok(font)
    }

    /// Returns a CSS representation of self for a wild (`*`) CSS selector.
    /// On gtk version below 3.20 unit needs to be `FontUnit::Pixel` and
    /// with version 3.20 and up, unit needs to be `FontUnit::Point`. This is
    /// to work around some gtk issues on versions before 3.20.
    pub fn as_wild_css(&self, unit: FontUnit) -> String {
        format!(
            "* {{ \
             font-family: \"{font_family}\"; \
             font-size: {font_size}{font_unit}; \
             }}",
            font_family = self.name,
            font_size = self.height,
            font_unit = unit,
        )
    }

    /// Returns a pango::FontDescription version of self.
    pub fn as_pango_font(&self) -> pango::FontDescription {
        let mut font_desc = pango::FontDescription::from_string(&format!(
            "{} {}",
            self.name, self.height
        ));

        // Make sure we dont have a font with size of 0, otherwise we'll
        // have problems later.
        if font_desc.get_size() == 0 {
            font_desc.set_size(DEFAULT_HEIGHT as i32 * pango::SCALE);
        }

        font_desc
    }
}

impl Default for Font {
    fn default() -> Self {
        Font {
            name: String::from("Monospace"),
            height: DEFAULT_HEIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_wild_css() {
        let font = Font {
            name: "foo".to_string(),
            height: 10,
        };

        assert_eq!(
            font.as_wild_css(FontUnit::Point),
            "* { \
             font-family: \"foo\"; \
             font-size: 10pt; \
             }"
        );

        assert_eq!(
            font.as_wild_css(FontUnit::Pixel),
            "* { \
             font-family: \"foo\"; \
             font-size: 10px; \
             }"
        );
    }

    #[test]
    fn test_from_guifont() {
        // Font with proper height.
        let f = Font::from_guifont("monospace:h11").unwrap();
        assert_eq!(f.name, "monospace");
        assert_eq!(f.height, 11);

        // Font with invalid height.
        let f = Font::from_guifont("font:h");
        assert_eq!(f.is_err(), true);
        let f = Font::from_guifont("font:hn");
        assert_eq!(f.is_err(), true);
        let f = Font::from_guifont("font:h-1");
        assert_eq!(f.is_err(), true);

        // Font with height zero.
        let f = Font::from_guifont("foo:h0").unwrap();
        assert_eq!(f.name, "foo");
        assert_eq!(f.height, DEFAULT_HEIGHT);

        // Font with no hegith.
        let f = Font::from_guifont("bar").unwrap();
        assert_eq!(f.name, "bar");
        assert_eq!(f.height, DEFAULT_HEIGHT);
    }
}
