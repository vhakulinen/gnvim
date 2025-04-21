use std::cell::{self, RefCell};
use std::collections::VecDeque;

use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk, prelude::*};
use nvim::types::uievents::WinViewportMargins;

use crate::font::Font;
use crate::math::ease_out_cubic;
use crate::{some_or_return, SCALE};

use super::row::Cell;
use super::Row;

#[derive(Default, Clone, Copy, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "ViewportMargins")]
pub struct ViewportMargins {
    pub top: i64,
    pub bottom: i64,
    pub left: i64,
    pub right: i64,
}

impl ViewportMargins {
    pub fn viewport_size(&self, font: &Font, size: &Size) -> graphene::Rect {
        let x = font.col_to_x(self.left as f64);
        let y = font.row_to_y(self.top as f64);
        let h = font.row_to_y(size.height as f64) - font.row_to_y(self.bottom as f64);
        let w = font.col_to_x(size.width as f64) - font.col_to_x(self.right as f64);

        graphene::Rect::new(x as f32, y as f32, w as f32, h as f32)
    }
}

impl From<&WinViewportMargins> for ViewportMargins {
    fn from(value: &WinViewportMargins) -> Self {
        Self {
            top: value.top,
            bottom: value.bottom,
            left: value.left,
            right: value.right,
        }
    }
}

#[derive(Default, Clone, Copy, glib::Boxed)]
#[boxed_type(name = "GridSize")]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

pub struct Nodes {
    pub foreground: gsk::RenderNode,
    pub background: gsk::RenderNode,
    pub margins: gsk::RenderNode,

    // margin_* nodes are cached portions of the `margins` node.
    pub margin_top: Option<gsk::RenderNode>,
    pub margin_bottom: Option<gsk::RenderNode>,
    pub margin_sides: Option<gsk::RenderNode>,
}

impl Default for Nodes {
    fn default() -> Self {
        Self {
            foreground: gsk::ContainerNode::new(&[]).upcast(),
            margins: gsk::ContainerNode::new(&[]).upcast(),
            background: gsk::ContainerNode::new(&[]).upcast(),

            margin_top: None,
            margin_bottom: None,
            margin_sides: None,
        }
    }
}

struct ScrollNode {
    node: gsk::RenderNode,
    y_offset: f64,
    end_time: f64,
}

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::GridBuffer)]
pub struct GridBuffer {
    /// Our rows of content.
    pub rows: RefCell<Vec<Row>>,
    pub nodes: RefCell<Nodes>,
    #[property(get, set = Self::set_size)]
    size: RefCell<Size>,

    /// Scroll transition time.
    #[property(set, minimum = 0.0)]
    pub scroll_transition: cell::Cell<f64>,
    /// Y offset for the main buffer.
    #[property(get, set)]
    pub y_offset: cell::Cell<f64>,

    #[property(get, set = Self::set_font)]
    pub font: RefCell<Font>,

    /// The viewport delta value from win_viewport event.
    ///
    /// Setting this property will cause the buffer to do a scroll animation.
    #[property(get, set = Self::set_scroll_delta)]
    scroll_delta: std::cell::Cell<f64>,

    #[property(get, set = Self::set_viewport_margins)]
    pub viewport_margins: RefCell<ViewportMargins>,
    /// If our content is "dirty" (i.e. we're waiting for flush event).
    #[property(get, set)]
    pub dirty: std::cell::Cell<bool>,
    /// Previous render. Drawn when we're "dirty".
    backbuffer: RefCell<Option<gsk::RenderNode>>,

    scroll_nodes: RefCell<VecDeque<ScrollNode>>,
    tick_id: RefCell<Option<gtk::TickCallbackId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GridBuffer {
    const NAME: &'static str = "GridBuffer";
    type Type = super::GridBuffer;
    type ParentType = gtk::Widget;
}

impl GridBuffer {
    fn set_size(&self, size: Size) {
        self.size.replace(size);
        self.invalidate_viewport_margins(None);

        let mut rows = self.rows.borrow_mut();
        rows.resize_with(size.height, Default::default);

        for row in rows.iter_mut() {
            // Invalidate the last cell's nodes so they'll get re-render when
            // truncating the rows.
            row.cells.resize(size.width, Cell::default());

            // Clear the last cell's render nodes. This is needed when we're
            // truncating, which might cause the last render segment to be
            // cut off.
            if let Some(cell) = row.cells.last_mut() {
                // TODO(ville): Should we do this also before the resize?
                cell.clear_nodes();
            }
        }

        let obj = self.obj();
        obj.set_dirty(true);
        obj.queue_resize();
    }

    fn set_font(&self, value: Font) {
        self.font.replace(value);
        self.invalidate_viewport_margins(None);
        self.obj().set_dirty(true);

        // Invalidate all the render nodes.
        self.rows.borrow_mut().iter_mut().for_each(|row| {
            row.clear_render_node();
            row.cells.iter_mut().for_each(Cell::clear_nodes)
        });
    }

    fn invalidate_viewport_margins(&self, og: Option<ViewportMargins>) {
        let mut nodes = self.nodes.borrow_mut();

        match og {
            Some(og) => {
                let value = self.viewport_margins.borrow();
                if og != *value {
                    if value.top != og.top {
                        nodes.margin_top.take();
                    }
                    if value.bottom != og.bottom {
                        nodes.margin_bottom.take();
                    }
                    if value.left != og.left || value.right != og.right {
                        nodes.margin_sides.take();
                    }
                }
            }
            None => {
                nodes.margin_top.take();
                nodes.margin_bottom.take();
                nodes.margin_sides.take();
            }
        }
    }

    fn set_viewport_margins(&self, value: ViewportMargins) {
        let og = self.viewport_margins.replace(value);
        self.invalidate_viewport_margins(Some(og));
        self.obj().set_dirty(true)
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

    fn scroll_node(&self, delta: f64, y: f64, end_time: f64) -> ScrollNode {
        let (from, to) = self.scroll_delta_to_range(delta);
        let rows = self.rows.borrow();
        let node = gsk::ContainerNode::new(
            &rows
                .iter()
                .skip(from)
                .take(to)
                .map(|row| row.cached_render_node().clone())
                .collect::<Vec<gsk::RenderNode>>(),
        )
        .upcast();

        ScrollNode {
            node,
            y_offset: y,
            end_time,
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

        let duration = self.scroll_transition.get();
        let end_time = start_time + duration;

        let delta_rows = self.font.borrow().row_to_y(delta);
        let y = self.y_offset.get() + delta_rows;

        let mut scroll_nodes = self.scroll_nodes.borrow_mut();

        // Remove any scroll nodes that have ended.
        let to_pop = scroll_nodes
            .iter()
            .take_while(|s| s.end_time < start_time)
            .count();
        scroll_nodes.drain(0..to_pop);

        // Adjust the existing scroll nodes.
        scroll_nodes
            .iter_mut()
            .for_each(|s| s.y_offset -= delta_rows);

        // Add the new scroll node.
        scroll_nodes.push_back(self.scroll_node(delta, -delta_rows, end_time));

        let old_id = self
            .tick_id
            .replace(Some(self.obj().add_tick_callback(move |obj, clock| {
                obj.queue_draw();

                let now = clock.frame_time() as f64;
                let time_left = end_time - now;
                if time_left < 0.0 {
                    obj.set_y_offset(0.0);
                    // Clear the scroll nodes.
                    obj.imp().scroll_nodes.borrow_mut().clear();
                    return glib::ControlFlow::Break;
                }

                let d = time_left / duration;
                let e = 1.0 - ease_out_cubic(1.0 - d);
                obj.set_y_offset(y * e);

                glib::ControlFlow::Continue
            })));

        if let Some(id) = old_id {
            id.remove()
        }
    }

    pub fn create_margins_mask(&self) -> gsk::RenderNode {
        let font = self.font.borrow();
        let vp = self.viewport_margins.borrow();
        let size = self.size.borrow();

        gsk::ColorNode::new(&gdk::RGBA::WHITE, &vp.viewport_size(&font, &size)).upcast()
    }
}

#[glib::derived_properties]
impl ObjectImpl for GridBuffer {}

impl WidgetImpl for GridBuffer {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        if self.dirty.get() {
            if let Some(ref node) = self.backbuffer.borrow().as_ref() {
                snapshot.append_node(node);
            }
            return;
        }

        let y_offset = self.y_offset.get();
        let scroll_nodes = self
            .scroll_nodes
            .borrow()
            .iter()
            .map(|s| {
                let node = gsk::TransformNode::new(
                    &s.node,
                    &gsk::Transform::new().translate(&graphene::Point::new(0.0, s.y_offset as f32)),
                );

                return node.upcast();
            })
            .collect::<Vec<gsk::RenderNode>>();

        let scroll = gsk::TransformNode::new(
            &gsk::ContainerNode::new(&scroll_nodes),
            &gsk::Transform::new().translate(&graphene::Point::new(0.0, y_offset as f32)),
        );

        let nodes = self.nodes.borrow();

        let foreground = gsk::TransformNode::new(
            &nodes.foreground,
            &gsk::Transform::new().translate(&graphene::Point::new(0.0, y_offset as f32)),
        );

        let node = gsk::ContainerNode::new(&[
            nodes.background.clone(),
            scroll.upcast(),
            foreground.upcast(),
            nodes.margins.clone(),
        ]);

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
