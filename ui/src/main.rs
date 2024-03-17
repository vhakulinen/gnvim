use std::io::IsTerminal;

use glib::ExitCode;
use gtk::{gio, pango, prelude::*};

mod api;
mod app;
mod boxed;
mod child_iter;
mod colors;
mod components;
mod font;
mod input;
mod macros;
mod math;
mod nvim;
mod render;

pub const SCALE: f32 = pango::SCALE as f32;
pub const WINDOW_RESIZE_DEBOUNCE_MS: u64 = 10;

fn main() -> ExitCode {
    gio::resources_register_include!("gnvim.gresource").expect("Failed to register resources.");

    // Duplicate the stdin fd if the user is trying to pipe content to us.
    // See `:h ui-startup-stdin`.
    let stdin_fd = (!std::io::stdin().is_terminal())
        .then(|| dup_stdin())
        .flatten();

    let app = app::App::new(stdin_fd);
    app.run()
}

fn dup_stdin() -> Option<i32> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::prelude::AsRawFd;

            unsafe {
                // Duplicate the stdin fd.
                let fd_dup = libc::dup(std::io::stdin().as_raw_fd());
                if fd_dup < 0 {
                    println!("ERR: couldn't duplicate stdin");
                    return None;
                }

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
