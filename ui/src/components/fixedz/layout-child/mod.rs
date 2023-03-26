use std::cell::Ref;

use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

mod imp;

glib::wrapper! {
    pub struct Child(ObjectSubclass<imp::Child>)
        @extends gtk::LayoutChild, gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Child {
    pub fn new(manager: &super::layout_manager::LayoutManager, for_child: &gtk::Widget) -> Self {
        glib::Object::builder()
            .property("layout-manager", manager)
            .property("child-widget", for_child)
            .build()
    }

    pub fn set_position(&self, x: f32, y: f32) {
        let transform = gsk::Transform::new().translate(&graphene::Point::new(x, y));

        self.set_property("position", transform);
    }

    pub fn set_zindex(&self, z: i64) {
        self.set_property("z-index", z);
    }

    pub fn position(&self) -> Ref<'_, gsk::Transform> {
        self.imp().position.borrow()
    }

    pub fn zindex(&self) -> i64 {
        self.imp().zindex.get()
    }
}
