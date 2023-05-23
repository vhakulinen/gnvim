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

#[derive(gtk::CompositeTemplate, glib::Properties, Default)]
#[properties(wrapper_type = super::Shell)]
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

    #[property(get, set)]
    pub nvim: RefCell<Neovim>,

    pub grids: RefCell<Vec<Grid>>,
    /// Current grid.
    ///
    /// On startup this will be an invalid grid, but the first cursor goto
    /// event will fix that.
    pub current_grid: RefCell<Grid>,
    #[property(get, set)]
    pub font: RefCell<Font>,
    #[property(get, set, default = false)]
    pub busy: Cell<bool>,
    #[property(get, set)]
    pub current_mode_info: RefCell<ModeInfo>,
    #[property(get, set, minimum = 0.0)]
    pub cursor_blink_transition: Cell<f64>,
    #[property(get, set, minimum = 0.0)]
    pub cursor_position_transition: Cell<f64>,
    #[property(get, set, minimum = 0.0)]
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
    fn constructed(&self) {
        self.parent_constructed();

        // TODO(ville): To avoid duplication of the property binding code
        // between the ui file and the grid creation code, perhaps the root
        // grid should be created here through code.
        // Add the root grid to the grids list.
        self.grids.borrow_mut().push(self.root_grid.clone());
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

impl WidgetImpl for Shell {
    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        // Currently, the shell's size is the same as the root grid's size.
        // Note that for the min width we need to report something smaller so
        // that the top level window remains resizable (since its using the
        // shell as the root widget).
        let (mw, nw, mb, nb) = self.root_grid.measure(orientation, for_size);
        (mw.min(1), nw, mb, nb)
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        self.root_grid.allocate(width, height, -1, None);
        self.fixed.allocate(width, height, -1, None);

        let prev = self.prev_size.get();
        // TODO(ville): Check for rows/col instead.
        // NOTE(ville): If we try to resize nvim unconditionally, we'll
        // end up in a infinite loop.
        if prev != (width, height) {
            self.prev_size.set((width, height));
            self.obj().resize_nvim();
        }
    }
}
