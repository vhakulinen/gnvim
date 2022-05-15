use std::cell::RefCell;

use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::components::grid::Grid;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/shell.ui")]
pub struct Shell {
    #[template_child(id = "msg-fixed")]
    pub msg_fixed: TemplateChild<gtk::Fixed>,
    #[template_child(id = "root-grid")]
    pub root_grid: TemplateChild<Grid>,

    pub grids: RefCell<Vec<Grid>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Shell {
    const NAME: &'static str = "Shell";
    type Type = super::Shell;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Grid::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Shell {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.grids.borrow_mut().push(self.root_grid.clone());
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

        let mut child: Option<gtk::Widget> = widget.first_child();
        while let Some(sib) = child {
            if sib.should_layout() {
                sib.allocate(width, height, -1, None);
            }

            child = sib.next_sibling();
        }
    }
}
