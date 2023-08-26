use std::{ffi::OsString, io::IsTerminal};

use gtk::glib;

#[derive(clap::Parser, Default, Debug, Clone, glib::Boxed)]
#[clap(author, version)]
#[boxed_type(name = "Arguments")]
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

    #[clap(skip)]
    pub stdin_fd: Option<i32>,
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

    /// Wrapper around `clap::Praser::parse`. Handle's `Self::stdin_fd`.
    pub fn parse() -> Self {
        let mut args: Self = clap::Parser::parse();

        if !std::io::stdin().is_terminal() {
            args.stdin_fd = dup_stdin();
        }

        args
    }
}

fn dup_stdin() -> Option<i32> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::prelude::AsRawFd;

            unsafe {
                // Duplicate the stdin fd.
                let fd_dup = libc::dup(std::io::stdin().as_raw_fd());

                let fdflags = libc::fcntl(fd_dup, libc::F_GETFD);
                if fdflags < 0 {
                    println!("ERR: couldn't get fdglags");
                    return None;
                }

                // Remove FD_CLOEXEC.
                if fdflags & libc::FD_CLOEXEC == 1
                    && libc::fcntl(fd_dup, libc::F_SETFD, fdflags & !libc::FD_CLOEXEC) < 0
                    {
                        println!("ERR: couldn't set fdglags");
                        return None;
                    }

                Some(fd_dup)
            }
        } else {
            println!("ERR: stdin pipe not supported on this platform");
            return None;
        }
    }
}
