mod imp;

use gtk::{gio, glib, prelude::*};
use nvim::NeovimApi;

use crate::{debug, spawn_local};

glib::wrapper! {
    pub struct AppWindow(ObjectSubclass<imp::AppWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AppWindow {
    pub fn open_files(&self, files: &[gtk::gio::File]) {
        let nvim = self.nvim();
        for file in files.iter() {
            debug!("opening {}", file.uri());
            spawn_local!(glib::clone!(@weak nvim, @strong file => async move {
                nvim
                    .nvim_command(&format!("e {}", file.uri()))
                    .await
                    .expect("nvim_command failed");
            }));
        }
    }
}
