use std::cell::{Ref, RefCell, RefMut};
use std::thread;

/// ThreadGuard is a _runtime_ thread guard for its internal data. It panics if
/// data is being accessed from a thread other than the one that ThreadGuard
/// was initialized in.
pub struct ThreadGuard<T> {
    thread_id: thread::ThreadId,
    data: RefCell<T>,
}

unsafe impl<T> Send for ThreadGuard<T> {}
unsafe impl<T> Sync for ThreadGuard<T> {}

impl<T> ThreadGuard<T> {
    pub fn new(data: T) -> Self {
        ThreadGuard {
            thread_id: thread::current().id(),
            data: RefCell::new(data),
        }
    }

    #[allow(unused)]
    pub fn borrow(&self) -> Ref<T> {
        match self.check_thread() {
            Ok(_) => self.data.borrow(),
            Err(()) => {
                panic!(
                    "Data is only accessible on thread {:?} (current is {:?})",
                    self.thread_id,
                    thread::current().id(),
                );
            }
        }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        match self.check_thread() {
            Ok(_) => self.data.borrow_mut(),
            Err(()) => {
                panic!(
                    "Data is only accessible on thread {:?} (current is {:?})",
                    self.thread_id,
                    thread::current().id(),
                );
            }
        }
    }

    fn check_thread(&self) -> Result<(), ()> {
        if self.thread_id == thread::current().id() {
            return Ok(());
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn access_denied_across_thread() {
        let data = 1;
        let guard = ThreadGuard::new(data);

        thread::spawn(move || {
            guard.borrow();
        })
        .join()
        .unwrap();
    }

    #[test]
    fn access_granted_from_correct_thread() {
        let data = 1;
        let guard = ThreadGuard::new(data);

        guard.borrow();
    }

    #[test]
    fn can_mutate() {
        let data = 1;
        let guard = ThreadGuard::new(data);

        {
            let mut data = guard.borrow_mut();
            *data = 4;
        }

        assert_eq!(*guard.borrow(), 4);
    }
}
