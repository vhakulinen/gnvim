use std::cell::{Cell, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, gsk};

use super::buffer::Buffer;

#[derive(Default)]
pub struct Grid {
    /// The grid id from neovim.
    pub id: Cell<i64>,
    /// The content.
    pub buffer: RefCell<Buffer>,
    /// Background nodes.
    pub background_nodes: RefCell<Vec<gsk::RenderNode>>,
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

impl WidgetImpl for Grid {
    fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk::Snapshot) {
        for node in self.background_nodes.borrow().iter() {
            snapshot.append_node(node);
        }

        for row in self.buffer.borrow().rows.iter() {
            for node in row.bg_nodes.iter() {
                snapshot.append_node(node);
            }

            for node in row.fg_nodes.iter() {
                snapshot.append_node(node);
            }
        }
    }
}
