use gtk::{glib, subclass::prelude::*};

use crate::{boxed::Tabpage, nvim::Neovim};

mod imp;

glib::wrapper! {
    pub struct Tab(ObjectSubclass<imp::Tab>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Tab {
    pub fn new(nvim: &Neovim, label: &str, tabpage: Tabpage) -> Self {
        glib::Object::new(&[("nvim", nvim), ("label", &label), ("tabpage", &tabpage)])
            .expect("Failed to create a Tab")
    }

    fn nvim(&self) -> Neovim {
        self.imp().nvim.borrow().clone()
    }
}
