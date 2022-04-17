use gtk::gio;
use gtk::prelude::*;
use gtk::Application;

mod components;
use components::appwindow::AppWindow;

fn main() {
    gio::resources_register_include!("gnvim.gresource").expect("Failed to register resources.");

    let app = Application::builder()
        .application_id("com.github.vhakulinen.gnvim")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = AppWindow::new(app);
    window.present();
}
