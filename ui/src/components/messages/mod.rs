use std::{cell::Ref, collections::HashMap};

use gtk::{glib, prelude::*, subclass::prelude::*};
use nvim::types::uievents::{MsgHistoryShow, MsgShow};

use crate::{api::MessageKind, colors::Colors};

use super::Message;

mod object;

pub use object::MessageObject;

glib::wrapper! {
    pub struct Messages(ObjectSubclass<imp::Messages>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Messages {
    pub fn kinds(&self) -> Ref<Kinds> {
        self.imp().kinds.borrow()
    }

    pub fn handle_message_clear(&self) {
        self.imp().store.remove_all();
    }

    pub fn handle_message_history_clear(&self) {
        self.handle_message_clear();
    }

    pub fn handle_message_show(&self, events: Vec<MsgShow>, colors: &Colors) {
        let imp = self.imp();
        let kinds = imp.kinds.borrow();
        let n = events.len();
        let objs = events
            .into_iter()
            .fold(Vec::with_capacity(n), |mut acc, item| {
                if item.replace_last {
                    // If our accumulator doesn't have any values in it, we'll
                    // have to remove actual previous item.
                    if !acc.is_empty() {
                        acc.pop();
                    } else if imp.store.n_items() > 0 {
                        imp.store.remove(imp.store.n_items() - 1);
                    }
                }

                acc.push(MessageObject::new(item, colors, &kinds));

                acc
            });
        imp.store.splice(imp.store.n_items(), 0, &objs);
    }

    pub fn handle_message_history_show(&self, event: MsgHistoryShow, colors: &Colors) {
        let imp = self.imp();
        let kinds = imp.kinds.borrow();
        let objs = event
            .entries
            .into_iter()
            .map(|entry| MessageObject::new_history(entry, colors, &kinds))
            .collect::<Vec<_>>();

        imp.store.splice(0, 0, &objs);
    }

    pub fn set_kinds(&self, kinds: Kinds) {
        self.imp().kinds.replace(kinds);
    }

    pub fn scroll_to_bottom(&self) {
        let imp = self.imp();
        let n = imp.store.n_items();
        if n > 0 {
            imp.listview
                .activate_action("list.scroll-to-item", Some(&(n - 1).to_variant()))
                .expect("failed to activate list.scroll-to-item action");
        }
    }
}

pub type Kinds = HashMap<String, MessageKind>;

mod imp {
    use std::cell::RefCell;

    use glib::{clone, subclass::InitializingObject};
    use gtk::{gio, glib, prelude::*, subclass::prelude::*};

    use crate::{child_iter::IterChildren, components::Message};

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/com/github/vhakulinen/gnvim/messages.ui")]
    pub struct Messages {
        // NOTE(ville): HashMap is not supported by the glib::Properties derive.
        pub kinds: RefCell<super::Kinds>,

        #[template_child(id = "messages-list")]
        pub listview: TemplateChild<gtk::ListView>,

        pub store: gio::ListStore,
    }

    impl Default for Messages {
        fn default() -> Self {
            Self {
                kinds: Default::default(),
                listview: Default::default(),
                store: gio::ListStore::new::<super::MessageObject>(),
            }
        }
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
        fn constructed(&self) {
            let factory = gtk::SignalListItemFactory::new();

            factory.connect_setup(clone!(@weak self as this => move |_, listitem| {
                 let child = Message::default();

                 listitem.set_child(Some(&child));
            }));

            factory.connect_bind(|_, listitem| {
                let msg_object = listitem
                    .item()
                    .and_downcast::<super::MessageObject>()
                    .expect("item must be MessageObject");

                let msg_widget = listitem
                    .child()
                    .and_downcast::<super::Message>()
                    .expect("child must be Message");

                msg_widget.set_kind(msg_object.kind_markup());
                msg_widget.set_content(msg_object.content_markup());
                msg_widget.set_css_classes(&[&format!("kind-{}", msg_object.kind())]);
            });

            let model = gtk::NoSelection::new(Some(self.store.clone()));
            self.listview.set_model(Some(&model));
            self.listview.set_factory(Some(&factory));

            self.store
                .connect_items_changed(clone!(@weak self as this => move |store, _, _, _| {
                    this.obj().set_visible(store.n_items() > 0);
                }));
        }

        fn dispose(&self) {
            self.obj()
                .iter_children()
                .for_each(|child| child.unparent())
        }
    }

    impl WidgetImpl for Messages {}
}
