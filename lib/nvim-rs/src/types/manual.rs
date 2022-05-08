use into_value::IntoValue;

#[derive(Debug)]
pub enum OptionSet {
    Guifont(String),
    Linespace(i64),
    Unknown(String),
}

impl<'de> serde::Deserialize<'de> for OptionSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let data = rmpv::Value::deserialize(d)?;

        let bad_value = || serde::de::Error::missing_field("value");

        let name = data[0]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("bad name"))?;

        match name {
            "linespace" => Ok(Self::Linespace(data[1].as_i64().ok_or_else(bad_value)?)),
            "guifont" => Ok(Self::Guifont(
                data[1].as_str().ok_or_else(bad_value)?.to_string(),
            )),
            _ => Ok(Self::Unknown(name.to_string())),
        }
    }
}

#[derive(Debug, Default, into_value_proc::IntoValue)]
pub struct UiOptions {
    pub rgb: bool,
    pub r#override: bool,
    pub ext_cmdline: bool,
    pub ext_hlstate: bool,
    pub ext_linegrid: bool,
    pub ext_messages: bool,
    pub ext_multigrid: bool,
    pub ext_popupmenu: bool,
    pub ext_tabline: bool,
    pub ext_termcolors: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct HlAttr {
    pub foreground: Option<i64>,
    pub background: Option<i64>,
    pub special: Option<i64>,
    pub reverse: Option<bool>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub underlineline: Option<bool>,
    pub undercurl: Option<bool>,
    pub underdot: Option<bool>,
    pub underdash: Option<bool>,
    pub blend: Option<i64>,
}

#[derive(Debug, Default)]
pub struct GridLineData {
    pub text: String,
    pub hl_id: Option<i64>,
    pub repeat: Option<i64>,
}

impl<'de> serde::Deserialize<'de> for GridLineData {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let data = rmpv::Value::deserialize(d)?;

        let text = data[0]
            .as_str()
            .ok_or_else(|| serde::de::Error::missing_field("text"))?;
        let hl_id = data[1].as_i64();
        let repeat = data[2].as_i64();

        Ok(Self {
            text: text.to_string(),
            hl_id,
            repeat,
        })
    }
}

#[derive(Debug)]
pub enum CursorShape {
    Block,
    Horizontal,
    Vertical,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Block
    }
}

impl<'de> serde::Deserialize<'de> for CursorShape {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let data = rmpv::Value::deserialize(d)?;

        match data.as_str() {
            Some("block") => Ok(Self::Block),
            Some("horizontal") => Ok(Self::Horizontal),
            Some("vertical") => Ok(Self::Vertical),
            Some(v) => Err(serde::de::Error::custom(format!(
                "unknown cursor shape: {}",
                v
            ))),
            None => Err(serde::de::Error::custom("missing value for cursor shape")),
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ModeInfo {
    pub cursor_shape: Option<CursorShape>,
    pub cell_percentage: Option<u64>,
    pub blinkwait: Option<u64>,
    pub blinkon: Option<u64>,
    pub blinkoff: Option<u64>,
    pub attr_id: Option<u64>,
    pub attr_id_lm: Option<u64>,
    pub short_name: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Window(rmpv::Value);

impl IntoValue for Window {
    fn into_value(self) -> rmpv::Value {
        self.0
    }
}
