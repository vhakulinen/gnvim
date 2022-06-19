use std::ffi::{OsStr, OsString};

use futures::lock::{MappedMutexGuard, MutexGuard};
use gio_compat::{CompatRead, CompatWrite};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

mod imp;

glib::wrapper! {
    /// Wraps the nvim rpc client into a gobject.
    pub struct Neovim(ObjectSubclass<imp::Neovim>);
}

pub type MutexGuardedNeovim<'a> =
    MappedMutexGuard<'a, Option<nvim::Client<CompatWrite>>, nvim::Client<CompatWrite>>;

impl Neovim {
    fn new() -> Self {
        glib::Object::new(&[]).expect("failed to create Neovim")
    }

    /// Locks the internal client for the caller. The lock should not be held
    /// for too long to allow other parts of the application to access the
    /// client too.
    pub async fn client(&self) -> MutexGuardedNeovim<'_> {
        MutexGuard::map(self.imp().nvim.lock().await, |opt| opt.as_mut().unwrap())
    }

    pub fn open(&self, nvim_bin: &OsStr, files: &[OsString], args: &[OsString]) -> CompatRead {
        let mut flags = gio::SubprocessFlags::empty();
        flags.insert(gio::SubprocessFlags::STDIN_PIPE);
        flags.insert(gio::SubprocessFlags::STDOUT_PIPE);

        let default_args = vec![nvim_bin, OsStr::new("--embed")];
        let cmd_args: Vec<&OsStr> = default_args
            .into_iter()
            .chain(args.iter().map(|a| a.as_ref()))
            .chain(files.iter().map(|a| a.as_ref()))
            .collect();

        let p = gio::Subprocess::newv(&cmd_args, flags).expect("failed to open nvim subprocess");

        let writer: CompatWrite = p
            .stdin_pipe()
            .expect("get stdin pipe")
            .dynamic_cast::<gio::PollableOutputStream>()
            .expect("cast to PollableOutputStream")
            .into_async_write()
            .expect("convert to async write")
            .into();

        let reader: CompatRead = p
            .stdout_pipe()
            .expect("get stdout pipe")
            .dynamic_cast::<gio::PollableInputStream>()
            .expect("cast to PollableInputStream")
            .into_async_read()
            .expect("covert to async read")
            .into();

        let imp = self.imp();
        imp.nvim
            .try_lock()
            .expect("nvim already set")
            .replace(nvim::Client::new(writer));

        reader
    }
}

impl Default for Neovim {
    fn default() -> Self {
        Self::new()
    }
}