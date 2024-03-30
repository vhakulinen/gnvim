use std::cell::RefCell;

use glib::{clone, subclass::InitializingObject};
use gtk::{glib, prelude::*, subclass::prelude::*};
use nvim::NeovimApi;

use crate::{boxed::Tabpage, nvim::Neovim, spawn_local};

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/com/github/vhakulinen/gnvim/tab.ui")]
pub struct Tab {
    #[template_child(id = "content")]
    pub content: TemplateChild<gtk::Label>,

    pub nvim: RefCell<Neovim>,
    // NOTE(ville): Tabpage doesn't have Default impl, hence the wrapped option.
    pub tabpage: RefCell<Option<Tabpage>>,
    pub gesture_click: gtk::GestureClick,
}

#[glib::object_subclass]
impl ObjectSubclass for Tab {
    const NAME: &'static str = "Tab";
    type Type = super::Tab;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
        klass.set_css_name("tab");
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Tab {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.add_controller(self.gesture_click.clone());

        self.gesture_click
            .connect_pressed(clone!(@weak obj => move |_, _, _, _| {
                spawn_local!(async move {
                    let nvim = obj.nvim();
                    let page = obj
                        .imp()
                        .tabpage
                        .borrow()
                        .clone()
                        .expect("tabpage not set");
                    nvim.nvim_set_current_tabpage(&page)
                        .await
                        .expect("nvim_set_current_tabpage failed");
                });
            }));
    }

    fn dispose(&self) {
        self.content.unparent();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecString::builder("label")
                    .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT)
                    .build(),
                glib::ParamSpecObject::builder::<Neovim>("nvim")
                    .flags(glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT)
                    .build(),
                glib::ParamSpecBoxed::builder::<Tabpage>("tabpage")
                    .flags(glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "label" => self.content.label().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "nvim" => {
                self.nvim
                    .replace(value.get().expect("nvim must to be an Neovim object"));
            }
            "tabpage" => {
                self.tabpage
                    .replace(Some(value.get().expect("tabpage must be a Tabpage object")));
            }
            "label" => {
                self.content
                    .set_label(value.get().expect("label must be a string"));
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Tab {}
