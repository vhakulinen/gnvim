use std::cell::RefCell;

use futures::lock::Mutex;
use gtk::{gio, glib, subclass::prelude::*};
use nvim::rpc::caller::Sender;

#[derive(Default)]
pub struct Neovim {
    pub writer: Mutex<Option<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>>,
    pub msgid_counter: RefCell<u32>,
    pub callbacks: RefCell<Vec<(u32, Sender)>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Neovim {
    const NAME: &'static str = "Neovim";
    type Type = super::Neovim;
}

impl ObjectImpl for Neovim {}
