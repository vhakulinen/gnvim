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

        impl<'de> serde::Deserialize<'de> for UiEvent {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let redraw = rmpv::Value::deserialize(d)?;

                let name = redraw[0].as_str();
                // TODO(ville): Would be nice if this was possible to do with the derilization it self...
                let params = redraw.as_array().and_then(|v| {
                    if v[1].as_array().map(|v| v.is_empty()) == Some(true) {
                        None
                    } else {
                        Some(v[1..].to_vec())
                    }
                });

                // TODO(ville): Error handling.
                Ok(match (name, params) {
                    #(#decode_matches)*
                    v => panic!("failed to decode message {:?}", v),
                })
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
