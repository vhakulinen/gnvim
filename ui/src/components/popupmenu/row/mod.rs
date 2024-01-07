use gtk::glib;

mod imp;

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Row {
    fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}
