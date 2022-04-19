use std::cell::RefCell;

use gtk::glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene};

use crate::colors::Color;

pub struct Cursor {
    pub pos: RefCell<graphene::Rect>,
    pub color: RefCell<Color>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: RefCell::new(graphene::Rect::new(0.0, 0.0, 0.0, 0.0)),
            color: Default::default(),
        }
    }
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
        snapshot.append_color(&self.color.borrow(), &self.pos.borrow())
    }
}
