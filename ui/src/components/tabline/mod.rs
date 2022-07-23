use gtk::{glib, prelude::*, subclass::prelude::*};
use nvim::types::{uievents::TablineUpdate, ShowTabline};

use crate::child_iter::IterChildren;

mod imp;

glib::wrapper! {
    pub struct Tabline(ObjectSubclass<imp::Tabline>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Tabline {
    pub fn handle_tabline_update(&self, event: TablineUpdate) {
        let imp = self.imp();

        imp.content
            .iter_children()
            .for_each(|child| child.unparent());

        for tab in event.tabs.iter() {
            let label = gtk::Label::new(Some(&tab.name));

            if tab.tab == event.current {
                label.add_css_class("selected");
            }

            imp.content.append(&label);
        }
    }

    pub fn flush(&self) {
        let imp = self.imp();
        let visible = match **imp.show.borrow() {
            ShowTabline::Never => false,
            ShowTabline::Always => true,
            ShowTabline::MoreThanOne => imp.content.iter_children().count() > 1,
        };

        self.set_visible(visible);
    }
}
