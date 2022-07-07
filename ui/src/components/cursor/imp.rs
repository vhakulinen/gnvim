use std::cell::{Cell, RefCell};

use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk, prelude::*};

use crate::font::Font;
use crate::mode_info::ModeInfo;
use crate::SCALE;

use super::blink::Blink;

#[derive(Default)]
pub struct Cursor {
    pub pos: RefCell<(i64, i64)>,
    pub text: RefCell<String>,
    pub double_width: RefCell<bool>,

    pub node: RefCell<Option<gsk::RenderNode>>,

    pub width_percentage: RefCell<f32>,
    pub attr_id: RefCell<i64>,

    pub active: Cell<bool>,
    pub busy: Cell<bool>,

    pub font: RefCell<Font>,
    pub blink: RefCell<Option<Blink>>,
    pub blink_transition: Cell<f64>,
    /// Callback id to our function that makes the cursor blink.
    pub blink_tick: RefCell<Option<gtk::TickCallbackId>>,
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
    fn set_blink(&self, obj: &super::Cursor, blink: Option<Blink>) {
        if let Some(id) = self.blink_tick.borrow_mut().take() {
            id.remove();
        }

        self.blink.replace(blink);
        if self.blink.borrow().is_none() {
            return;
        }

        let new_id = obj.add_tick_callback(
            clone!(@weak obj => @default-return Continue(false), move |_, clock| {
                let imp = obj.imp();
                if imp.active.get() || !imp.busy.get() {
                    if let Some(ref mut blink) = *imp.blink.borrow_mut() {
                        blink.tick(clock.frame_time() as f64);
                    }
                }

                obj.queue_draw();
                Continue(true)
            }),
        );

        self.blink_tick.replace(Some(new_id));
    }
}

impl ObjectImpl for Cursor {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // Set the default width.
        self.width_percentage.replace(1.0);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::builder("font", Font::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("active")
                    .default_value(false)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("busy")
                    .default_value(false)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoxed::builder("mode-info", ModeInfo::static_type())
                    .flags(glib::ParamFlags::WRITABLE)
                    .build(),
                glib::ParamSpecDouble::builder("blink-transition")
                    .minimum(0.0)
                    .default_value(160.0)
                    .flags(glib::ParamFlags::WRITABLE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "font" => self.font.borrow().to_value(),
            "active" => self.active.get().to_value(),
            "busy" => self.busy.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be object Font"));
                obj.queue_draw();
            }
            "active" => {
                self.active
                    .set(value.get().expect("active must be a boolean"));
                obj.queue_draw();
            }
            "busy" => {
                self.busy.set(value.get().expect("busy must be a boolean"));
                obj.queue_draw();
            }
            "mode-info" => {
                let mode: ModeInfo = value.get().expect("mode-info must be an ModeInfo object");

                let cell_percentage = mode
                    .cell_percentage
                    // Make sure we have non 0 value.
                    .map(|v| if v == 0 { 100 } else { v })
                    .map(|v| v as f32 / 100.0)
                    .unwrap_or(100.0);

                self.width_percentage.replace(cell_percentage);
                self.attr_id.replace(mode.attr_id.unwrap_or(0) as i64);

                self.set_blink(
                    obj,
                    Blink::new(
                        mode.blinkwait.unwrap_or(0) as f64 * 1000.0,
                        mode.blinkon.unwrap_or(0) as f64 * 1000.0,
                        mode.blinkoff.unwrap_or(0) as f64 * 1000.0,
                        self.blink_transition.get(),
                        obj.frame_clock()
                            .map(|click| click.frame_time() as f64)
                            .unwrap_or(0.0),
                    ),
                );

                self.node.replace(None);
            }
            "blink-transition" => {
                let transition =
                    value.get::<f64>().expect("blink-transition must be a f64") * 1000.0;

                let (wait, on, off) = self
                    .blink
                    .borrow()
                    .as_ref()
                    .map(|blink| (blink.wait(), blink.on(), blink.off()))
                    .unwrap_or_default();

                self.blink_transition.set(transition);
                self.set_blink(
                    obj,
                    Blink::new(
                        wait,
                        on,
                        off,
                        transition,
                        obj.frame_clock()
                            .map(|click| click.frame_time() as f64)
                            .unwrap_or(0.0),
                    ),
                );
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Cursor {
    fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk::Snapshot) {
        if self.busy.get() || !self.active.get() {
            return;
        }

        if let Some(ref node) = *self.node.borrow() {
            let node = gsk::OpacityNode::new(
                node,
                self.blink
                    .borrow()
                    .as_ref()
                    .map(|blink| blink.alpha as f32)
                    .unwrap_or(1.0),
            );
            snapshot.append_node(node);
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
            _ => self.parent_measure(widget, orientation, for_size),
        }
    }
}
