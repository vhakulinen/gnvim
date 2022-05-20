use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp;

glib::wrapper! {
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

    pub fn set_y(&self, y: f32) {
        self.imp().y.set(y);
        self.queue_resize();
    }

    pub fn y(&self) -> f32 {
        self.imp().y.get()
    }
}
