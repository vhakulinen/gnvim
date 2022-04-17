pub trait AsPascalCase {
    fn as_pascal_case(&self) -> String;
}

impl<T: AsRef<str>> AsPascalCase for T {
    fn as_pascal_case(&self) -> String {
        let mut upper = false;
        let mut out = String::with_capacity(self.as_ref().chars().count());
        for (i, ch) in self.as_ref().chars().enumerate() {
            if i == 0 {
                out.push(ch.to_ascii_uppercase());
                continue;
            }

            if ch == '_' {
                upper = true;
                continue;
            }

            if upper {
                out.push(ch.to_ascii_uppercase());
            } else {
                out.push(ch)
            }

            upper = false;
        }

        out
    }
}

pub trait AsRustType {
    fn as_rust_type(&self) -> String;
}

impl<T: AsRef<str>> AsRustType for T {
    fn as_rust_type(&self) -> String {
        let f = match self.as_ref() {
            "Boolean" => "bool",
            "Integer" => "i64",
            "Float" => "f64",
            "String" => "String",
            "void" => "()",
            "ArrayOf(Integer, 2)" => "(i64, i64)",
            "ArrayOf(String)" => "Vec<String>",
            "ArrayOf(Integer)" => "Vec<i64>",
            "ArrayOf(Buffer)" => return format!("Vec<{}>", "Buffer".as_rust_type()),
            "ArrayOf(Dictionary)" => return format!("Vec<{}>", "Dictionary".as_rust_type()),
            "ArrayOf(Tabpage)" => return format!("Vec<{}>", "Tabpage".as_rust_type()),
            "ArrayOf(Window)" => return format!("Vec<{}>", "Window".as_rust_type()),
            s => return format!("rmpv::Value /* {} */", s),
        };

        f.into()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Parameter {
    pub r#type: String,
    pub name: String,
}

impl Parameter {
    pub fn rust_name(&self) -> &str {
        match self.name.as_str() {
            "fn" => "_fn",
            "type" => "_type",
            s => s,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Function {
    pub name: String,
    pub since: i64,
    pub deprecated_since: Option<i64>,
    pub parameters: Vec<Parameter>,

    pub return_type: String,
    pub method: bool,
}

impl Function {
    pub fn rust_type_for_param(&self, param: &Parameter) -> String {
        //println!("{} {}", self.name, param.name);
        let t = match (self.name.as_ref(), param.name.as_ref()) {
            ("nvim_ui_attach", "options") => "UiOptions",
            _ => return param.r#type.as_rust_type(),
        };

        t.into()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ExtType {
    pub id: i64,
    pub prefix: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Types {
    #[serde(rename = "Buffer")]
    pub buffer: ExtType,
    #[serde(rename = "Window")]
    pub window: ExtType,
    #[serde(rename = "Tabpage")]
    pub tabpage: ExtType,
}

#[derive(Debug, serde::Deserialize)]
pub struct ExtErrorType {
    pub id: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct ErrorTypes {
    #[serde(rename = "Exception")]
    pub exception: ExtErrorType,
    #[serde(rename = "Validation")]
    pub validation: ExtErrorType,
}

#[derive(Debug, serde::Deserialize)]
pub struct UiEvent {
    pub parameters: Vec<Parameter>,
    pub name: String,
    pub since: i64,
}

impl UiEvent {
    pub fn parameter_type_for<S: AsRef<str>>(evt: S, param: S, _type: S) -> String {
        let s: &str = match (evt.as_ref(), param.as_ref()) {
            _ => return _type.as_rust_type(),
        };

        return s.into();
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub api_level: i64,
    pub api_compatible: i64,
    pub api_prerelease: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiMetadata {
    pub version: Version,

    pub functions: Vec<Function>,

    pub ui_events: Vec<UiEvent>,
    pub ui_options: Vec<String>,

    pub types: Types,
    pub error_types: ErrorTypes,
}
