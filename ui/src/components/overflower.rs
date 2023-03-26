glib::wrapper! {
    /// Overflower is a simple widget that helps making the omnibar able to
    /// overflow the GtkHeaderBar widget while retaining its "original"
    /// (e.g. the title label's) height.
    ///
    /// Note that Overflower can only have one child.
    pub struct Overflower(ObjectSubclass<imp::Overflower>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

mod imp {
    use std::cell::Cell;

    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(Default)]
    pub struct Overflower {
        pub height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Overflower {
        const NAME: &'static str = "Overflower";
        type Type = super::Overflower;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("overflower");
        }
    }

    impl ObjectImpl for Overflower {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecInt::builder("height")
                    .flags(glib::ParamFlags::READWRITE)
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "height" => self.height.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "height" => {
                    self.height.set(value.get().expect("height must be a i32"));
                    self.obj().queue_resize();
                }
                _ => unimplemented!(),
            };
        }
    }

    impl WidgetImpl for Overflower {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let m = self
                .obj()
                .first_child()
                .map(|child| child.measure(orientation, for_size))
                .unwrap_or_else(|| self.parent_measure(orientation, for_size));

            match orientation {
                gtk::Orientation::Vertical => {
                    let h = self.height.get();
                    (h, h, -1, -1)
                }
                _ => m,
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            if let Some(child) = self.obj().first_child() {
                let (_, req) = child.preferred_size();

                // NOTE(ville): Using the child's preferred natural height,
                // because that is what we want to "overflow" while the width
                // is something we want to limit when needed.
                child.allocate(width, req.height(), baseline, None);
            }
        }
    }
}
