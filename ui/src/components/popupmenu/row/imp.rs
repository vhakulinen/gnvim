use std::cell::RefCell;

use gtk::{
    glib::{self, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
};

use crate::{child_iter::IterChildren, font::Font, SCALE};

#[derive(gtk::CompositeTemplate, glib::Properties, Default)]
#[properties(wrapper_type = super::Row)]
#[template(resource = "/com/github/vhakulinen/gnvim/popupmenu_row.ui")]
pub struct Row {
    #[template_child(id = "word")]
    pub word: TemplateChild<gtk::Label>,
    #[template_child(id = "kind")]
    pub kind: TemplateChild<gtk::Label>,

    #[property(set = Self::set_font)]
    pub font: RefCell<Font>,

    #[property(get, set)]
    pub word_label: RefCell<String>,
    #[property(get, set)]
    pub kind_markup: RefCell<String>,
}

impl Row {
    fn set_font(&self, font: Font) {
        let w = (font.char_width() / SCALE).ceil() as i32;

        // Adjust margins and spacing.
        self.obj().set_margin_start(w);
        self.obj().set_margin_end(w);
        self.obj().set_spacing(w);

        // Propagate the font onwards.
        self.obj().iter_children().for_each(|child| {
            child
                .pango_context()
                .set_font_description(Some(&font.font_desc()));
        });

        self.font.replace(font);
    }
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

#[glib::derived_properties]
impl ObjectImpl for Row {}

impl WidgetImpl for Row {}

impl BoxImpl for Row {}
