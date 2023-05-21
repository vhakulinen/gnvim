use gtk::{glib, graphene, gsk};

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

    pub fn set_position_xy(&self, x: f32, y: f32) {
        let transform = gsk::Transform::new().translate(&graphene::Point::new(x, y));
        self.set_position(transform);
    }
}
