use gtk::{glib::subclass::InitializingObject, prelude::*, subclass::prelude::*};

use crate::{child_iter::IterChildren, components::Cmdline};

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/omnibar.ui")]
pub struct Omnibar {
    #[template_child(id = "title")]
    pub title: TemplateChild<gtk::Label>,
    #[template_child(id = "cmdline-revealer")]
    pub cmdline_revealer: TemplateChild<gtk::Revealer>,
    #[template_child(id = "cmdline")]
    pub cmdline: TemplateChild<Cmdline>,
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

impl ObjectImpl for Omnibar {
    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecString::builder("title")
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecInt::builder("title-height")
                    .flags(glib::ParamFlags::READABLE)
                    .build(),
                glib::ParamSpecInt::builder("max-height")
                    .flags(glib::ParamFlags::READWRITE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "title" => self.title.label().to_value(),
            "max-height" => self.cmdline.max_height().to_value(),
            "title-height" => {
                let h = self.title.preferred_size().1.height();

                let style_ctx = obj.style_context();
                let border = style_ctx.border();
                let margin = style_ctx.margin();

                // Add our border and margin sizes to the height.
                let h = h
                    + border.top() as i32
                    + border.bottom() as i32
                    + margin.top() as i32
                    + margin.bottom() as i32;

                h.to_value()
            }
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "title" => {
                self.title
                    .set_label(value.get().expect("label must be a string"));
            }
            "max-height" => {
                let h: i32 = value.get().expect("max-height must be a i32");

                let style_ctx = obj.style_context();
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
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Omnibar {
    fn measure(
        &self,
        widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        match orientation {
            gtk::Orientation::Horizontal => {
                let (mw, _nw, mb, nb) = self.parent_measure(widget, orientation, for_size);

                // TODO(ville): Make the width smarter/configurable?
                (mw, 800, mb, nb)
            }
            gtk::Orientation::Vertical => widget.iter_children().fold(
                self.parent_measure(widget, orientation, for_size),
                |acc, child| {
                    let (child_min, child_nat, _, _) = child.measure(orientation, for_size);

                    (acc.0.max(child_min), acc.1.max(child_nat), acc.2, acc.3)
                },
            ),
            _ => self.parent_measure(widget, orientation, for_size),
        }
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        widget.iter_children().for_each(|child| {
            child.allocate(width, height, -1, None);
        });

        widget.notify("title-height");
    }
}
