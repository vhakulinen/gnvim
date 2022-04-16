extern crate rmp_serde;
extern crate rmpv;
extern crate serde;

mod types;

use types::{ApiMetadata, AsPascalCase, AsRustType};

use crate::types::UiEvent;

fn usage_exit() -> ! {
    eprintln!("Usage: apigen functions|uievents");
    std::process::exit(1);
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let cmd = args.get(1).to_owned().cloned();

    let mut stdin = std::io::stdin();
    let res: Result<ApiMetadata, rmp_serde::decode::Error> = rmp_serde::from_read(&mut stdin);
    let res = res.unwrap();

    match cmd.as_deref() {
        Some("functions") => {
            let functions: Vec<String> = res
                .functions
                .iter()
                .filter_map(|function| {
                    if function.deprecated_since.is_some() {
                        return None;
                    }

                    Some(format!(
                        include_str!("./function.rs.txt"),
                        name = function.name,
                        method = function.name,
                        args_in = function
                            .parameters
                            .iter()
                            .map(|p| format!("{}: {}", p.rust_name(), p.r#type.as_rust_type()))
                            .collect::<Vec<String>>()
                            .join(", "),
                        args_out = function
                            .parameters
                            .iter()
                            .map(|p| p.rust_name())
                            .collect::<Vec<&str>>()
                            .join(", "),
                        output = function.return_type.as_rust_type(),
                    ))
                })
                .collect();

            println!(
                include_str!("./api.rs.txt"),
                functions = functions.join("\n\n"),
            );
        }
        Some("uievents") => {
            let structs = res
                .ui_events
                .iter()
                .filter_map(|event| {
                    if event.parameters.is_empty() {
                        return None;
                    }

                    Some(format!(
                        r#"
                        #[derive(Debug, serde::Deserialize)]
                        pub struct {name} {{
                            {fields}
                        }}
                    "#,
                        name = event.name.as_pascal_case(),
                        fields = event
                            .parameters
                            .iter()
                            .map(|param| {
                                format!(
                                    "pub {name}: {_type},",
                                    name = param.rust_name(),
                                    _type = UiEvent::parameter_type_for(
                                        &event.name,
                                        &param.name,
                                        &param.r#type
                                    ),
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n"),
                    ))
                })
                .collect::<Vec<String>>()
                .join("\n\n");

            let members = res
                .ui_events
                .iter()
                .map(|event| {
                    let name = event.name.as_pascal_case();

                    if event.parameters.is_empty() {
                        format!("pub {}", name)
                    } else {
                        format!("pub {}(Vec<{}>)", name, name)
                    }
                })
                .collect::<Vec<String>>()
                .join(",\n");

            let decode_matches = res
                .ui_events
                .iter()
                .map(|event| {
                    if event.parameters.is_empty() {
                        format!(
                            r#"
                                (Some("{name}"), None) => UiEvent::{member},
                            "#,
                            name = event.name,
                            member = event.name.as_pascal_case(),
                        )
                    } else {
                        format!(
                            r#"
                                (Some("{name}"), Some(params)) => UiEvent::{member}({{
                                    params.into_iter().map({member}::deserialize)
                                    .collect::<Result<Vec<_>, _>>()
                                    .map_err(serde::de::Error::custom)?
                                }}),
                            "#,
                            name = event.name,
                            member = event.name.as_pascal_case(),
                        )
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");

            println!(
                include_str!("./uievents.rs.txt"),
                structs = structs,
                members = members,
                decode_matches = decode_matches,
            );
        }
        _ => usage_exit(),
    }

    /*
    println!("{:?}", res.ui_options);

    println!("{:?}", res.version);

    println!("{:?}", res.types);
    println!("{:?}", res.error_types);
    */
}
