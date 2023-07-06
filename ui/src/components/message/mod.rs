use gtk::glib;
use nvim::types::{uievents::MsgShow, MsgHistoryShowContent, MsgHistoryShowEntry, MsgShowContent};

use crate::colors::{Colors, Highlight, HlAttr};

use super::messages::Kinds;

glib::wrapper! {
    pub struct Message(ObjectSubclass<imp::Message>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Message {
    pub fn new(event: MsgShow, colors: &Colors, kinds: &Kinds) -> Self {
        let content: String = event
            .content
            .iter()
            .map(|c| c.chunk_markup(colors))
            .collect();

        let kind = event.kind_markup(colors, kinds);

        glib::Object::builder()
            .property("content", content)
            .property("kind", kind)
            .build()
    }

    pub fn new_history(entry: MsgHistoryShowEntry, colors: &Colors, kinds: &Kinds) -> Self {
        let content: String = entry
            .content
            .iter()
            .map(|c| c.chunk_markup(colors))
            .collect();

        let kind = entry.kind_markup(colors, kinds);

        glib::Object::builder()
            .property("content", content)
            .property("kind", kind)
            .build()
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

trait Chunk {
    fn text_chunk<'a>(&'a self) -> &'a str;
    fn attr_id<'a>(&'a self) -> &'a i64;

    fn chunk_markup(&self, colors: &Colors) -> String {
        let hl = colors.get_hl(self.attr_id());
        hl.pango_markup(self.text_chunk())
    }
}

impl Chunk for &MsgShowContent {
    fn text_chunk<'a>(&'a self) -> &'a str {
        &self.text_chunk
    }

    fn attr_id<'a>(&'a self) -> &'a i64 {
        &self.attr_id
    }
}

impl Chunk for &MsgHistoryShowContent {
    fn text_chunk<'a>(&'a self) -> &'a str {
        &self.text_chunk
    }

    fn attr_id<'a>(&'a self) -> &'a i64 {
        &self.attr_id
    }
}

trait Kind {
    fn kind<'a>(&'a self) -> &'a str;

    fn kind_markup(&self, colors: &Colors, kinds: &Kinds) -> String {
        let kind = self.kind();
        kinds
            .get(kind)
            .map(|kind| {
                let hl = Highlight::new(colors, kind.hl_attr.as_ref());
                hl.pango_markup(&kind.label)
            })
            .unwrap_or_else(|| Highlight::new(colors, None).pango_markup(kind))
    }
}

impl Kind for MsgShow {
    fn kind<'a>(&'a self) -> &'a str {
        &self.kind
    }
}

impl Kind for MsgHistoryShowEntry {
    fn kind<'a>(&'a self) -> &'a str {
        &self.kind
    }
}
