use std::cell::RefCell;
use std::rc::Rc;

use gtk::subclass::prelude::*;
use gtk::{glib, gsk};

use super::Row;

#[derive(Default)]
pub struct GridBuffer {
    /// Our rows of content.
    pub rows: RefCell<Vec<Row>>,
    /// Background nodes.
    pub background_nodes: RefCell<Vec<gsk::RenderNode>>,
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
}