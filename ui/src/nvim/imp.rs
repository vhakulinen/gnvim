use std::cell::RefCell;

use futures::lock::Mutex;
use gtk::{
    gio::{self},
    glib,
    subclass::prelude::*,
};
use nvim::rpc::caller::Sender;

pub enum Connection {
    Subprocess(gio::Subprocess),
    Socket(gio::SocketConnection),
}

#[derive(Default)]
pub struct Neovim {
    pub writer: Mutex<Option<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>>,
    pub msgid_counter: RefCell<u32>,
    pub callbacks: RefCell<Vec<(u32, Sender)>>,
    /// Store reference to the actual "connection" to nvim (i.e. subprocess or socket).
    ///
    /// We need this because dropping the `gio::SocketConnection` will close the actual
    /// connection, and keeping reference to the I/O channels wont be enough to avoid this.
    pub connection: RefCell<Option<Connection>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Neovim {
    const NAME: &'static str = "Neovim";
    type Type = super::Neovim;
}

impl ObjectImpl for Neovim {}
