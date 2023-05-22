use std::cell::{Cell, RefCell};

use gtk::{
    glib::{self, clone, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
};

use crate::font::Font;

use super::Row;

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

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Popupmenu {
    fn constructed(&self) {
        self.parent_constructed();

        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(clone!(@weak self as imp => move |_, listitem| {
            let item = Row::default();
            let obj = imp.obj();
            obj.bind_property("font", &item, "font")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            listitem.set_child(Some(&item));
        }));

        factory.connect_bind(|_, listitem| {
            let item = listitem
                .item()
                .expect("failed to get item from listitem")
                .downcast::<glib::BoxedAnyObject>()
                .expect("unexpected item type");

            let row = listitem
                .child()
                .expect("failed to get child from listitem")
                .downcast::<Row>()
                .expect("unexpected child type");

            row.set_item(&item.borrow());
        });

        self.listview.set_model(Some(&self.store));
        self.listview.set_factory(Some(&factory));
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

impl WidgetImpl for Popupmenu {
    fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        // Use the listview's measurement for our size.
        let (_, n, _, _) = self.listview.measure(orientation, for_size);

        match orientation {
            gtk::Orientation::Horizontal => {
                let w = n.min(self.max_width.get());
                (w, w, -1, -1)
            }
            gtk::Orientation::Vertical => {
                let h = n.min(self.max_height.get());
                (h, h, -1, -1)
            }
            _ => self.parent_measure(orientation, for_size),
        }
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        self.scrolledwindow.allocate(width, height, baseline, None);
    }
}
