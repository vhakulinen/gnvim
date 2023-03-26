use std::cell::{self, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk, prelude::*};

use crate::font::Font;
use crate::SCALE;

use super::row::Cell;
use super::Row;

#[derive(Default)]
pub struct GridBuffer {
    /// Our rows of content.
    pub rows: RefCell<Vec<Row>>,
    /// Background nodes.
    pub background_nodes: RefCell<Vec<gsk::RenderNode>>,

    /// Node containing the "background" buffer (used for the scroll effect).
    pub scroll_node: RefCell<Option<gsk::RenderNode>>,
    /// Callback id for scroll animation.
    pub scroll_tick: RefCell<Option<gtk::TickCallbackId>>,
    /// Scroll transition time.
    pub scroll_transition: cell::Cell<f64>,
    /// Y offset for the main buffer.
    pub y_offset: cell::Cell<f32>,
    /// Y offset for the scroll/background buffer.
    pub y_offset_scroll: cell::Cell<f32>,

    pub font: RefCell<Font>,
}

#[glib::object_subclass]
impl ObjectSubclass for GridBuffer {
    const NAME: &'static str = "GridBuffer";
    type Type = super::GridBuffer;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for GridBuffer {
    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::builder::<Font>("font")
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecDouble::builder("scroll-transition")
                    .minimum(0.0)
                    .flags(glib::ParamFlags::WRITABLE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "font" => self.font.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be object Font"));

                // Invalidate all the render notes.
                self.rows
                    .borrow_mut()
                    .iter_mut()
                    .for_each(|row| row.cells.iter_mut().for_each(Cell::clear_nodes));
            }
            "scroll-transition" => self
                .scroll_transition
                .set(value.get::<f64>().expect("scroll-transition must be a f64") * 1000.0),
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for GridBuffer {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let (_, req) = self.obj().preferred_size();

        snapshot.push_clip(&graphene::Rect::new(
            0.0,
            0.0,
            req.width() as f32,
            req.height() as f32,
        ));

        if let Some(ref node) = *self.scroll_node.borrow() {
            snapshot.save();
            snapshot.translate(&graphene::Point::new(0.0, self.y_offset_scroll.get()));
            snapshot.append_node(node);
            snapshot.restore();
        }

        snapshot.translate(&graphene::Point::new(0.0, self.y_offset.get()));

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

        snapshot.pop();
    }

    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
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

                (h, h, -1, -1)
            }
            _ => self.parent_measure(orientation, for_size),
        }
    }
}
