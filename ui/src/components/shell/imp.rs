use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::boxed::ModeInfo;
use crate::components::grid::Grid;
use crate::components::{Fixedz, MsgWin, Popupmenu};
use crate::font::Font;
use crate::nvim::Neovim;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/shell.ui")]
pub struct Shell {
    /// Container to mainly place floating window.
    #[template_child(id = "windows")]
    pub fixed: TemplateChild<Fixedz>,
    /// The root grid.
    #[template_child(id = "root-grid")]
    pub root_grid: TemplateChild<Grid>,
    /// The message window.
    ///
    /// Note that the window it self is not direct child of the shell,
    /// but instead child of `windows_container`.
    #[template_child(id = "msg-win")]
    pub msg_win: TemplateChild<MsgWin>,

    #[template_child(id = "popupmenu")]
    pub popupmenu: TemplateChild<Popupmenu>,

    pub nvim: RefCell<Neovim>,

    pub grids: RefCell<Vec<Grid>>,
    /// Current grid.
    ///
    /// On startup this will be an invalid grid, but the first cursor goto
    /// event will fix that.
    pub current_grid: RefCell<Grid>,
    pub font: RefCell<Font>,
    pub busy: Cell<bool>,
    pub current_mode_info: RefCell<ModeInfo>,
    pub cursor_blink_transition: Cell<f64>,
    pub cursor_position_transition: Cell<f64>,
    pub scroll_transition: Cell<f64>,
    /// Source id for debouncing nvim resizing.
    pub resize_id: RefCell<Option<glib::SourceId>>,
    /// Our previous size. Used to track when we need to tell neovim to resize
    /// itself.
    pub prev_size: Cell<(i32, i32)>,
}

#[glib::object_subclass]
impl ObjectSubclass for Shell {
    const NAME: &'static str = "Shell";
    type Type = super::Shell;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Grid::ensure_type();
        MsgWin::ensure_type();
        Fixedz::ensure_type();
        Popupmenu::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Shell {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // TODO(ville): To avoid duplication of the property binding code
        // between the ui file and the grid creation code, perhaps the root
        // grid should be created here through code.
        // Add the root grid to the grids list.
        self.grids.borrow_mut().push(self.root_grid.clone());
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::builder("font", Font::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecObject::builder("nvim", Neovim::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("busy")
                    .default_value(false)
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoxed::builder("current-mode-info", ModeInfo::static_type())
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
            "font" => self.font.borrow().to_value(),
            "busy" => self.busy.get().to_value(),
            "nvim" => self.nvim.borrow().to_value(),
            "current-mode-info" => self.current_mode_info.borrow().to_value(),
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
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be an Font object"));
            }
            "busy" => self
                .busy
                .set(value.get().expect("busy value needs to be a bool")),
            "nvim" => {
                self.nvim.replace(
                    value
                        .get()
                        .expect("nvim value needs to be an Neovim object"),
                );
            }
            "current-mode-info" => {
                self.current_mode_info.replace(
                    value
                        .get()
                        .expect("current-mode-info must be an ModeInfo object"),
                );
            }
            "cursor-blink-transition" => self.cursor_blink_transition.set(
                value
                    .get()
                    .expect("cursor-blink-transition value must be a f64"),
            ),
            "cursor-position-transition" => self.cursor_position_transition.set(
                value
                    .get()
                    .expect("cursor-position-transition value must be a f64"),
            ),
            "scroll-transition" => self
                .scroll_transition
                .set(value.get().expect("scroll-transition value must be a f64")),
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Shell {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        // Currently, the shell's size is the same as the root grid's size.
        // Note that for the min width we need to report something smaller so
        // that the top level window remains resizable (since its using the
        // shell as the root widget).
        let (mw, nw, mb, nb) = self.root_grid.measure(orientation, for_size);
        (mw.min(1), nw, mb, nb)
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        self.root_grid.allocate(width, height, -1, None);
        self.fixed.allocate(width, height, -1, None);

        let prev = self.prev_size.get();
        // TODO(ville): Check for rows/col instead.
        // NOTE(ville): If we try to resize nvim unconditionally, we'll
        // end up in a infinite loop.
        if prev != (width, height) {
            self.prev_size.set((width, height));
            widget.resize_nvim();
        }
    }
}
