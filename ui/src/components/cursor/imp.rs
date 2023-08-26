use std::cell::{Cell, RefCell};

use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk, prelude::*};

use crate::font::Font;
use crate::SCALE;

use super::blink::Blink;

#[derive(Default)]
pub struct Position {
    /// Actual position on the grid.
    pub grid: (i64, i64),
    /// Current position on the screen.
    pub pos: (f64, f64),
    /// Transition time in milliseconds.
    pub transition: f64,
    /// Is the positon already set once.
    ///
    /// If not, we must skip the transition animation to avoid jumpy cursor
    /// (for example, when opening splits).
    pub is_set: bool,
    /// Additional offset to apply to the y coordinate.
    pub y_offset: f64,
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::Cursor)]
pub struct Cursor {
    #[property(set, name ="position-transition", member = transition, type = f64)]
    #[property(set, name ="y-offset", member = y_offset, type = f64)]
    pub pos: RefCell<Position>,

    pub text: RefCell<String>,
    pub double_width: RefCell<bool>,

    pub node: RefCell<Option<gsk::RenderNode>>,

    #[property(set, default = 1.0)]
    pub width_percentage: RefCell<f32>,
    #[property(set)]
    pub attr_id: RefCell<i64>,

    #[property(get, set, default = false)]
    pub active: Cell<bool>,
    #[property(get, set, default = false)]
    pub busy: Cell<bool>,

    #[property(get, set)]
    pub font: RefCell<Font>,
    #[property(set = Self::set_blink)]
    pub blink: RefCell<Option<Blink>>,
    /// Callback id to our function that makes the cursor blink.
    pub blink_tick: RefCell<Option<gtk::TickCallbackId>>,
    pub pos_tick: RefCell<Option<gtk::TickCallbackId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Cursor {
    const NAME: &'static str = "Cursor";
    type Type = super::Cursor;
    type ParentType = gtk::Widget;
}

impl Cursor {
    /// Set the cursor's blink. Removes earlier tick callback, and adds the
    /// new one if needed.
    fn set_blink(&self, blink: Option<Blink>) {
        if let Some(id) = self.blink_tick.borrow_mut().take() {
            id.remove();
        }

        self.blink.replace(blink);
        if self.blink.borrow().is_none() {
            return;
        }

        let new_id = self.obj().add_tick_callback(|this, clock| {
            let imp = this.imp();
            if imp.active.get() || !imp.busy.get() {
                if let Some(ref mut blink) = imp.blink.borrow_mut().as_mut() {
                    blink.tick(clock.frame_time() as f64);
                }
            }

            this.queue_draw();
            glib::ControlFlow::Continue
        });

        self.blink_tick.replace(Some(new_id));
    }
}

impl ObjectImpl for Cursor {
    fn constructed(&self) {
        self.parent_constructed();

        // TODO(ville): Use custom setter instead of signals.

        self.obj().connect_font_notify(|this| this.queue_draw());
        self.obj().connect_active_notify(|this| this.queue_draw());
        self.obj().connect_busy_notify(|this| this.queue_draw());

        self.obj().connect_blink_notify(|this| {
            // Clear the cached render node, since blink directly contributes to it.
            this.imp().node.replace(None);
        });
    }

    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        self.derived_property(id, pspec)
    }

    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }
}

impl WidgetImpl for Cursor {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        if self.busy.get() || !self.active.get() {
            return;
        }

        if let Some(ref node) = *self.node.borrow() {
            let pos = self.pos.borrow();
            snapshot.translate(&graphene::Point::new(
                pos.pos.0 as f32,
                (pos.pos.1 + pos.y_offset) as f32,
            ));
            snapshot.push_opacity(
                self.blink
                    .borrow()
                    .as_ref()
                    .map(|blink| blink.alpha)
                    .unwrap_or(1.0),
            );

            snapshot.append_node(node);

            snapshot.pop();
        }
    }

    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        match orientation {
            gtk::Orientation::Horizontal => {
                // width
                let len = self.double_width.borrow().then(|| 2.0).unwrap_or(1.0);
                let font = self.font.borrow();
                let w = len * (font.char_width() / SCALE);
                let w = w.ceil() as i32;

                (w, w, -1, -1)
            }
            gtk::Orientation::Vertical => {
                // height
                let font = self.font.borrow();
                let h = font.height() / SCALE;
                let h = h.ceil() as i32;

                (h, h, -1, -1)
            }
            _ => self.parent_measure(orientation, for_size),
        }
    }
}
