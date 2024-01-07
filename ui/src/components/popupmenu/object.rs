use glib;
use nvim::types::PopupmenuItem;

use crate::colors::Colors;

use super::Kinds;

glib::wrapper! {
    pub struct PopupmenuObject(ObjectSubclass<imp::PopupmenuObject>);
}

impl PopupmenuObject {
    pub fn new(item: &PopupmenuItem, colors: &Colors, kinds: &mut Kinds) -> Self {
        let kind = kinds.get(&item.kind, colors);

        glib::Object::builder()
            .property("word", &item.word)
            .property("kind", kind)
            .build()
    }
}

impl Default for PopupmenuObject {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::{prelude::*, subclass::prelude::*};

    use crate::components::popupmenu::Kind;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::PopupmenuObject)]
    pub struct PopupmenuObject {
        #[property(get, set)]
        kind: RefCell<Kind>,
        #[property(get, set)]
        word: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PopupmenuObject {
        const NAME: &'static str = "PopupmenuObject";
        type Type = super::PopupmenuObject;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PopupmenuObject {}
}
