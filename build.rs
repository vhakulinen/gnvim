use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gnvim_version.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut cmd = Command::new("git");
    cmd.arg("describe").arg("--always").arg("--tags");

    let version = cmd.output().unwrap();

    if !version.status.success() {
        panic!("Failed to get version from git");
    }

    let mut version_str = String::from_utf8(version.stdout).unwrap();
    version_str.pop();

    f.write_all(
        format!("const VERSION: &str = \"{}\";", version_str)
            .into_bytes()
            .as_slice(),
    )
    .unwrap();
}
