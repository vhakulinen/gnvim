use std::cell::{Cell, RefCell};

use gtk::glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*};
use nvim::types::Window;

use crate::components::{Cursor, GridBuffer};
use crate::font::Font;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/grid.ui")]
pub struct Grid {
    /// Our cursor on the screen.
    #[template_child(id = "cursor")]
    pub cursor: TemplateChild<Cursor>,
    #[template_child(id = "fixed")]
    pub fixed: TemplateChild<gtk::Fixed>,
    /// The content.
    #[template_child(id = "buffer")]
    pub buffer: TemplateChild<GridBuffer>,

    pub font: RefCell<Font>,

    /// The grid id from neovim.
    pub id: Cell<i64>,
    /// Neovim window associated to this grid.
    pub nvim_window: RefCell<Option<Window>>,
    /// If grid is the active grid or not.
    pub active: Cell<bool>,
    pub busy: Cell<bool>,

    pub gesture_click: gtk::GestureClick,
    pub gesture_drag: gtk::GestureDrag,
    pub event_controller_scroll: gtk::EventControllerScroll,
    pub event_controller_motion: gtk::EventControllerMotion,
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

        obj.add_controller(&self.gesture_click);
        obj.add_controller(&self.gesture_drag);
        obj.add_controller(&self.event_controller_scroll);
        obj.add_controller(&self.event_controller_motion);
    }

    fn dispose(&self, _obj: &Self::Type) {
        self.buffer.unparent();
        self.cursor.unparent();
        self.fixed.unparent();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecInt64::new(
                    "grid-id",
                    "Grid ID",
                    "Grid ID",
                    i64::MIN,
                    i64::MAX,
                    0,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                ),
                glib::ParamSpecObject::new(
                    "font",
                    "font",
                    "Font",
                    Font::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoolean::new(
                    "busy",
                    "Busy",
                    "Busy",
                    false,
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoolean::new(
                    "active",
                    "active",
                    "Active",
                    false,
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "grid-id" => self.id.get().to_value(),
            "font" => self.font.borrow().to_value(),
            "busy" => self.busy.get().to_value(),
            "active" => self.active.get().to_value(),
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
            "busy" => self
                .busy
                .set(value.get().expect("busy value must be a boolean")),
            "active" => self
                .active
                .set(value.get().expect("active value must be a boolean")),
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
