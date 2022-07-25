use std::cell::{Cell, RefCell};

use gtk::glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
};
use nvim::types::Window;

use crate::boxed::ModeInfo;
use crate::components::{Cursor, ExternalWindow, GridBuffer};
use crate::font::Font;
use crate::nvim::Neovim;
use crate::spawn_local;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/grid.ui")]
pub struct Grid {
    /// Our cursor on the screen.
    #[template_child(id = "cursor")]
    pub cursor: TemplateChild<Cursor>,
    /// The content.
    #[template_child(id = "buffer")]
    pub buffer: TemplateChild<GridBuffer>,

    pub font: RefCell<Font>,

    /// The grid id from neovim.
    pub id: Cell<i64>,
    /// Neovim window associated to this grid.
    pub nvim_window: RefCell<Option<Window>>,
    pub nvim: RefCell<Neovim>,
    /// If grid is the active grid or not.
    pub active: Cell<bool>,
    pub busy: Cell<bool>,
    pub mode_info: RefCell<ModeInfo>,

    pub external_win: RefCell<Option<ExternalWindow>>,
    pub gesture_click: gtk::GestureClick,
    pub gesture_drag: gtk::GestureDrag,
    pub event_controller_scroll: gtk::EventControllerScroll,
    pub event_controller_motion: gtk::EventControllerMotion,
    pub cursor_blink_transition: Cell<f64>,
    pub cursor_position_transition: Cell<f64>,
    pub scroll_transition: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for Grid {
    const NAME: &'static str = "Grid";
    type Type = super::Grid;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        GridBuffer::ensure_type();
        Cursor::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Grid {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.gesture_click.set_button(0);
        self.gesture_drag.set_button(0);
        let mut flags = gtk::EventControllerScrollFlags::empty();
        flags.insert(gtk::EventControllerScrollFlags::DISCRETE);
        flags.insert(gtk::EventControllerScrollFlags::BOTH_AXES);
        self.event_controller_scroll.set_flags(flags);

        // Add our event handlers to the buffer widget.
        self.buffer.add_controller(&self.gesture_click);
        self.buffer.add_controller(&self.gesture_drag);
        self.buffer.add_controller(&self.event_controller_scroll);
        self.buffer.add_controller(&self.event_controller_motion);

        // Connect mouse events.
        obj.connect_mouse(
            clone!(@weak obj => move |id, mouse, action, modifier, row, col| {
                spawn_local!(async move {
                    let res = obj
                        .nvim()
                        .client()
                        .await
                        .nvim_input_mouse(
                            mouse.as_nvim_input().to_owned(),
                            action.as_nvim_action().to_owned(),
                            modifier,
                            id,
                            row as i64,
                            col as i64,
                        )
                        .await.expect("call to nvim failed");

                    res.await.expect("nvim_input_mouse failed");
                });
            }),
        )
    }

    fn dispose(&self, _obj: &Self::Type) {
        self.buffer.unparent();
        self.cursor.unparent();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecInt64::builder("grid-id")
                    .default_value(0)
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY)
                    .build(),
                glib::ParamSpecObject::builder("nvim", Neovim::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecObject::builder("font", Font::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("busy")
                    .default_value(false)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("active")
                    .default_value(false)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoxed::builder("mode-info", ModeInfo::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecDouble::builder("cursor-blink-transition")
                    .minimum(0.0)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecDouble::builder("cursor-position-transition")
                    .minimum(0.0)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecDouble::builder("scroll-transition")
                    .minimum(0.0)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "grid-id" => self.id.get().to_value(),
            "font" => self.font.borrow().to_value(),
            "nvim" => self.nvim.borrow().to_value(),
            "busy" => self.busy.get().to_value(),
            "active" => self.active.get().to_value(),
            "mode-info" => self.mode_info.borrow().to_value(),
            "cursor-blink-transition" => self.cursor_blink_transition.get().to_value(),
            "cursor-position-transition" => self.cursor_position_transition.get().to_value(),
            "scroll-transition" => self.scroll_transition.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "grid-id" => {
                let id = value.get().expect("property `grid-id` needs to be i64");
                self.id.replace(id);
            }
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be object Font"));
            }
            "nvim" => {
                self.nvim
                    .replace(value.get().expect("font value must be object Neovim"));
            }
            "busy" => self
                .busy
                .set(value.get().expect("busy value must be a boolean")),
            "active" => self
                .active
                .set(value.get().expect("active value must be a boolean")),
            "mode-info" => {
                self.mode_info.replace(
                    value
                        .get()
                        .expect("mode-info needs to be an ModeInfo object"),
                );
            }
            "cursor-blink-transition" => self.cursor_blink_transition.set(
                value
                    .get()
                    .expect("cursor-blink-transition value needs to be a f64"),
            ),
            "cursor-position-transition" => self.cursor_position_transition.set(
                value
                    .get()
                    .expect("cursor-position-transition value needs to be a f64"),
            ),
            "scroll-transition" => self.scroll_transition.set(
                value
                    .get()
                    .expect("scroll-transition value needs to be a f64"),
            ),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for Grid {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        // NOTE(ville): Currently, our size is always the same as our buffer's
        // size. This manual measure implementation is to avoid issues where
        // the buffer's sibling, gtkfixed, has "old" size on flush.
        self.buffer.measure(orientation, for_size)
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        let mut child: Option<gtk::Widget> = widget.first_child();
        while let Some(sib) = child {
            if sib.should_layout() {
                let (req, _) = sib.preferred_size();
                sib.allocate(req.width(), req.height(), -1, None);
            }

            child = sib.next_sibling();
        }
    }
}
