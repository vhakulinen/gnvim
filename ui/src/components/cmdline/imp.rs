use std::cell::Cell;

use gtk::{glib::subclass::InitializingObject, prelude::*, subclass::prelude::*};

use crate::components::Popupmenu;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/cmdline.ui")]
pub struct Cmdline {
    #[template_child(id = "container")]
    pub container: TemplateChild<gtk::Box>,
    #[template_child(id = "top")]
    pub top: TemplateChild<gtk::Box>,
    #[template_child(id = "main")]
    pub main: TemplateChild<gtk::TextView>,
    #[template_child(id = "block")]
    pub block: TemplateChild<gtk::TextView>,
    #[template_child(id = "completion")]
    pub popupmenu: TemplateChild<Popupmenu>,

    pub max_height: Cell<i32>,

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
    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecInt::builder("max-height")
                .flags(glib::ParamFlags::READWRITE)
                .build()]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "max-height" => self.max_height.get().to_value(),
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
            "max-height" => {
                self.max_height
                    .set(value.get().expect("max-height must be a i32"));
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Cmdline {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        self.container.measure(orientation, for_size)
    }

    fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.container.allocate(width, height, baseline, None);

        let (_, req) = self.top.preferred_size();
        let max_h = self.max_height.get();
        self.popupmenu.set_max_height(max_h - req.height());
        self.popupmenu.set_max_width(req.width());
    }
}
