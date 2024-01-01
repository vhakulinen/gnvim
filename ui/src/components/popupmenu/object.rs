use glib::{prelude::*, subclass::prelude::*};
use gtk::glib;
use nvim::types::PopupmenuItem;

use crate::colors::{Colors, Highlight, HlGroup};

use super::{row::Row, Kinds};

glib::wrapper! {
    pub struct PopupmenuObject(ObjectSubclass<imp::PopupmenuObject>);
}

impl PopupmenuObject {
    pub fn new(item: &PopupmenuItem, colors: &Colors, kinds: &Kinds) -> Self {
        let kind = kinds.get(&item.kind);

        let hl = kind
            .and_then(|kind| {
                kind.hl_attr
                    .as_ref()
                    .map(|attr| Highlight::new(colors, Some(attr)))
            })
            .unwrap_or_else(|| colors.get_hl_group(&HlGroup::Pmenu));
        let hl_sel = kind
            .and_then(|kind| {
                kind.hl_attr_sel
                    .as_ref()
                    .map(|attr| Highlight::new(colors, Some(attr)))
            })
            .unwrap_or_else(|| colors.get_hl_group(&HlGroup::PmenuSel));
        let kind_label = kind.map(|kind| &kind.label).unwrap_or(&item.kind);

        glib::Object::builder()
            .property("word", &item.word)
            .property("kind", &item.kind)
            .property("kind-markup", hl.pango_markup(kind_label))
            .property("kind-unselected", hl.pango_markup(kind_label))
            .property("kind-selected", hl_sel.pango_markup(kind_label))
            .build()
    }

    /// Unbind every property bound in `bind_to`.
    pub fn unbind(&self) {
        for binding in self.imp().bindings.replace(vec![]) {
            binding.unbind();
        }
    }

    /// Bind properties to given row.
    pub fn bind_to(&self, row: &Row) {
        self.unbind();

        self.imp().bindings.replace(vec![
            self.bind_property("kind-markup", row, "kind-markup")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build(),
            self.bind_property("word", row, "word-label")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build(),
        ]);

        row.set_word_label(self.word());
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use glib::{prelude::*, subclass::prelude::*};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::PopupmenuObject)]
    pub struct PopupmenuObject {
        /// Original "word" value.
        #[property(get, set)]
        pub word: RefCell<String>,
        /// Original "kind" value.
        #[property(get, set)]
        pub kind: RefCell<String>,

        /// Currently displayed kind markup.
        #[property(get, set)]
        pub kind_markup: RefCell<String>,

        /// Kind markup for when the item is selected.
        #[property(get, set)]
        pub kind_selected: RefCell<String>,
        /// Kind markup for when the item is not selected.
        #[property(get, set)]
        pub kind_unselected: RefCell<String>,

        /// If the item is selected.
        ///
        /// Toggles the `kind_markup` between `kind_selected` & `kind_unselected`.
        #[property(get, set = Self::set_selected)]
        pub selected: Cell<bool>,

        /// Book keeping of our bindings.
        pub bindings: RefCell<Vec<glib::Binding>>,
    }

    impl PopupmenuObject {
        fn set_selected(&self, selected: bool) {
            self.selected.replace(selected);

            let obj = self.obj();
            if selected {
                obj.set_kind_markup(&**self.kind_selected.borrow());
            } else {
                obj.set_kind_markup(&**self.kind_unselected.borrow());
            }
        }
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
