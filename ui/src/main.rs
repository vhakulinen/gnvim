use glib::ExitCode;
use gtk::{gio, pango, prelude::*};

mod api;
mod app;
mod boxed;
mod child_iter;
mod colors;
mod components;
mod fd;
mod font;
mod input;
mod macros;
mod math;
mod nvim;
mod render;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

pub const SCALE: f32 = pango::SCALE as f32;
pub const WINDOW_RESIZE_DEBOUNCE_MS: u64 = 10;

fn main() -> ExitCode {
    gio::resources_register_include!("gnvim.gresource").expect("Failed to register resources.");

    let app = app::App::default();
    app.run()
}
