use std::cell::Cell;

use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::child_iter::IterChildren;

#[derive(Default)]
pub struct MsgWin {
    pub height: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for MsgWin {
    const NAME: &'static str = "MsgWin";
    type Type = super::MsgWin;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for MsgWin {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.set_property("overflow", gtk::Overflow::Hidden);
    }
}

impl WidgetImpl for MsgWin {
    fn measure(
        &self,
        widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        let m = if let Some(child) = widget.first_child() {
            child.measure(orientation, for_size)
        } else {
            self.parent_measure(widget, orientation, for_size)
        };

        match orientation {
            gtk::Orientation::Vertical => {
                let height = self.height.get();
                (height, height, m.2, m.3)
            }
            _ => m,
        }
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        for child in widget.iter_children() {
            if child.should_layout() {
                child.allocate(width, height, -1, None);
            }
        }
    }
}
