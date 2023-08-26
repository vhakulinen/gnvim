use std::cell::{Cell, RefCell};

use glib::clone;
use gtk::glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk};
use gtk::{graphene, prelude::*};

use crate::boxed::ModeInfo;
use crate::components::grid::Grid;
use crate::components::{Fixedz, MsgWin, Popupmenu};
use crate::font::Font;
use crate::nvim::Neovim;
use crate::SCALE;

#[derive(Default)]
pub struct PopupmenuPos {
    row: i64,
    col: i64,
    visible: bool,
    grid: i64,
}

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

    #[property(name = "pmenu-col", member = col, get, set, type = i64)]
    #[property(name = "pmenu-row", member = row, get, set, type = i64)]
    #[property(name = "pmenu-visible", member = visible, get, set, type = bool)]
    #[property(name = "pmenu-grid", member = grid, get, set, type = i64)]
    pub pmenu_pos: RefCell<PopupmenuPos>,
}

impl Shell {
    /// Adjust (or set) the popupmenu position.
    pub fn adjust_pmenu(&self) {
        let pmenu_pos = self.pmenu_pos.borrow();

        if !pmenu_pos.visible {
            return;
        }

        let font = self.font.borrow();

        let pos = if pmenu_pos.grid == 1 {
            gsk::Transform::new()
        } else {
            self.fixed
                .child_position(&find_grid_or_return!(self.obj(), pmenu_pos.grid))
        }
        .transform_point(&graphene::Point::new(
            font.col_to_x(pmenu_pos.col as f64) as f32,
            font.row_to_y(pmenu_pos.row as f64 + 1.0) as f32,
        ));

        let (_, req) = self.root_grid.preferred_size();
        let max_w = req.width() as f32;
        // Make sure the msg window and the popupmenu won't overlap.
        let max_h = (req.height() - self.msg_win.height()) as f32;
        let (x, y) = (pos.x(), pos.y());
        let below = max_h - y;
        let above = max_h - below - font.height() / SCALE;

        let (_, req) = self.popupmenu.listview_preferred_size();
        let (pmenu_w, pmenu_h) = (req.width() as f32, req.height() as f32);

        // TODO(ville): Would be nice to make the popupmenu to retain its
        // placement (e.g. above vs. below) when the popupmenu is already
        // shown and displayed in a way where it has enough space.
        let (y, max_h) = if pmenu_h > below && above > below {
            // Place above.
            ((y - font.height() / SCALE - pmenu_h).max(0.0), above)
        } else {
            // Place below.
            (y, below)
        };

        let x = if x + pmenu_w > max_w {
            (max_w - pmenu_w).max(0.0)
        } else {
            // Adjust for padding when not overflowing to the right.
            x - self.popupmenu.get_padding_x()
        };

        self.popupmenu.set_max_width(max_w.floor() as i32);
        self.popupmenu.set_max_height(max_h.floor() as i32);
        self.fixed.move_(&*self.popupmenu, x, y);

        self.popupmenu.report_pum_bounds(&self.nvim.borrow(), x, y);
    }
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

#[glib::derived_properties]
impl ObjectImpl for Shell {
    fn constructed(&self) {
        self.parent_constructed();

        // TODO(ville): To avoid duplication of the property binding code
        // between the ui file and the grid creation code, perhaps the root
        // grid should be created here through code.
        // Add the root grid to the grids list.
        self.grids.borrow_mut().push(self.root_grid.clone());

        let obj = self.obj();
        self.popupmenu
            .store()
            .connect_items_changed(clone!(@weak obj => move |_, _, _, _| {
                obj.imp().adjust_pmenu();
            }));
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
