use std::cell::RefCell;

use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::components::grid::Grid;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/shell.ui")]
pub struct Shell {
    #[template_child(id = "fixed")]
    pub fixed: TemplateChild<gtk::Fixed>,
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

        // Set our layout manager.
        klass.set_layout_manager_type::<gtk::BinLayout>();

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

impl WidgetImpl for Shell {}
