mod imp;

use glib::Object;
use gtk::{gio, glib, subclass::prelude::*};

glib::wrapper! {
    pub struct Grid(ObjectSubclass<imp::Grid>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Grid {
    fn new(id: i64) -> Self {
        Object::new(&[("grid-id", &id)]).expect("Failed to create Grid")
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(0)
    }
}
