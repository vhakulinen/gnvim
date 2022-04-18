use gtk::glib::subclass::InitializingObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::components::grid::Grid;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/shell.ui")]
pub struct Shell {
    #[template_child(id = "overlay")]
    overlay: TemplateChild<gtk::Overlay>,
    #[template_child(id = "fixed")]
    fixed: TemplateChild<gtk::Fixed>,
    #[template_child(id = "root-grid")]
    pub root_grid: TemplateChild<Grid>,
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
    }

    fn dispose(&self, _obj: &Self::Type) {
        // The overlay widget got added into our widget/layout manager by
        // the template, so we need to remove it.
        self.overlay.unparent();
    }
}

impl WidgetImpl for Shell { }
