use quote::quote;

mod types;

use types::ApiMetadata;

fn usage_exit() -> ! {
    eprintln!("Usage: apigen functions|uievents");
    std::process::exit(1);
}

fn functions(res: ApiMetadata) {
    let functions = res
        .functions
        .iter()
        .filter_map(|function| function.to_tokens());

    let out = quote! {
        use crate::rpc::WriteError;
        use crate::{rpc::{Caller, CallResponse}, types::{UiOptions, Window, Tabpage, Buffer, Dictionary, LuaRef, Object}};

        impl<T> Neovim for T where T: Caller {}

        #[async_trait::async_trait(?Send)]
        pub trait Neovim
        where
            Self: Caller,
        {
            #(#functions)*
        }
    };

    println!("{}", out);
}

fn uievents(res: ApiMetadata) {
    let structs = res.ui_events.iter().filter_map(|event| event.to_struct());
    let members = res.ui_events.iter().map(|event| event.to_enum_arm());
    let display_members = res.ui_events.iter().map(|event| event.to_display_arm());
    let decode_matches = res.ui_events.iter().map(|event| event.to_decode_arm());

    let out = quote! {
        use std::fmt::Display;

        use super::manual::*;

        #(#structs)*

        #[derive(Debug)]
        pub enum UiEvent {
            #(#members)*
        }

        impl Display for UiEvent {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_members)*
                }
            }
        }


        macro_rules! seq_to_vec {
            ($seq:expr) => {{
                let mut v = Vec::with_capacity($seq.size_hint().unwrap_or(0));
                while let Some(evt) = $seq.next_element()? {
                    v.push(evt);
                }
                v
            }};
        }

        impl<'de> serde::Deserialize<'de> for UiEvent {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {

                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = UiEvent;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("valid UiEvent")
                    }

                    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                    where
                        V: serde::de::SeqAccess<'de>,
                    {
                        let name = seq
                            .next_element::<String>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                        Ok(match name.as_str() {
                            #(#decode_matches)*
                            v => panic!("failed to decode message {:?}", v),
                        })
                    }
                }

                d.deserialize_seq(Visitor)
            }
        }
    }
    .to_string();

    println!("{}", out);
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let cmd = args.get(1).to_owned().cloned();

    let mut stdin = std::io::stdin();
    let res: Result<ApiMetadata, rmp_serde::decode::Error> = rmp_serde::from_read(&mut stdin);
    let res = res.unwrap();

    match cmd.as_deref() {
        Some("functions") => functions(res),
        Some("uievents") => uievents(res),
        _ => usage_exit(),
    }

    /*
    println!("{:?}", res.ui_options);

    println!("{:?}", res.version);

    println!("{:?}", res.types);
    println!("{:?}", res.error_types);
    */
}
