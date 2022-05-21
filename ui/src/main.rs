use gtk::gio;
use gtk::pango;
use gtk::prelude::*;
use gtk::Application;

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

    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);

    let app = Application::builder()
        .application_id("com.github.vhakulinen.gnvim")
        .flags(flags)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = AppWindow::new(app);
    window.present();
}
