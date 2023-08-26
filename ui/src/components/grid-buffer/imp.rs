use std::cell::{self, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk, prelude::*};

use crate::font::Font;
use crate::math::ease_out_cubic;
use crate::{some_or_return, warn, SCALE};

use super::row::Cell;
use super::Row;

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::GridBuffer)]
pub struct GridBuffer {
    /// Our rows of content.
    pub rows: RefCell<Vec<Row>>,
    /// Render nodes of our rows from the latest `flush` event.
    pub row_nodes: RefCell<Vec<gsk::RenderNode>>,
    /// Background nodes.
    pub background_nodes: RefCell<Vec<gsk::RenderNode>>,

    /// Node containing the "background" buffer (used for the scroll effect).
    pub scroll_node: RefCell<Option<gsk::RenderNode>>,
    /// Callback id for scroll animation.
    pub scroll_tick: RefCell<Option<gtk::TickCallbackId>>,
    /// Scroll transition time.
    #[property(set, minimum = 0.0)]
    pub scroll_transition: cell::Cell<f64>,
    /// Y offset for the main buffer.
    #[property(get, set)]
    pub y_offset: cell::Cell<f32>,

    #[property(get, set = Self::set_font)]
    pub font: RefCell<Font>,

    /// The viewport delta value from win_viewport event.
    ///
    /// Setting this property will cause the buffer to do a scroll animation.
    #[property(get, set = Self::set_scroll_delta)]
    pub scroll_delta: std::cell::Cell<f64>,
    /// If our content is "dirty" (i.e. we're waiting for flush event).
    #[property(get, set)]
    pub dirty: std::cell::Cell<bool>,
    /// Previous render. Drawn when we're "dirty".
    backbuffer: RefCell<Option<gsk::RenderNode>>,

    scroll_nodes: RefCell<Vec<ScrollNode>>,
}

struct ScrollNode {
    node: gsk::RenderNode,
    offset: f32,
    target: f32,
    start: f32,
    /// If the scroll node has reached theview yet (some nodes might start
    /// their animation off screen).
    reached_view: bool,
}

impl ScrollNode {
    fn adjust(&mut self, adjust: f32) {
        self.start += self.offset;
        self.offset = 0.0;
        self.target += adjust;
    }
}

#[glib::object_subclass]
impl ObjectSubclass for GridBuffer {
    const NAME: &'static str = "GridBuffer";
    type Type = super::GridBuffer;
    type ParentType = gtk::Widget;
}

impl GridBuffer {
    fn set_font(&self, value: Font) {
        self.font.replace(value);

        // Invalidate all the render nodes.
        self.rows
            .borrow_mut()
            .iter_mut()
            .for_each(|row| row.cells.iter_mut().for_each(Cell::clear_nodes));
    }

    fn scroll_delta_to_range(&self, delta: f64) -> (usize, usize) {
        let l = self.rows.borrow().len();
        if delta < 0.0 {
            // Scroll up, shift content down.
            let from = (l as i64 - delta.abs() as i64).max(0) as usize;
            let to = l;
            (from, to)
        } else {
            // Scroll down, shift content up.
            let from = 0;
            let to = (delta.abs() as usize).min(l);
            (from, to)
        }
    }

    fn set_scroll_delta(&self, delta: f64) {
        self.scroll_delta.set(delta);

        if delta == 0.0 {
            // Avoid unnecessary work.
            return;
        }

        let start_time = some_or_return!(
            self.obj().frame_clock(),
            "Failed to get a frame clock for grid buffer animation"
        )
        .frame_time() as f64;
        let end_time = start_time + self.scroll_transition.get();

        let font = self.font.borrow();
        let (from, to) = self.scroll_delta_to_range(delta);
        let rows = self.row_nodes.borrow();

        let node = gsk::ContainerNode::new(rows.get(from..to).unwrap_or(&[])).upcast();

        let start = self.y_offset.get();
        let target = font.row_to_y(-delta) as f32;
        let scroll_node = ScrollNode {
            node,
            offset: 0.0,
            target,
            start,
            reached_view: false,
        };

        self.scroll_nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|s| s.adjust(target));

        self.scroll_nodes.borrow_mut().push(scroll_node);

        let target_y = 0.0;
        let start_y = self.y_offset.get() + font.row_to_y(delta) as f32;

        let old_id = self
            .scroll_tick
            .borrow_mut()
            .replace(self.obj().add_tick_callback(move |this, clock| {
                let now = clock.frame_time() as f64;
                if now < start_time {
                    warn!("Clock going backwards");
                    return glib::ControlFlow::Continue;
                }

                let (_, req) = this.preferred_size();
                let clip = graphene::Rect::new(0.0, 0.0, req.width() as f32, req.height() as f32);

                if !this.dirty() {
                    this.queue_draw();
                }

                let imp = this.imp();
                if now < end_time {
                    let t = ease_out_cubic((now - start_time) / (end_time - start_time)) as f32;

                    // Update scroll nodes, and retain only those that haven't gone
                    // of screen yet.
                    imp.scroll_nodes.borrow_mut().retain_mut(|s| {
                        let y = (s.target - s.start) * t;
                        s.offset = y;

                        let mut bounds = s.node.bounds();
                        bounds.offset(0.0, s.start + s.offset);

                        if clip.intersection(&bounds).is_none() {
                            !s.reached_view
                        } else {
                            s.reached_view = true;
                            true
                        }
                    });

                    let y = start_y + ((target_y - start_y) * t);
                    this.set_y_offset(y);

                    glib::ControlFlow::Continue
                } else {
                    this.set_y_offset(target_y);
                    imp.scroll_nodes.borrow_mut().clear();

                    glib::ControlFlow::Break
                }
            }));

        if let Some(old_id) = old_id {
            old_id.remove();
        }
    }
}

impl ObjectImpl for GridBuffer {
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }

    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        self.derived_property(id, pspec)
    }
}

impl WidgetImpl for GridBuffer {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        if self.dirty.get() {
            if let Some(ref node) = self.backbuffer.borrow().as_ref() {
                snapshot.append_node(node);
            }
            return;
        }

        let background = gsk::ContainerNode::new(&self.background_nodes.borrow());

        let scroll_nodes = self
            .scroll_nodes
            .borrow()
            .iter()
            .filter_map(|s| {
                if !s.reached_view {
                    return None;
                }

                let node = gsk::TransformNode::new(
                    &s.node,
                    &gsk::Transform::new()
                        .translate(&graphene::Point::new(0.0, s.start + s.offset)),
                )
                .upcast();

                Some(node)
            })
            .collect::<Vec<gsk::RenderNode>>();

        let scroll = gsk::ContainerNode::new(&scroll_nodes);

        let foreground = gsk::TransformNode::new(
            &gsk::ContainerNode::new(&self.row_nodes.borrow()),
            &gsk::Transform::new().translate(&graphene::Point::new(0.0, self.y_offset.get())),
        );

        let node =
            gsk::ContainerNode::new(&[background.upcast(), scroll.upcast(), foreground.upcast()]);

        snapshot.append_node(&node);
        self.backbuffer.replace(Some(node.upcast()));
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
