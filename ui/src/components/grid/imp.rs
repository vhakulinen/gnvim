use std::cell::{Cell, RefCell};

use gtk::glib::subclass::InitializingObject;
use gtk::graphene;
use gtk::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
};
use nvim::types::Window;
use nvim::NeovimApi;

use crate::boxed::ModeInfo;
use crate::components::{cursor, Cursor, ExternalWindow, GridBuffer};
use crate::font::Font;
use crate::nvim::Neovim;
use crate::spawn_local;

#[derive(gtk::CompositeTemplate, glib::Properties, Default)]
#[properties(wrapper_type = super::Grid)]
#[template(resource = "/com/github/vhakulinen/gnvim/grid.ui")]
pub struct Grid {
    /// Our cursor on the screen.
    /// TODO(ville): fix the cursor to render within the window margins.
    #[template_child(id = "cursor")]
    pub cursor: TemplateChild<Cursor>,
    /// The content.
    #[template_child(id = "buffer")]
    pub buffer: TemplateChild<GridBuffer>,

    #[property(get, set)]
    pub font: RefCell<Font>,

    /// The grid id from neovim.
    #[property(name = "grid-id", construct_only, get, set, default = 0)]
    pub id: Cell<i64>,
    /// Neovim window associated to this grid.
    pub nvim_window: RefCell<Option<Window>>,
    #[property(get, set)]
    pub nvim: RefCell<Neovim>,
    /// If grid is the active grid or not.
    #[property(get, set, default = false)]
    pub active: Cell<bool>,
    #[property(get, set, default = false)]
    pub busy: Cell<bool>,
    #[property(get, set)]
    pub mode_info: RefCell<ModeInfo>,

    pub external_win: RefCell<Option<ExternalWindow>>,
    pub gesture_click: gtk::GestureClick,
    pub gesture_drag: gtk::GestureDrag,
    pub event_controller_scroll: gtk::EventControllerScroll,
    pub event_controller_motion: gtk::EventControllerMotion,

    /// The cursor blink animation speed.
    #[property(get, set, minimum = 0.0)]
    pub cursor_blink_transition: Cell<f64>,
    /// The cursor position animation speed.
    #[property(get, set, minimum = 0.0)]
    pub cursor_position_transition: Cell<f64>,
    /// The scroll animation speed.
    #[property(get, set, minimum = 0.0)]
    pub scroll_transition: Cell<f64>,
}

#[gtk::template_callbacks(functions)]
impl Grid {
    #[template_callback]
    fn multiply(a: f64, b: f64) -> f64 {
        a * b
    }

    #[template_callback]
    fn cursor_width(mode: &ModeInfo) -> f32 {
        mode.cell_percentage
            // Make sure we have non 0 value.
            .map(|v| if v == 0 { 100 } else { v })
            .map(|v| v as f32 / 100.0)
            .unwrap_or(100.0)
    }

    #[template_callback]
    fn cursor_attr_id(mode: &ModeInfo) -> i64 {
        mode.attr_id.unwrap_or(0) as i64
    }

    #[template_callback(function = false)]
    fn cursor_blink(&self, mode: &ModeInfo, transition: f64) -> Option<cursor::Blink> {
        cursor::Blink::new(
            mode.blinkwait.unwrap_or(0) as f64 * 1000.0,
            mode.blinkon.unwrap_or(0) as f64 * 1000.0,
            mode.blinkoff.unwrap_or(0) as f64 * 1000.0,
            transition * 1000.0,
            self.obj()
                .frame_clock()
                .map(|clock| clock.frame_time() as f64)
                .unwrap_or(0.0),
        )
    }
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
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Grid {
    fn constructed(&self) {
        self.parent_constructed();

        self.gesture_click.set_button(0);
        self.gesture_drag.set_button(0);
        let mut flags = gtk::EventControllerScrollFlags::empty();
        flags.insert(gtk::EventControllerScrollFlags::DISCRETE);
        flags.insert(gtk::EventControllerScrollFlags::BOTH_AXES);
        self.event_controller_scroll.set_flags(flags);

        // Add our event handlers to the buffer widget.
        self.buffer.add_controller(self.gesture_click.clone());
        self.buffer.add_controller(self.gesture_drag.clone());
        self.buffer
            .add_controller(self.event_controller_scroll.clone());
        self.buffer
            .add_controller(self.event_controller_motion.clone());

        // Connect mouse events.
        let obj = self.obj();
        obj.connect_mouse(
            clone!(@weak obj => move |id, mouse, action, modifier, row, col| {
                spawn_local!(async move {
                    obj
                        .nvim()
                        .nvim_input_mouse(
                            mouse.as_nvim_input(),
                            action.as_nvim_action(),
                            &modifier,
                            id,
                            row as i64,
                            col as i64,
                        )
                        .await
                        .expect("nvim_input_mouse failed");
                });
            }),
        )
    }

    fn dispose(&self) {
        self.buffer.unparent();
        self.cursor.unparent();
    }
}

impl WidgetImpl for Grid {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let (_, req) = self.obj().preferred_size();

        snapshot.push_clip(&graphene::Rect::new(
            0.0,
            0.0,
            req.width() as f32,
            req.height() as f32,
        ));

        self.parent_snapshot(snapshot);

        snapshot.pop();
    }

    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        // NOTE(ville): Currently, our size is always the same as our buffer's
        // size. This manual measure implementation is to avoid issues where
        // the buffer's sibling, gtkfixed, has "old" size on flush.
        self.buffer.measure(orientation, for_size)
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        let mut child: Option<gtk::Widget> = self.obj().first_child();
        while let Some(sib) = child {
            if sib.should_layout() {
                let (req, _) = sib.preferred_size();
                sib.allocate(req.width(), req.height(), -1, None);
            }

            child = sib.next_sibling();
        }
    }
}
