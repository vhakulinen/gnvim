use gtk::glib;
use nvim::types::{uievents::MsgShow, MsgHistoryShowContent, MsgHistoryShowEntry, MsgShowContent};

use crate::colors::{Colors, Highlight};

use super::Kinds;

glib::wrapper! {
    pub struct MessageObject(ObjectSubclass<imp::MessageObject>);
}

impl MessageObject {
    pub fn new(event: MsgShow, colors: &Colors, kinds: &Kinds) -> Self {
        let content: String = event
            .content
            .iter()
            .map(|c| c.chunk_markup(colors))
            .collect();

        let kind = event.kind_markup(colors, kinds);

        glib::Object::builder()
            .property("content-markup", content)
            .property("kind-markup", kind)
            .property("kind", event.kind)
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
            .property("content-markup", content)
            .property("kind-markup", kind)
            .property("kind", entry.kind)
            .build()
    }
}

impl Default for MessageObject {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::{prelude::*, subclass::prelude::*};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MessageObject)]
    pub struct MessageObject {
        #[property(get, set)]
        pub kind: RefCell<String>,
        #[property(get, set)]
        pub kind_markup: RefCell<String>,
        #[property(get, set)]
        pub content_markup: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageObject {
        const NAME: &'static str = "MessageObject";
        type Type = super::MessageObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for MessageObject {
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
}

pub trait Chunk {
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

pub trait Kind {
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
