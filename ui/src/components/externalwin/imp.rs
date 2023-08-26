use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use nvim::NeovimApi;

use crate::{components::Grid, spawn_local};

#[derive(Default)]
pub struct ExternalWindow {
    pub grid: RefCell<Grid>,
    pub main_window: RefCell<gtk::Window>,

    resize_id: Rc<Cell<Option<glib::SourceId>>>,
    prev_win_size: Cell<(i32, i32)>,
}

impl ExternalWindow {
    fn resize_nvim(&self, obj: &super::ExternalWindow) {
        let (cols, rows) = self
            .grid
            .borrow()
            .font()
            .grid_size_for_allocation(&obj.allocation());

        let id = glib::timeout_add_local(
            Duration::from_millis(crate::WINDOW_RESIZE_DEBOUNCE_MS),
            clone!(@weak obj => @default-return glib::ControlFlow::Break, move || {
                spawn_local!(clone!(@weak obj => async move {
                    let res = obj
                        .nvim()
                        .nvim_ui_try_resize_grid(
                            obj.grid_id(),
                            cols.max(1) as i64, rows.max(1) as i64)
                        .await
                        .unwrap();

                    res.await.expect("nvim_ui_try_resize failed");
                }));

                // Clear after our selves, so we don't try to remove
                // our id once we're already done.
                obj.imp().resize_id.replace(None);

                glib::ControlFlow::Break
            }),
        );

        // Cancel the earlier timeout if it exists.
        if let Some(id) = self.resize_id.replace(Some(id)).take() {
            id.remove();
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ExternalWindow {
    const NAME: &'static str = "ExternalWindow";
    type Type = super::ExternalWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for ExternalWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Override css classes.
        obj.set_property("css-classes", vec!["external-window"].to_value());

        // NOTE(ville): The root interface implementation does some extra magic
        // compared to the GtkWindow's own set_focus function, so use the root
        // one. Also, at the time of writing, `obj.set_focus` _would_ call the
        // root interface, make the call explicit for clarity.
        gtk::Root::set_focus(obj.upcast_ref(), Some(&*self.main_window.borrow()));
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::builder::<gtk::Window>("main-window")
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY)
                    .build(),
                glib::ParamSpecObject::builder::<Grid>("grid")
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "grid" => self.grid.borrow().to_value(),
            "main-window" => self.main_window.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "grid" => {
                let grid = value.get().expect("grid value must be an Grid object");
                self.obj().set_child(Some(&grid));
                self.grid.replace(grid);
            }
            "main-window" => {
                self.main_window.replace(
                    value
                        .get()
                        .expect("main-window must be an gtk::Window object"),
                );
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for ExternalWindow {
    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        let prev = self.prev_win_size.get();
        // TODO(ville): Check for rows/col instead.
        // NOTE(ville): If we try to resize nvim unconditionally, we'll
        // end up in a infinite loop.
        if prev != (width, height) {
            self.prev_win_size.set((width, height));
            self.resize_nvim(&self.obj());
        }
    }

    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        let (mw, nw, mb, nb) = self.grid.borrow().measure(orientation, for_size);
        (mw.min(1), nw, mb, nb)
    }
}

impl WindowImpl for ExternalWindow {}
