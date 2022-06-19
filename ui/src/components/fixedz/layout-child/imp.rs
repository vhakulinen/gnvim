use std::cell::{Cell, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, gsk, prelude::*};

#[derive(Default)]
pub struct Child {
    pub position: RefCell<gsk::Transform>,
    pub zindex: Cell<i64>,
}

#[glib::object_subclass]
impl ObjectSubclass for Child {
    const NAME: &'static str = "FixedzChild";
    type Type = super::Child;
    type ParentType = gtk::LayoutChild;
}

impl ObjectImpl for Child {
    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoxed::new(
                    "position",
                    "position",
                    "Position",
                    gsk::Transform::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecInt64::new(
                    "z-index",
                    "z-index",
                    "Z-index",
                    i64::MIN,
                    i64::MAX,
                    0,
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "position" => self.position.borrow().to_value(),
            "z-index" => self.zindex.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "position" => {
                self.position
                    .replace(value.get().expect("position must be object gsk::Transform"));
            }
            "z-index" => {
                self.zindex
                    .replace(value.get().expect("font value must be i64"));
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for Child {}

impl LayoutChildImpl for Child {}