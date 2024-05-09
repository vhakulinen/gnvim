use std::cell::RefCell;

use gtk::{gio, glib, glib::subclass::*, prelude::*, subclass::prelude::*};

glib::wrapper! {
    pub struct NavFile(ObjectSubclass<Private>)
        @extends gtk::Widget;
}

#[derive(Default, gtk::CompositeTemplate, glib::Properties)]
#[properties(wrapper_type = NavFile)]
#[template(resource = "/com/github/vhakulinen/gnvim/navfile.ui")]
pub struct Private {
    #[template_child(id = "expander")]
    expander: TemplateChild<gtk::TreeExpander>,
    #[property(get, set, nullable)]
    gicon: RefCell<Option<gio::Icon>>,
    #[property(get, set, nullable)]
    label: RefCell<Option<String>>,
    #[property(get, set, nullable)]
    list_row: RefCell<Option<gtk::TreeListRow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Private {
    const NAME: &'static str = "NavFile";
    type Type = NavFile;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Private {
    fn constructed(&self) {
        self.parent_constructed();

        let controller = gtk::GestureClick::new();

        let obj = self.obj();
        controller.connect_released(glib::clone!(@weak obj => move |_, _, _, _| {
            // We want to just activate the action, ignore the result.
            let _ = obj.imp().expander.activate_action("listitem.toggle-expand", None);
        }));

        obj.add_controller(controller);
    }

    fn dispose(&self) {
        self.dispose_template();
    }
}

impl WidgetImpl for Private {}
