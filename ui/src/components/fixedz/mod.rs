use gtk::{glib, gsk, prelude::*, subclass::prelude::*};

mod imp;
#[path = "./layout-child/mod.rs"]
mod layout_child;
#[path = "./layout-manager/mod.rs"]
mod layout_manager;

use layout_child::Child;

glib::wrapper! {
    /// Fixedz is like gtk::Fixed, but with support for z-index.
    pub struct Fixedz(ObjectSubclass<imp::Fixedz>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Fixedz {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn child_position(&self, child: &impl IsA<gtk::Widget>) -> gsk::Transform {
        self.imp()
            .layout_manager
            .layout_child(child)
            .position()
            .clone()
    }

    pub fn set_zindex(&self, widget: &impl IsA<gtk::Widget>, z: i64) {
        self.imp().layout_manager.layout_child(widget).set_zindex(z);
    }

    pub fn move_(&self, widget: &impl IsA<gtk::Widget>, x: f32, y: f32) {
        self.imp()
            .layout_manager
            .layout_child(widget)
            .set_position(x, y);
    }

    pub fn put(&self, widget: &impl IsA<gtk::Widget>, x: f32, y: f32) {
        widget.set_parent(self);

        self.imp()
            .layout_manager
            .layout_child(widget)
            .set_position(x, y);
    }
}

impl Default for Fixedz {
    fn default() -> Self {
        Self::new()
    }
}
