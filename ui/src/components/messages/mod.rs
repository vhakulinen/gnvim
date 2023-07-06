use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use gtk::{glib, prelude::*, subclass::prelude::*};
use nvim::types::uievents::{MsgHistoryShow, MsgShow};

use crate::{api::MessageKind, child_iter::IterChildren, colors::Colors};

use super::Message;

glib::wrapper! {
    pub struct Messages(ObjectSubclass<imp::Messages>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Messages {
    pub fn handle_message_clear(&self) {
        self.iter_children().for_each(|child| child.unparent());
    }

    pub fn handle_message_history_clear(&self) {
        self.handle_message_clear();
    }

    pub fn handle_message_show(&self, event: MsgShow, colors: &Colors) {
        if event.replace_last {
            if let Some(child) = self.last_child() {
                child.unparent();
            }
        }

        let child = Message::new(event, colors, &self.imp().kinds.borrow());
        child.set_parent(&*self);
    }

    pub fn handle_message_history_show(&self, event: MsgHistoryShow, colors: &Colors) {
        let kinds = self.imp().kinds.borrow();
        event.entries.into_iter().for_each(|entry| {
            let child = Message::new_history(entry, colors, &kinds);
            child.set_parent(&*self);
        });
    }

    pub fn set_kinds(&self, kinds: HashMap<String, MessageKind>) {
        self.imp().kinds.replace(Kinds(kinds));
    }
}

#[derive(Default)]
pub struct Kinds(HashMap<String, MessageKind>);

impl Deref for Kinds {
    type Target = HashMap<String, MessageKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Kinds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::{glib, prelude::*, subclass::prelude::*};

    use crate::child_iter::IterChildren;

    use super::Kinds;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/vhakulinen/gnvim/messages.ui")]
    pub struct Messages {
        // NOTE(ville): HashMap is not supported by the glib::Properties derive.
        pub kinds: RefCell<Kinds>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Messages {
        const NAME: &'static str = "Messages";
        type Type = super::Messages;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("messages");
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Messages {
        fn dispose(&self) {
            self.obj()
                .iter_children()
                .for_each(|child| child.unparent())
        }
    }

    impl WidgetImpl for Messages {}
}
