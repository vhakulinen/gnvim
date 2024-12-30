//! This tool is based on what flatpak does when running flatpaks, and how
//! gnome builder also does the same thing (for fonts _and_ a11y bus).
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn font_args() -> Vec<String> {
    let system_fonts_dir = PathBuf::from("/usr/share/fonts");
    let system_local_fonts_dir = PathBuf::from("/usr/local/share/fonts");
    let system_font_cache_dirs = [
        PathBuf::from("/var/cache/fontconfig"),
        PathBuf::from("/usr/lib/fontconfig/cache"),
    ];

    let mut maps: Vec<String> = vec![];

    if let Some(arg) = bind_mount(&system_fonts_dir, "/run/host/fonts") {
        maps.push(arg);
    }
    if let Some(arg) = bind_mount(&system_local_fonts_dir, "/run/host/local-fonts") {
        maps.push(arg);
    }

    for p in system_font_cache_dirs {
        if let Some(arg) = bind_mount(&p, "/run/host/fonts-cache") {
            maps.push(arg);
        }
    }

    if let Some(arg) = bind_mount(&glib::user_data_dir().join("fonts"), "/run/host/user-fonts") {
        maps.push(arg);
    } else if let Some(arg) = bind_mount(&glib::home_dir().join(".fonts"), "/run/host/user-fonts") {
        maps.push(arg);
    }

    if let Some(arg) = bind_mount(
        &glib::user_cache_dir().join("fontconfig"),
        "/run/host/user-fonts-cache",
    ) {
        maps.push(arg);
    }

    maps
}

fn main() {
    let args: Vec<String> = env::args()
        // Skip the bin name.
        .skip(1)
        .collect();

    let mut cmd = Command::new("flatpak");

    cmd.arg("build")
        .arg("--with-appdir")
        .arg("--allow=devel")
        .arg("--die-with-parent")
        .arg("--talk-name=org.a11y.Bus")
        .arg("--talk-name=org.freedesktop.Flatpak")
        .arg("--filesystem=/sys")
        .arg("--device=dri")
        .arg("--share=network")
        .arg("--share=ipc")
        .arg("--socket=fallback-x11")
        .arg("--socket=wayland")
        // Set the PREFIX that we're using in our make file.
        .arg("--env=PREFIX=/app")
        // Set the target dir to be flatpak specific. This avoids conflicts
        // (cache invalidation) between the host and the flatpak container.
        .arg("--env=CARGO_TARGET_DIR=target/flatpak")
        // Set the path required by the rust sdk.
        .arg("--env=PATH=/app/bin:/usr/bin:/usr/lib/sdk/rust-stable/bin:/usr/lib/sdk/llvm16/bin");

    for arg in font_args() {
        cmd.arg(arg);
    }

    cmd.args(args.iter());

    cmd.spawn()
        .expect("failed to spawn")
        .wait()
        .expect("failed to wait");
}

fn bind_mount(path: &Path, container: &str) -> Option<String> {
    path.exists().then(|| {
        let p = path.to_str().unwrap();
        format!("--bind-mount={}={}", container, p)
    })
}
