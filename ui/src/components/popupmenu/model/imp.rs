use std::cell::{Cell, RefCell};

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

#[derive(Default)]
pub struct Model {
    pub items: RefCell<Vec<glib::BoxedAnyObject>>,
    pub to_add: RefCell<Vec<glib::BoxedAnyObject>>,
    pub lazy: RefCell<Option<glib::SourceId>>,

    /// Item to select from the lazy loading operations.
    pub selected_item: Cell<Option<u32>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Model {
    const NAME: &'static str = "PopupmenuModel";
    type Type = super::Model;
    type Interfaces = (gio::ListModel, gtk::SelectionModel);
}

impl ObjectImpl for Model {}

impl ListModelImpl for Model {
    fn item_type(&self, _: &Self::Type) -> glib::Type {
        glib::BoxedAnyObject::static_type()
    }

    fn n_items(&self, _: &Self::Type) -> u32 {
        self.items.borrow().len() as u32
    }

    fn item(&self, _: &Self::Type, position: u32) -> Option<glib::Object> {
        self.items
            .borrow()
            .get(position as usize)
            .map(|b| b.clone().upcast::<glib::Object>())
    }
}

impl SelectionModelImpl for Model {
    fn select_item(&self, model: &Self::Type, position: u32, _unselect_rest: bool) -> bool {
        let old = self.selected_item.replace(Some(position));
        // NOTE(ville): We need to notify selection-changed on our old item too.
        model.do_selection_changed(old);
        model.do_selection_changed(Some(position));

        true
    }

    fn unselect_all(&self, model: &Self::Type) -> bool {
        let prev = self.selected_item.replace(None);
        model.do_selection_changed(prev);
        true
    }

    fn selection_in_range(
        &self,
        _model: &Self::Type,
        _position: u32,
        _n_items: u32,
    ) -> gtk::Bitset {
        unimplemented!("selection_in_range not supported");
    }

    fn is_selected(&self, _model: &Self::Type, position: u32) -> bool {
        self.selected_item
            .get()
            .map(|i| i == position)
            .unwrap_or(false)
    }

    fn select_all(&self, _model: &Self::Type) -> bool {
        false
    }

    fn select_range(
        &self,
        _model: &Self::Type,
        _position: u32,
        _n_items: u32,
        _unselect_rest: bool,
    ) -> bool {
        false
    }

    fn set_selection(
        &self,
        _model: &Self::Type,
        _selected: &gtk::Bitset,
        _mask: &gtk::Bitset,
    ) -> bool {
        false
    }

    fn unselect_item(&self, _model: &Self::Type, _position: u32) -> bool {
        false
    }

    fn unselect_range(&self, _model: &Self::Type, _position: u32, _n_items: u32) -> bool {
        false
    }
}
