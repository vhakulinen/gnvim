use gtk::{glib, graphene, prelude::*, subclass::prelude::*};

use crate::child_iter::IterChildren;

#[derive(Default)]
pub struct Fixedz {
    pub layout_manager: super::layout_manager::LayoutManager,
}

#[glib::object_subclass]
impl ObjectSubclass for Fixedz {
    const NAME: &'static str = "Fixedz";
    type Type = super::Fixedz;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for Fixedz {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.set_layout_manager(Some(&self.layout_manager));
    }
}

impl WidgetImpl for Fixedz {
    fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
        let mut children = widget
            .iter_children()
            .map(|c| self.layout_manager.layout_child(&c))
            .collect::<Vec<super::Child>>();

        children.sort_by_key(|c| c.zindex());

        for child in children.iter() {
            widget.snapshot_child(&child.child_widget(), snapshot);
        }
    }

    fn contains(&self, widget: &Self::Type, x: f64, y: f64) -> bool {
        /* NOTE(ville): Implement `contains` manually, because we have
         * holes/gaps within our content.
         */

        widget
            .iter_children()
            .map(|c| self.layout_manager.layout_child(&c))
            .find(|c| {
                let pos = c.position();
                let pos = pos.transform_point(&graphene::Point::zero());
                let (x1, y1) = (pos.x() as f64, pos.y() as f64);
                if x1 < x || y1 < y {
                    return false;
                }

                let (req, _) = c.child_widget().preferred_size();
                let (width, height) = (req.width() as f64, req.height() as f64);
                let x2 = x1 + width;
                let y2 = y1 + height;

                if x < x2 || y < y2 {
                    return false;
                }

                true
            })
            .is_some()
    }
}
