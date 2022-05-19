use futures::lock::Mutex;
use gio_compat::CompatWrite;
use gtk::{glib, subclass::prelude::*};
use nvim;

#[derive(Default)]
pub struct Neovim {
    pub nvim: Mutex<Option<nvim::Client<CompatWrite>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Neovim {
    const NAME: &'static str = "Neovim";
    type Type = super::Neovim;
}

impl ObjectImpl for Neovim {}
