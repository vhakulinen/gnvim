use gtk::{glib, graphene, prelude::*, subclass::prelude::*};

use crate::{child_iter::IterChildren, components::fixedz};

#[derive(Default)]
pub struct LayoutManager {}

#[glib::object_subclass]
impl ObjectSubclass for LayoutManager {
    const NAME: &'static str = "FixedzLayoutManager";
    type Type = super::LayoutManager;
    type ParentType = gtk::LayoutManager;
}

impl ObjectImpl for LayoutManager {}

impl LayoutManagerImpl for LayoutManager {
    fn layout_child_type() -> Option<glib::Type> {
        Some(fixedz::Child::static_type())
    }

    fn create_layout_child(
        &self,
        layout_manager: &Self::Type,
        _widget: &gtk::Widget,
        for_child: &gtk::Widget,
    ) -> gtk::LayoutChild {
        fixedz::Child::new(layout_manager, for_child).upcast()
    }

    fn measure(
        &self,
        layout_manager: &Self::Type,
        widget: &gtk::Widget,
        orientation: gtk::Orientation,
        _for_size: i32,
    ) -> (i32, i32, i32, i32) {
        let mut self_min = 0f32;
        let mut self_nat = 0f32;

        // NOTE(ville): Originally taken from GtkFixed.

        for child in widget.iter_children() {
            if !child.should_layout() {
                continue;
            }

            let (child_min, child_nat, _, _) = child.measure(orientation, -1);
            let (child_min_opp, child_nat_opp, _, _) = child.measure(
                if matches!(orientation, gtk::Orientation::Vertical) {
                    gtk::Orientation::Vertical
                } else {
                    gtk::Orientation::Horizontal
                },
                -1,
            );

            let (min, nat) = match orientation {
                gtk::Orientation::Horizontal => (
                    graphene::Rect::new(0.0, 0.0, child_min as f32, child_min_opp as f32),
                    graphene::Rect::new(0.0, 0.0, child_nat as f32, child_nat_opp as f32),
                ),
                gtk::Orientation::Vertical => (
                    graphene::Rect::new(0.0, 0.0, child_min_opp as f32, child_min as f32),
                    graphene::Rect::new(0.0, 0.0, child_nat_opp as f32, child_nat as f32),
                ),
                _ => unimplemented!(),
            };

            let layout_child = layout_manager.layout_child(&child);
            let child_pos = layout_child.position();

            let min = child_pos.transform_bounds(&min);
            let nat = child_pos.transform_bounds(&nat);

            match orientation {
                gtk::Orientation::Horizontal => {
                    self_min = self_min.max(min.x() + min.width());
                    self_nat = self_nat.max(nat.x() + nat.width());
                }
                gtk::Orientation::Vertical => {
                    self_min = self_min.max(min.y() + min.height());
                    self_nat = self_nat.max(nat.y() + nat.height());
                }
                _ => unimplemented!(),
            }
        }

        (self_min.ceil() as i32, self_nat.ceil() as i32, -1, -1)
    }

    fn allocate(
        &self,
        layout_manager: &Self::Type,
        widget: &gtk::Widget,
        width: i32,
        height: i32,
        baseline: i32,
    ) {
        self.parent_allocate(layout_manager, widget, width, height, baseline);

        for child in widget.iter_children() {
            if !child.should_layout() {
                continue;
            }

            let (req, _) = child.preferred_size();
            child.allocate(
                req.width(),
                req.height(),
                -1,
                Some(&layout_manager.layout_child(&child).position()),
            );
        }
    }
}
