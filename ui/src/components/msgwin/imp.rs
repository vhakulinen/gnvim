use std::cell::Cell;

use gtk::{glib, prelude::*, subclass::prelude::*};

#[derive(Default)]
pub struct MsgWin {
    pub height: Cell<i32>,
    pub y: Cell<f32>,
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

        let mut child: Option<gtk::Widget> = widget.first_child();
        while let Some(sib) = child {
            if sib.should_layout() {
                sib.allocate(width, height, -1, None);
            }

            child = sib.next_sibling();
        }
    }
}
