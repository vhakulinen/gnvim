use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::boxed::ShowTabline;
use crate::nvim::Neovim;

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/com/github/vhakulinen/gnvim/tabline.ui")]
pub struct Tabline {
    #[template_child(id = "scrolled-window")]
    pub scrolledwindow: TemplateChild<gtk::ScrolledWindow>,
    #[template_child(id = "content")]
    pub content: TemplateChild<gtk::Box>,

    pub nvim: RefCell<Neovim>,
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

impl ObjectImpl for Tabline {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.scrolledwindow
            .child()
            .expect("failed to get scrolledwindow's child")
            .downcast::<gtk::Viewport>()
            .expect("child is not GtkViewport")
            .set_scroll_to_focus(true);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::builder("nvim", Neovim::static_type())
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoxed::builder("show", ShowTabline::static_type())
                    .flags(glib::ParamFlags::WRITABLE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "nvim" => self.nvim.borrow().to_value(),
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
            "nvim" => {
                self.nvim.replace(
                    value
                        .get()
                        .expect("nvim value needs to be an Neovim object"),
                );
            }
            "show" => {
                self.show
                    .replace(value.get().expect("font value must be a ShowTabline"));
            }

            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Tabline {}
