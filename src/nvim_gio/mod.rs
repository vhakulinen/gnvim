use gio;
use gio::prelude::*;
use glib;

use log::error;

use nvim_rs::{create::Spawner, neovim::Neovim, Handler};

use crate::nvim_bridge;

pub mod compat;

pub type GioWriter =
    Compat<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>;
pub type GioNeovim = Neovim<GioWriter>;

use compat::Compat;

pub fn new_child<H>(
    handler: H,
    args: Vec<&std::ffi::OsStr>,
    tx: glib::Sender<nvim_bridge::Message>,
) -> GioNeovim
where
    H: Spawner + Handler<Writer = GioWriter>,
{
    let mut flags = gio::SubprocessFlags::empty();
    flags.insert(gio::SubprocessFlags::STDIN_PIPE);
    flags.insert(gio::SubprocessFlags::STDOUT_PIPE);
    flags.insert(gio::SubprocessFlags::STDERR_PIPE);

    let p = gio::Subprocess::newv(&args, flags).unwrap();

    let input = p
        .get_stdin_pipe()
        .unwrap()
        .dynamic_cast::<gio::PollableOutputStream>()
        .unwrap();
    let write = Compat::new(input.into_async_write().unwrap());

    let output = p
        .get_stdout_pipe()
        .unwrap()
        .dynamic_cast::<gio::PollableInputStream>()
        .unwrap();
    let read = Compat::new(output.into_async_read().unwrap());

    let (neovim, io) = Neovim::<
        Compat<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>,
    >::new(read, write, handler);

    let c = glib::MainContext::default();

    let f = async move {
        let _ = io.await;
        if let Err(err) = tx.send(nvim_bridge::Message::Close) {
            error!("Failed to send close message to the gui: {}", err)
        }
    };

    c.spawn(f);

    neovim
}
