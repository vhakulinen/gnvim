use std::cell::RefCell;

use gtk::subclass::prelude::*;
use gtk::{glib, gsk};

#[derive(Default)]
pub struct Cursor {
    pub pos: RefCell<(i64, i64)>,
    pub text: RefCell<String>,
    pub double_width: RefCell<bool>,

    pub node: RefCell<Option<gsk::RenderNode>>,

    pub width_percentage: RefCell<f32>,
    pub attr_id: RefCell<i64>,
}

#[glib::object_subclass]
impl ObjectSubclass for Cursor {
    const NAME: &'static str = "Cursor";
    type Type = super::Cursor;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Cursor {}

impl WidgetImpl for Cursor {
    fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk::Snapshot) {
        if let Some(ref node) = *self.node.borrow() {
            snapshot.append_node(node);
        }
    }
}
