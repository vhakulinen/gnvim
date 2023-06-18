use std::cell::RefCell;

use futures::lock::Mutex;
use gio_compat::CompatWrite;
use gtk::{glib, subclass::prelude::*};
use nvim::rpc::caller::Sender;

#[derive(Default)]
pub struct Neovim {
    pub writer: Mutex<Option<CompatWrite>>,
    pub msgid_counter: RefCell<u32>,
    pub callbacks: RefCell<Vec<(u32, Sender)>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Neovim {
    const NAME: &'static str = "Neovim";
    type Type = super::Neovim;
}

impl ObjectImpl for Neovim {}
