use std::ffi::{OsStr, OsString};
use std::ops::Deref;

use clap::Parser;
use gtk::glib;

#[derive(Parser, Default, Debug, Clone)]
#[clap(author, version)]
pub struct Arguments {
    /// Neovim binary.
    #[clap(long, name = "BIN", default_value = "nvim")]
    pub nvim: OsString,

    /// Path to the gnvim runtime files.
    #[structopt(
        long = "rtp",
        name = "DIR",
        default_value = "/usr/local/share/gnvim/runtime",
        env = "GNVIM_RUNTIME_PATH"
    )]
    pub rtp: String,

    /// Files to open.
    #[clap(name = "FILES")]
    pub files: Vec<OsString>,

    /// Arguments for neovim.
    #[clap(name = "ARGS", last = true)]
    pub nvim_args: Vec<OsString>,
}

impl Arguments {
    pub fn nvim_cmd_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = vec![
            self.nvim.clone(),
            OsString::from("--embed"),
            OsString::from("--cmd"),
            OsString::from(format!("let &rtp.=',{}'", self.rtp)),
        ];

        args.extend_from_slice(&self.nvim_args);
        args.extend_from_slice(&self.files);

        args
    }
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
