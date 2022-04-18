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
            .ok_or(serde::de::Error::missing_field("text"))?;
        let hl_id = data[1].as_i64();
        let repeat = data[2].as_i64();

        Ok(Self {
            text: text.to_string(),
            hl_id,
            repeat,
        })
    }
}
