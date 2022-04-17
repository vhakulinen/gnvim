use std::cell::Cell;

use gtk::glib;
use gtk::subclass::prelude::*;

#[derive(Default)]
pub struct Grid {
    /// The grid id.
    id: Cell<i64>,
}

#[glib::object_subclass]
impl ObjectSubclass for Grid {
    const NAME: &'static str = "Grid";
    type Type = super::Grid;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Grid {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecInt64::new(
                "grid-id",
                "Grid ID",
                "Grid ID",
                i64::MIN,
                i64::MAX,
                0,
                // TODO(ville): Maybe we cal use ParamFlags::CONSTRUCT_ONLY here?
                glib::ParamFlags::READWRITE,
            )]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "grid-id" => {
                let id = value.get().expect("property `grid-id` needs to be i64");
                self.id.replace(id);
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for Grid {}
