use std::cell::{self, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk, prelude::*};

use crate::font::Font;
use crate::math::ease_out_cubic;
use crate::{some_or_return, some_or_return_val, warn, SCALE};

use super::row::Cell;
use super::Row;

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::GridBuffer)]
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
    #[property(set, minimum = 0.0)]
    pub scroll_transition: cell::Cell<f64>,
    /// Y offset for the main buffer.
    pub y_offset: cell::Cell<f32>,
    /// Y offset for the scroll/background buffer.
    pub y_offset_scroll: cell::Cell<f32>,

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

#[derive(Default)]
struct ScrollNode {
    node: Option<gsk::RenderNode>,
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

        let (from, to) = self.scroll_delta_to_range(delta);
        let rows = self.rows.borrow();
        let rows = &rows[from..to];

        let snapshot = gtk::Snapshot::new();
        rows.render_to_snapshot(&snapshot, &self.font.borrow());

        let font = self.font.borrow();

        let start = self.y_offset.get() + font.row_to_y(from as f64) as f32;
        let target = font.row_to_y(from as f64 - delta) as f32;
        let scroll_node = ScrollNode {
            node: snapshot.to_node(),
            offset: 0.0,
            target,
            start,
            reached_view: false,
        };

        let delta_neg = font.row_to_y(-delta) as f32;
        self.scroll_nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|s| s.adjust(delta_neg));

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
                    return Continue(true);
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

                        let mut bounds = some_or_return_val!(
                            &s.node,
                            false,
                            "grid-buffer: scroll node missing node"
                        )
                        .bounds();
                        bounds.offset(0.0, s.start + s.offset);

                        if clip.intersection(&bounds).is_none() {
                            !s.reached_view
                        } else {
                            s.reached_view = true;
                            true
                        }
                    });

                    let y = start_y + ((target_y - start_y) * t);
                    imp.y_offset.set(y);

                    Continue(true)
                } else {
                    imp.y_offset.set(target_y);
                    imp.scroll_nodes.borrow_mut().clear();

                    Continue(false)
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
    fn snapshot(&self, real_snapshot: &gtk::Snapshot) {
        let (_, req) = self.obj().preferred_size();

        if self.dirty.get() {
            if let Some(ref node) = self.backbuffer.borrow().as_ref() {
                real_snapshot.append_node(node);
            }
            return;
        }

        let snapshot = gtk::Snapshot::new();

        snapshot.push_clip(&graphene::Rect::new(
            0.0,
            0.0,
            req.width() as f32,
            req.height() as f32,
        ));

        for node in self.background_nodes.borrow().iter() {
            snapshot.append_node(node);
        }

        // Render the scroll nodes.
        self.scroll_nodes.borrow().iter().for_each(|s| {
            if !s.reached_view {
                return;
            }
            let node = some_or_return!(&s.node, "grid-buffer: scroll node missing node");

            snapshot.save();
            snapshot.translate(&graphene::Point::new(0.0, s.start + s.offset));
            snapshot.append_node(node);
            snapshot.restore();
        });

        snapshot.save();
        snapshot.translate(&graphene::Point::new(0.0, self.y_offset.get()));
        self.rows
            .borrow()
            .render_to_snapshot(&snapshot, &self.font.borrow());
        snapshot.restore();

        snapshot.pop();

        self.backbuffer.replace(snapshot.to_node());

        if let Some(ref node) = self.backbuffer.borrow().as_ref() {
            real_snapshot.append_node(node);
        }
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

trait RenderRows {
    fn render_to_snapshot(&self, snapshot: &gtk::Snapshot, font: &Font);
}

impl<T> RenderRows for T
where
    T: AsRef<[Row]>,
{
    fn render_to_snapshot(&self, snapshot: &gtk::Snapshot, font: &Font) {
        for (i, row) in self.as_ref().iter().enumerate() {
            let y = font.row_to_y(i as f64);

            snapshot.save();
            snapshot.translate(&graphene::Point::new(0.0, y as f32));

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
            snapshot.restore();
        }
    }
}
