use std::cell::RefCell;

use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::{boxed::Buffer, nvim::Neovim};

glib::wrapper! {
    pub struct BufferListObject(ObjectSubclass<Private>);
}

impl BufferListObject {
    pub fn new(name: &str, buffer: Buffer) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("buffer", buffer)
            .build()
    }
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = BufferListObject)]
pub struct Private {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    buffer: RefCell<Buffer>,
}

#[glib::object_subclass]
impl ObjectSubclass for Private {
    const NAME: &'static str = "BufferListObject";
    type Type = BufferListObject;
    type ParentType = glib::Object;
}

#[glib::derived_properties]
impl ObjectImpl for Private {}
