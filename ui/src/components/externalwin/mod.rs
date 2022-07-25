use crate::nvim::Neovim;
use gtk::{self, glib, subclass::prelude::*};

use super::Grid;

mod imp;

glib::wrapper! {
    pub struct ExternalWindow(ObjectSubclass<imp::ExternalWindow>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ExternalWindow {
    pub fn new(parent: &gtk::Window, grid: &Grid) -> Self {
        glib::Object::new(&[
            ("main-window", &parent),
            ("transient-for", &parent),
            ("grid", &grid),
            ("deletable", &false),
        ])
        .expect("failed to create ExternalWindow")
    }

    fn nvim(&self) -> Neovim {
        self.imp().grid.borrow().nvim()
    }

    fn grid_id(&self) -> i64 {
        self.imp().grid.borrow().id()
    }
}
