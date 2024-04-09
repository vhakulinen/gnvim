use std::cell::Cell;

use gtk::{glib::subclass::InitializingObject, prelude::*, subclass::prelude::*};

use crate::components::Popupmenu;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/cmdline.ui")]
pub struct Cmdline {
    #[template_child(id = "input")]
    pub input: TemplateChild<gtk::Box>,
    #[template_child(id = "main")]
    pub main: TemplateChild<gtk::TextView>,
    #[template_child(id = "block")]
    pub block: TemplateChild<gtk::TextView>,
    #[template_child(id = "completion")]
    pub popupmenu: TemplateChild<Popupmenu>,

    pub prompt_len: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Cmdline {
    const NAME: &'static str = "Cmdline";
    type Type = super::Cmdline;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Popupmenu::ensure_type();

        klass.set_css_name("cmdline");
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Cmdline {
    fn dispose(&self) {
        self.dispose_template();
    }
}

impl WidgetImpl for Cmdline {}
