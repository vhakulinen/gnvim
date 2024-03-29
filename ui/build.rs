use std::{env, fs, path::Path};

fn main() {
    let outdir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let outdir = Path::new(&outdir);

    let appid = env::var("APPID").unwrap_or(String::from("com.github.vhakulinen.gnvim.Devel"));

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "gnvim.gresource",
    );

    let config_dst = Path::new(&outdir).join("config.rs");
    fs::write(
        config_dst,
        format!("const APPID: &'static str = \"{appid}\";"),
    )
    .expect("failed to create config.rs");

    println!("cargo:rerun-if-changed=build.rs")
}
