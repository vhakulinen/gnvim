use std::cell::RefCell;

use gtk::subclass::prelude::*;
use gtk::{glib, gsk};

use crate::font::Font;
use crate::SCALE;

use super::Row;

#[derive(Default)]
pub struct GridBuffer {
    /// Our rows of content.
    pub rows: RefCell<Vec<Row>>,
    /// Background nodes.
    pub background_nodes: RefCell<Vec<gsk::RenderNode>>,

    pub font: RefCell<Font>,
}

#[glib::object_subclass]
impl ObjectSubclass for GridBuffer {
    const NAME: &'static str = "GridBuffer";
    type Type = super::GridBuffer;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for GridBuffer {}

impl WidgetImpl for GridBuffer {
    fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk::Snapshot) {
        for node in self.background_nodes.borrow().iter() {
            snapshot.append_node(node);
        }

        for row in self.rows.borrow().iter() {
            for nodes in row.render_node_iter() {
                if let Some(ref nodes) = *nodes {
                    snapshot.append_node(&nodes.bg);
                }
            }
            for nodes in row.render_node_iter() {
                if let Some(ref nodes) = *nodes {
                    snapshot.append_node(&nodes.fg);
                }
            }
        }
    }

    fn measure(
        &self,
        widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        match orientation {
            gtk::Orientation::Horizontal => {
                let w = if let Some(row) = self.rows.borrow().first() {
                    let len = row.cells.len() as f32;

                    let w = len * (self.font.borrow().char_width() / SCALE);
                    w.ceil() as i32
                } else {
                    (self.font.borrow().char_width() / SCALE).ceil() as i32
                };

                (w, w, -1, -1)
            }
            gtk::Orientation::Vertical => {
                let len = self.rows.borrow().len() as f32;
                let h = len * (self.font.borrow().height() / SCALE);
                let h = h.ceil() as i32;

                return (h, h, -1, -1);
            }
            _ => self.parent_measure(widget, orientation, for_size),
        }
    }
}
