mod imp;

use glib::Object;
use gtk::{gio, glib};

use crate::arguments::BoxedArguments;

glib::wrapper! {
    pub struct AppWindow(ObjectSubclass<imp::AppWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AppWindow {
    pub fn new(app: &gtk::Application, args: &BoxedArguments) -> Self {
        Object::builder()
            .property("application", app)
            .property("args", args)
            .build()
    }
}
