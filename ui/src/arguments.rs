use std::ffi::OsString;
use std::ops::Deref;

use clap::Parser;
use gtk::glib;

#[derive(Parser, Default, Debug, Clone)]
#[clap(author, version)]
pub struct Arguments {
    /// Neovim binary.
    #[clap(long, name = "BIN", default_value = "nvim")]
    pub nvim: OsString,

    /// Files to open.
    #[clap(name = "FILES")]
    pub files: Vec<OsString>,

    /// Arguments for neovim.
    #[clap(name = "ARGS", last = true)]
    pub nvim_args: Vec<OsString>,
}

#[derive(Default, Clone, glib::Boxed)]
#[boxed_type(name = "Arguments")]
pub struct BoxedArguments(pub Arguments);

impl Deref for BoxedArguments {
    type Target = Arguments;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
