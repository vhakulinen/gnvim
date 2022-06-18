use std::cell::{Cell, RefCell};

use gtk::{
    gio,
    glib::{self, clone, subclass::InitializingObject},
    prelude::*,
    subclass::prelude::*,
};

use crate::font::Font;

use super::Row;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/popupmenu.ui")]
pub struct Popupmenu {
    #[template_child(id = "scrolled-window")]
    pub scrolledwindow: TemplateChild<gtk::ScrolledWindow>,
    #[template_child(id = "list-view")]
    pub listview: TemplateChild<gtk::ListView>,

    pub max_height: Cell<i32>,
    pub max_width: Cell<i32>,

    pub selection_model: gtk::SingleSelection,
    pub store: RefCell<gio::ListStore>,
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
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // TODO(ville): Would it be better to use our own custom type here?
        self.store
            .replace(gio::ListStore::new(glib::BoxedAnyObject::static_type()));

        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(clone!(@weak obj => move |_, listitem| {
            let item = Row::new();
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

        self.selection_model.set_model(Some(&*self.store.borrow()));
        self.listview.set_model(Some(&self.selection_model));
        self.listview.set_factory(Some(&factory));
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecObject::new(
                "font",
                "font",
                "Font",
                Font::static_type(),
                glib::ParamFlags::READWRITE,
            )]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "font" => self.font.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be object Font"));
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Popupmenu {
    fn measure(
        &self,
        widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
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
            _ => self.parent_measure(widget, orientation, for_size),
        }
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        self.scrolledwindow.allocate(width, height, baseline, None);
    }
}
