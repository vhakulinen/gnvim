pub fn dup_stdin() -> Option<i32> {
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
