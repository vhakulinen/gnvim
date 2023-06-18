use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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

#[derive(Debug, serde::Deserialize)]
pub struct Parameter {
    pub r#type: String,
    pub name: String,
}

impl Parameter {
    pub fn rust_name(&self) -> syn::Ident {
        syn::parse_str(match self.name.as_str() {
            "fn" => "_fn",
            "type" => "_type",
            s => s,
        })
        .expect("failed to parse name")
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
    pub fn param_type_for(&self, param: &Parameter) -> TokenStream {
        match (self.name.as_ref(), param.name.as_ref()) {
            ("nvim_ui_attach", "options") => quote! { UiOptions },
            _ => self.param_type(&param.r#type),
        }
    }

    fn param_type(&self, ty: &str) -> TokenStream {
        match ty {
            "Boolean" => quote! { bool },
            "Integer" => quote! { i64 },
            "Float" => quote! { f64 },
            "String" => quote! { &str },
            "void" => quote! { () },
            "Window" => quote! { &Window },
            "Tabpage" => quote! { &Tabpage },
            "Buffer" => quote! { &Buffer },
            "ArrayOf(Integer, 2)" => quote! { (i64, i64) },
            "ArrayOf(String)" => quote! { Vec<String> },
            "ArrayOf(Integer)" => quote! { &[i64] },
            "ArrayOf(Buffer)" => quote! { Vec<Buffer> },
            "ArrayOf(Dictionary)" => quote! { Vec<Dictionary> },
            "ArrayOf(Tabpage)" => quote! { Vec<Tabpage> },
            "ArrayOf(Window)" => quote! { Vec<Window> },
            "Array" => quote! { Vec<rmpv::Value> },
            "Dictionary" => quote! { &Dictionary },
            "Object" => quote! { &Object },
            "LuaRef" => quote! { &LuaRef },
            s => unimplemented!("function param type '{}'", s),
        }
    }

    fn output_type_for(&self, ty: &str) -> TokenStream {
        match ty {
            "Boolean" => quote! { bool },
            "Integer" => quote! { i64 },
            "Float" => quote! { f64 },
            "String" => quote! { String },
            "void" => quote! { () },
            "Window" => quote! { Window },
            "Tabpage" => quote! { Tabpage },
            "Buffer" => quote! { Buffer },
            "ArrayOf(Integer, 2)" => quote! { (i64, i64) },
            "ArrayOf(String)" => quote! { Vec<String> },
            "ArrayOf(Integer)" => quote! { Vec<i64> },
            "ArrayOf(Buffer)" => quote! { Vec<Buffer> },
            "ArrayOf(Dictionary)" => quote! { Vec<Dictionary> },
            "ArrayOf(Tabpage)" => quote! { Vec<Tabpage> },
            "ArrayOf(Window)" => quote! { Vec<Window> },
            "Array" => quote! { Vec<rmpv::Value> },
            "Dictionary" => quote! { Dictionary },
            "Object" => quote! { Object },
            "LuaRef" => quote! { LuaRef },
            s => unimplemented!("function output type '{}'", s),
        }
    }

    fn args_in(&self) -> Vec<TokenStream> {
        self.parameters
            .iter()
            .map(|p| {
                let name = p.rust_name();
                let ty = self.param_type_for(p);
                quote! {
                    #name: #ty
                }
            })
            .collect()
    }

    fn args_out(&self) -> Vec<syn::Ident> {
        self.parameters.iter().map(|p| p.rust_name()).collect()
    }

    pub fn to_tokens(&self) -> Option<TokenStream> {
        if self.deprecated_since.is_some() {
            return None;
        }

        let fname: syn::Ident = syn::parse_str(&self.name).expect("failed to parse name");
        let method = &self.name;
        let args_in = self.args_in();
        let args_out = self.args_out();
        let output = self.output_type_for(&self.return_type);

        Some(quote! {
            async fn #fname(self, #(#args_in),*) -> Result<CallResponse<#output>, WriteError> {
                self.call(#method, (#(#args_out,)*)).await
            }
        })
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
    pub fn to_decode_arm(&self) -> TokenStream {
        let member: syn::Ident =
            syn::parse_str(&self.name.as_pascal_case()).expect("failed to parse name");
        let name = &self.name;

        if self.parameters.is_empty() {
            quote! {
                (Some(#name), None) => UiEvent::#member,
            }
        } else {
            quote! (
                (Some(#name), Some(params)) => UiEvent::#member({
                    params.into_iter().map(#member::deserialize)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(serde::de::Error::custom)?
                }),
            )
        }
    }

    pub fn to_display_arm(&self) -> TokenStream {
        let name: syn::Ident =
            syn::parse_str(&self.name.as_pascal_case()).expect("failed to parse name");
        let event = &self.name;

        if self.parameters.is_empty() {
            quote! {
                Self::#name => write!(f, #event),
            }
        } else {
            quote! {
                Self::#name(_) => write!(f, #event),
            }
        }
    }

    pub fn to_enum_arm(&self) -> TokenStream {
        let name: syn::Ident =
            syn::parse_str(&self.name.as_pascal_case()).expect("failed to parse name");

        if self.parameters.is_empty() {
            quote! {
                pub #name,
            }
        } else {
            quote! {
                pub #name(Vec<#name>),
            }
        }
    }

    pub fn to_struct(&self) -> Option<TokenStream> {
        if self.parameters.is_empty() || self.has_manual_type() {
            return None;
        }

        let name: syn::Ident =
            syn::parse_str(&self.name.as_pascal_case()).expect("failed to parse name");

        let fields = self.parameters.iter().map(|param| {
            let name = format_ident!("{}", &param.name);
            let ty = self.field_type_for(&param.name, &param.r#type);

            quote! {
                pub #name: #ty,
            }
        });

        Some(quote! {
            #[derive(Debug, serde::Deserialize)]
            pub struct #name {
                #(#fields)*
            }
        })
    }

    fn field_type_for(&self, param: &str, ty: &str) -> TokenStream {
        match (self.name.as_ref(), param) {
            ("grid_line", "data") => quote! { Vec<GridLineData> },
            ("hl_attr_define", "rgb_attrs") => quote! { HlAttr },
            ("hl_attr_define", "cterm_attrs") => quote! { HlAttr },
            ("mode_info_set", "cursor_styles") => quote! { Vec<ModeInfo> },
            ("popupmenu_show", "items") => quote! { Vec<PopupmenuItem> },
            ("tabline_update", "tabs") => quote! { Vec<TablineTab> },
            ("tabline_update", "buffers") => quote! { Vec<TablineBuffer> },
            ("cmdline_show", "content") => quote! { Vec<CmdlineContent> },
            ("cmdline_block_show", "lines") => quote! { Vec<Vec<CmdlineContent>> },
            ("cmdline_block_append", "lines") => quote! { Vec<CmdlineContent> },
            _ => self.field_type(ty),
        }
    }

    /// Get the rust struct field type for this ui event.
    fn field_type(&self, ty: &str) -> TokenStream {
        match ty {
            "Boolean" => quote! { bool },
            "Integer" => quote! { i64 },
            "Float" => quote! { f64 },
            "String" => quote! { String },
            "void" => quote! { () },
            "Window" => quote! { Window },
            "Tabpage" => quote! { Tabpage },
            "Buffer" => quote! { Buffer },
            "ArrayOf(Integer, 2)" => quote! { (i64, i64) },
            "ArrayOf(String)" => quote! { Vec<String> },
            "ArrayOf(Integer)" => quote! { Vec<i64> },
            "ArrayOf(Buffer)" => quote! { Vec<Buffer> },
            "ArrayOf(Dictionary)" => quote! { Vec<Dictionary> },
            "ArrayOf(Tabpage)" => quote! { Vec<Tabpage> },
            "ArrayOf(Window)" => quote! { Vec<Window> },
            "Array" => quote! { Vec<rmpv::Value> },
            "Dictionary" => quote! { Dictionary },
            "Object" => quote! { Object },
            "LuaRef" => quote! { LuaRef },
            s => unimplemented!("uievent field type '{}'", s),
        }
    }

    pub fn has_manual_type(&self) -> bool {
        match self.name.as_ref() {
            "option_set" => true,
            _ => false,
        }
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
