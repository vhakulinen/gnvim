use gtk::{
    glib::{self, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
};

use crate::{child_iter::IterChildren, font::Font};

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/popupmenu_row.ui")]
pub struct Row {
    #[template_child(id = "word")]
    pub word: TemplateChild<gtk::Label>,
    #[template_child(id = "kind")]
    pub kind: TemplateChild<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for Row {
    const NAME: &'static str = "PopupmenuRow";
    type Type = super::Row;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BoxLayout>();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Row {
    fn constructed(&self) {
        self.parent_constructed();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecObject::builder::<Font>("font")
                .flags(glib::ParamFlags::WRITABLE)
                .build()]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
        // NOTE(ville): Our only property is write only, so it shouldn't be read.
        unimplemented!()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "font" => {
                let font: Font = value.get().expect("font value must be object Font");

                // Propagate the font onwards.
                self.obj().iter_children().for_each(|child| {
                    child
                        .pango_context()
                        .set_font_description(Some(&font.font_desc()));
                });
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Row {}

impl BoxImpl for Row {}
