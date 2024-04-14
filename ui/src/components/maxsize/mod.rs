use gtk::glib;

glib::wrapper! {
    /// MaxSizeLayoutManager allows its child to use its natural size up to
    /// given max size.
    pub struct MaxSizeLayoutManager(ObjectSubclass<imp::MaxSizeLayoutManager>)
        @extends gtk::LayoutManager;
}

mod imp {
    use std::{cell::Cell, num::NonZeroI32};

    use gtk::{glib, prelude::*, subclass::prelude::*};

    use crate::child_iter::IterChildren;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MaxSizeLayoutManager)]
    pub struct MaxSizeLayoutManager {
        #[property(get, set)]
        max_width: Cell<Option<NonZeroI32>>,
        #[property(get, set)]
        max_height: Cell<Option<NonZeroI32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MaxSizeLayoutManager {
        const NAME: &'static str = "MaxSizeLayoutManager";
        type Type = super::MaxSizeLayoutManager;
        type ParentType = gtk::LayoutManager;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MaxSizeLayoutManager {}

    impl LayoutManagerImpl for MaxSizeLayoutManager {
        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            for child in widget.iter_children() {
                if !child.should_layout() {
                    continue;
                }
                child.allocate(width, height, baseline, None);
            }
        }

        fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let mut nat = -1;
            let mut min_baseline = -1;
            let mut nat_baseline = -1;

            for child in widget.iter_children() {
                let (n, mb, nb) = match orientation {
                    gtk::Orientation::Vertical => {
                        let (_, nat, mb, nb) = child.measure(orientation, -1);
                        if let Some(max_height) = self.max_height.get().map(|v| v.get()) {
                            (nat.min(max_height), mb, nb)
                        } else {
                            (nat, mb, nb)
                        }
                    }
                    gtk::Orientation::Horizontal => {
                        let (_, nat, mb, nb) = child.measure(orientation, -1);
                        if let Some(max_width) = self.max_width.get().map(|v| v.get()) {
                            (nat.min(max_width), mb, nb)
                        } else {
                            (nat, mb, nb)
                        }
                    }
                    _ => {
                        let (_, nat, mb, nb) = self.parent_measure(widget, orientation, for_size);
                        (nat, mb, nb)
                    }
                };

                nat = nat.max(n);
                min_baseline = mb.max(mb);
                nat_baseline = nb.max(nb);
            }

            (nat, nat, min_baseline, nat_baseline)
        }
    }
}
