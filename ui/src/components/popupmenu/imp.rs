use std::cell::{Cell, RefCell};

use gtk::{
    glib::{self, clone, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
};

use crate::{
    components::{popupmenu::Kind, MaxSizeLayoutManager},
    font::Font,
};

use super::{PopupmenuObject, Row};

#[derive(gtk::CompositeTemplate, glib::Properties, Default)]
#[properties(wrapper_type = super::Popupmenu)]
#[template(resource = "/com/github/vhakulinen/gnvim/popupmenu.ui")]
pub struct Popupmenu {
    #[template_child(id = "scrolled-window")]
    pub scrolledwindow: TemplateChild<gtk::ScrolledWindow>,
    #[template_child(id = "list-view")]
    pub listview: TemplateChild<gtk::ListView>,

    #[property(get, set)]
    pub max_height: Cell<i32>,
    #[property(get, set)]
    pub max_width: Cell<i32>,

    pub store: super::Model,
    #[property(get, set)]
    pub font: RefCell<Font>,
}

#[glib::object_subclass]
impl ObjectSubclass for Popupmenu {
    const NAME: &'static str = "Popupmenu";
    type Type = super::Popupmenu;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Row::ensure_type();
        MaxSizeLayoutManager::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Popupmenu {
    fn constructed(&self) {
        self.parent_constructed();

        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(clone!(@weak self as imp => move |_, listitem| {
            let listitem = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("invalid listitem type");
            let item = Row::default();
            let obj = imp.obj();
            obj.bind_property("font", &item, "font")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            // When the listitem is selected, adjust the `kind` value.
            listitem.property_expression("selected")
                .chain_closure::<String>(glib::closure!(|listitem: gtk::ListItem, selected: bool| {
                    let f = listitem.property_expression("item")
                        .chain_property::<PopupmenuObject>("kind")
                        .chain_property::<Kind>(if selected {
                            "selected"
                        } else {
                            "normal"
                        })
                    .evaluate_as::<String, gtk::ListItem>(Some(&listitem));

                    f.unwrap_or("".to_owned())
                }))
                .bind(&item, "kind", Some(listitem));

            // Default kind value.
            listitem.property_expression("item")
                .chain_property::<PopupmenuObject>("kind")
                .chain_property::<Kind>("normal")
                .bind(&item, "kind", Some(listitem));

            // Word value.
            listitem.property_expression("item")
                .chain_property::<PopupmenuObject>("word")
                .bind(&item, "word", Some(listitem));

            listitem.set_child(Some(&item));
        }));

        self.listview.set_model(Some(&self.store));
        self.listview.set_factory(Some(&factory));
    }

    fn dispose(&self) {
        self.dispose_template();
    }
}

impl WidgetImpl for Popupmenu {}
