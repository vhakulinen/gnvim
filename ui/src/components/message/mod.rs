use gtk::glib;

glib::wrapper! {
    pub struct Message(ObjectSubclass<imp::Message>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Default for Message {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::{glib, prelude::*, subclass::prelude::*};

    use crate::child_iter::IterChildren;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::Message)]
    #[template(resource = "/com/github/vhakulinen/gnvim/message.ui")]
    pub struct Message {
        #[property(get, set)]
        pub content: RefCell<String>,
        #[property(get, set)]
        pub kind: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Message {
        const NAME: &'static str = "Message";
        type Type = super::Message;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("message");
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Message {
        fn dispose(&self) {
            self.obj()
                .iter_children()
                .for_each(|child| child.unparent());
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }

    impl WidgetImpl for Message {}
}
