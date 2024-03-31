use std::ffi::OsStr;

use futures::channel::oneshot;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use nvim::{
    async_trait,
    rpc::{caller::Response, Caller, HandleError, RpcWriter, WriteError},
    serde,
};

mod imp;

glib::wrapper! {
    /// Wraps the nvim rpc client into a gobject.
    pub struct Neovim(ObjectSubclass<imp::Neovim>);
}

impl Neovim {
    fn new() -> Self {
        glib::Object::new()
    }

    /// Open the neovim subprocess.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments (including the nvim command) for the subprocess.
    /// * `inherit_fds` - If the fds should be shared with the subprocess. Required
    /// for the stdin_fd uiattach option.
    pub fn open(
        &self,
        args: &[&OsStr],
        inherit_fds: bool,
    ) -> gio::InputStreamAsyncRead<gio::PollableInputStream> {
        let mut flags = gio::SubprocessFlags::empty();
        flags.insert(gio::SubprocessFlags::STDIN_PIPE);
        flags.insert(gio::SubprocessFlags::STDOUT_PIPE);

        if inherit_fds {
            flags.insert(gio::SubprocessFlags::INHERIT_FDS);
        }

        let p = gio::Subprocess::newv(args, flags).expect("failed to open nvim subprocess");

        let writer = p
            .stdin_pipe()
            .expect("get stdin pipe")
            .dynamic_cast::<gio::PollableOutputStream>()
            .expect("cast to PollableOutputStream")
            .into_async_write()
            .expect("convert to async write");

        let reader = p
            .stdout_pipe()
            .expect("get stdout pipe")
            .dynamic_cast::<gio::PollableInputStream>()
            .expect("cast to PollableInputStream")
            .into_async_read()
            .expect("covert to async read");

        self.imp()
            .writer
            .try_lock()
            .expect("set rpc writer")
            .replace(writer);

        reader
    }

    pub fn handle_response(&self, response: Response) -> Result<(), HandleError> {
        let mut callbacks = self.imp().callbacks.borrow_mut();
        let caller = callbacks
            .iter()
            .position(|(msgid, _)| *msgid == response.msgid)
            .map(|index| callbacks.swap_remove(index));

        match caller {
            Some((_, recv)) => recv.send(response).map_err(HandleError::CallerDropped),
            None => Err(HandleError::CallerMissing(response)),
        }
    }

    pub async fn write_rpc_response<R: serde::Serialize, E: serde::Serialize>(
        self,
        msgid: u32,
        error: Option<&E>,
        result: Option<&R>,
    ) -> Result<(), WriteError> {
        self.imp()
            .writer
            .lock()
            .await
            .as_mut()
            .expect("nvim writer not set")
            .write_rpc_response(msgid, error, result)
            .await
    }

    pub async fn write_empty_rpc_response(self, msgid: u32) -> Result<(), WriteError> {
        self.write_rpc_response(msgid, None::<&()>, None::<&()>)
            .await
    }
}

#[async_trait::async_trait(?Send)]
impl Caller for &Neovim {
    async fn write<S: AsRef<str>, V: serde::Serialize>(
        self,
        msgid: u32,
        method: S,
        args: V,
    ) -> Result<(), WriteError> {
        self.imp()
            .writer
            .lock()
            .await
            .as_mut()
            .expect("nvim writer not set")
            .write_rpc_request(msgid, method.as_ref(), &args)
            .await
    }

    fn next_msgid(&mut self) -> u32 {
        let imp = self.imp();
        let mut msgid_counter = imp.msgid_counter.borrow_mut();
        let msgid = *msgid_counter;
        *msgid_counter += 1;

        msgid
    }

    fn store_handler(&mut self, msgid: u32, sender: oneshot::Sender<Response>) {
        self.imp().callbacks.borrow_mut().push((msgid, sender));
    }
}

impl Default for Neovim {
    fn default() -> Self {
        Self::new()
    }
}
