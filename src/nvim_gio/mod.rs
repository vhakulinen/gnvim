use gtk::prelude::*;
use gtk::{gio, glib};

use log::error;

use nvim_rs::{create::Spawner, neovim::Neovim, Handler};

use crate::nvim_bridge;

pub mod compat;

pub type GioWriter =
    Compat<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>;
pub type GioNeovim = Neovim<GioWriter>;

#[derive(Debug)]
pub enum Error {
    Pipe,
    ToPollaple,
    ToAsync,
    Glib(glib::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Pipe => write!(fmt, "Failed to open pipe to subprocess"),
            Error::ToPollaple => {
                write!(fmt, "Failed to turn pipe into pollable stream")
            }
            Error::ToAsync => {
                write!(fmt, "Failed to turn pollable stream into async")
            }
            Error::Glib(e) => {
                write!(fmt, "Failed to open nvim subprocess: {}", e)
            }
        }
    }
}

impl From<glib::Error> for Error {
    fn from(arg: glib::Error) -> Self {
        Error::Glib(arg)
    }
}

use compat::Compat;

pub fn new_child<H>(
    handler: H,
    args: Vec<&std::ffi::OsStr>,
    tx: glib::Sender<nvim_bridge::Message>,
) -> Result<GioNeovim, Error>
where
    H: Spawner + Handler<Writer = GioWriter>,
{
    let mut flags = gio::SubprocessFlags::empty();
    flags.insert(gio::SubprocessFlags::STDIN_PIPE);
    flags.insert(gio::SubprocessFlags::STDOUT_PIPE);
    flags.insert(gio::SubprocessFlags::STDERR_PIPE);

    let p = gio::Subprocess::newv(&args, flags).map_err(Error::from)?;

    let input = p
        .stdin_pipe()
        .ok_or(Error::Pipe)?
        .dynamic_cast::<gio::PollableOutputStream>()
        .map_err(|_| Error::ToPollaple)?;
    let write =
        Compat::new(input.into_async_write().map_err(|_| Error::ToAsync)?);

    let output = p
        .stdout_pipe()
        .ok_or(Error::Pipe)?
        .dynamic_cast::<gio::PollableInputStream>()
        .map_err(|_| Error::ToPollaple)?;
    let read =
        Compat::new(output.into_async_read().map_err(|_| Error::ToAsync)?);

    let (neovim, io) = Neovim::<
        Compat<gio::OutputStreamAsyncWrite<gio::PollableOutputStream>>,
    >::new(read, write, handler);

    let c = glib::MainContext::default();

    c.spawn(async move {
        let _ = io.await;
        if let Err(err) = tx.send(nvim_bridge::Message::Close) {
            error!("Failed to send close message to the gui: {}", err)
        }
    });

    Ok(neovim)
}
