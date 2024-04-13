use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::boxed::ShowTabline;
use crate::nvim::Neovim;

#[derive(Default, glib::Properties, gtk::CompositeTemplate)]
#[template(resource = "/com/github/vhakulinen/gnvim/tabline.ui")]
#[properties(wrapper_type = super::Tabline)]
pub struct Tabline {
    #[template_child(id = "content")]
    pub content: TemplateChild<gtk::Box>,

    #[property(get, set)]
    pub nvim: RefCell<Neovim>,
    #[property(set)]
    pub show: RefCell<ShowTabline>,
}

#[glib::object_subclass]
impl ObjectSubclass for Tabline {
    const NAME: &'static str = "Tabline";
    type Type = super::Tabline;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
        klass.set_css_name("tabline");

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Tabline {
    fn dispose(&self) {
        self.dispose_template();
    }
}

impl WidgetImpl for Tabline {}
