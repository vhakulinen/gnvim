use gtk::{glib, prelude::*, subclass::prelude::*};

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
}
