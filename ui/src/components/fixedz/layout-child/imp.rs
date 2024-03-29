use std::cell::{Cell, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, gsk, prelude::*};

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::Child)]
pub struct Child {
    #[property(get, set)]
    pub position: RefCell<gsk::Transform>,
    #[property(get, set, name = "z-index")]
    pub zindex: Cell<i64>,
}

#[glib::object_subclass]
impl ObjectSubclass for Child {
    const NAME: &'static str = "FixedzChild";
    type Type = super::Child;
    type ParentType = gtk::LayoutChild;
}

#[glib::derived_properties]
impl ObjectImpl for Child {
    fn constructed(&self) {
        self.parent_constructed();

        self.obj().connect_position_notify(|this| {
            gtk::prelude::LayoutChildExt::layout_manager(this).layout_changed();
        });

        self.obj().connect_z_index_notify(|this| {
            gtk::prelude::LayoutChildExt::layout_manager(this).layout_changed();
        });
    }
}

impl WidgetImpl for Child {}

impl LayoutChildImpl for Child {}
