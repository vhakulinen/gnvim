use gtk::{glib::subclass::InitializingObject, prelude::*, subclass::prelude::*};

use crate::{child_iter::IterChildren, components::Cmdline};

#[derive(gtk::CompositeTemplate, Default, glib::Properties)]
#[properties(wrapper_type = super::Omnibar)]
#[template(resource = "/com/github/vhakulinen/gnvim/omnibar.ui")]
pub struct Omnibar {
    #[property(
        name = "title",
        type = glib::GString,
        get = Self::get_title,
        set = Self::set_title,
    )]
    #[property(
        name = "title-height",
        type = i32,
        get = Self::get_title_height,
    )]
    #[property(
        name = "max-height",
        type = i32,
        get = Self::get_max_height,
        set = Self::set_max_height,
    )]
    #[template_child(id = "title")]
    pub title: TemplateChild<gtk::Label>,
    #[template_child(id = "cmdline-revealer")]
    pub cmdline_revealer: TemplateChild<gtk::Revealer>,
    #[template_child(id = "cmdline")]
    pub cmdline: TemplateChild<Cmdline>,
}

impl Omnibar {
    fn get_title(&self) -> glib::GString {
        self.title.label()
    }

    fn set_title(&self, title: glib::GString) {
        self.title.set_label(&title);
    }

    fn get_title_height(&self) -> i32 {
        let h = self.title.preferred_size().1.height();

        let style_ctx = self.obj().style_context();
        let border = style_ctx.border();
        let margin = style_ctx.margin();

        // Add our border and margin sizes to the height.
        h + border.top() as i32
            + border.bottom() as i32
            + margin.top() as i32
            + margin.bottom() as i32
    }

    fn get_max_height(&self) -> i32 {
        self.cmdline.max_height()
    }

    fn set_max_height(&self, h: i32) {
        let style_ctx = self.obj().style_context();
        let border = style_ctx.border();
        let margin = style_ctx.margin();

        // Remove our border and margin.
        let h = h
            - border.top() as i32
            - border.bottom() as i32
            - margin.top() as i32
            - margin.bottom() as i32;

        self.cmdline.set_max_height(h);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Omnibar {
    const NAME: &'static str = "Omnibar";
    type Type = super::Omnibar;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Cmdline::ensure_type();

        klass.set_css_name("omnibar");

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Omnibar {}

impl WidgetImpl for Omnibar {
    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        match orientation {
            gtk::Orientation::Horizontal => {
                let (mw, _nw, mb, nb) = self.parent_measure(orientation, for_size);

                // TODO(ville): Make the width smarter/configurable?
                (mw, 800, mb, nb)
            }
            gtk::Orientation::Vertical => self.obj().iter_children().fold(
                self.parent_measure(orientation, for_size),
                |acc, child| {
                    let (child_min, child_nat, _, _) = child.measure(orientation, for_size);

                    (acc.0.max(child_min), acc.1.max(child_nat), acc.2, acc.3)
                },
            ),
            _ => self.parent_measure(orientation, for_size),
        }
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        let obj = self.obj();
        obj.iter_children().for_each(|child| {
            child.allocate(width, height, -1, None);
        });

        obj.notify("title-height");
    }
}
