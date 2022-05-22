use clap::Parser;
use gtk::gio;
use gtk::pango;
use gtk::prelude::*;
use gtk::Application;

mod arguments;
mod colors;
mod components;
mod font;
mod input;
mod macros;
mod nvim;
mod render;

use components::appwindow::AppWindow;

pub const SCALE: f32 = pango::SCALE as f32;
pub const WINDOW_RESIZE_DEBOUNCE_MS: u64 = 10;

fn main() {
    gio::resources_register_include!("gnvim.gresource").expect("Failed to register resources.");

    let args = arguments::Arguments::parse();
    let args = arguments::BoxedArguments(args);

    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);

    let app = Application::builder()
        .application_id("com.github.vhakulinen.gnvim")
        .flags(flags)
        .build();

    app.connect_activate(move |app| build_ui(app, &args));

    app.run_with_args::<&str>(&[]);
}

fn build_ui(app: &Application, args: &arguments::BoxedArguments) {
    let window = AppWindow::new(app, args);
    window.present();
}
