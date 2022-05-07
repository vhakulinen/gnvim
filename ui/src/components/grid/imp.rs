use std::cell::{Cell, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*};

use crate::components::{Cursor, GridBuffer};

#[derive(Default)]
pub struct DragStartPosition {
    pub pos: RefCell<(usize, usize)>,
    pub offset: RefCell<(f64, f64)>,
}

#[derive(Default)]
pub struct Grid {
    /// The grid id from neovim.
    pub id: Cell<i64>,
    /// Our cursor on the screen.
    pub cursor: Cursor,
    /// The content.
    pub buffer: GridBuffer,

    pub gesture_click: gtk::GestureClick,
    pub gesture_drag: gtk::GestureDrag,
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

        let layout = gtk::BinLayout::new();
        obj.set_layout_manager(Some(&layout));

        self.buffer.set_parent(obj);
        self.cursor.set_parent(obj);

        self.gesture_click.set_button(0);
        self.gesture_drag.set_button(0);

        obj.add_controller(&self.gesture_click);
        obj.add_controller(&self.gesture_drag);
    }

    fn dispose(&self, _obj: &Self::Type) {
        self.buffer.unparent();
        self.cursor.unparent();
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
