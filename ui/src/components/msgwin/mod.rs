use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp;

glib::wrapper! {
    /// Container for the message window.
    ///
    /// MsgWin provides way to set grid independent height.
    pub struct MsgWin(ObjectSubclass<imp::MsgWin>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl MsgWin {
    pub fn set_height(&self, h: i32) {
        self.imp().height.set(h);
        self.queue_resize();
    }

    pub fn height(&self) -> i32 {
        self.imp().height.get()
    }
}
