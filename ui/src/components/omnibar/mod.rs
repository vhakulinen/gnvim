use gtk::subclass::prelude::*;
use nvim::types::uievents::{
    CmdlineBlockAppend, CmdlineBlockShow, CmdlineHide, CmdlinePos, CmdlineShow, CmdlineSpecialChar,
    PopupmenuSelect, PopupmenuShow,
};

use crate::colors::Colors;

use super::popupmenu;

mod imp;

glib::wrapper! {
    pub struct Omnibar(ObjectSubclass<imp::Omnibar>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Omnibar {
    pub fn handle_cmdline_show(&self, event: CmdlineShow, colors: &Colors) {
        let imp = self.imp();

        imp.cmdline.show(event, colors);

        imp.cmdline_revealer.set_reveal_child(true);
    }

    pub fn handle_cmdline_hide(&self, _event: CmdlineHide) {
        let imp = self.imp();

        imp.cmdline_revealer.set_reveal_child(false);
    }

    pub fn handle_cmdline_pos(&self, event: CmdlinePos) {
        let imp = self.imp();

        imp.cmdline.pos(event);
    }

    pub fn handle_cmdline_special_char(&self, event: CmdlineSpecialChar) {
        let imp = self.imp();

        imp.cmdline.special_char(event);
    }

    pub fn handle_cmdline_block_show(&self, event: CmdlineBlockShow, colors: &Colors) {
        self.imp().cmdline.block_show(event, colors);
    }

    pub fn handle_cmdline_block_hide(&self) {
        self.imp().cmdline.block_hide();
    }

    pub fn handle_cmdline_block_append(&self, event: CmdlineBlockAppend, colors: &Colors) {
        self.imp().cmdline.block_append(event, colors);
    }

    pub fn set_cmdline_linespace(&self, space: f32) {
        self.imp().cmdline.set_linespace(space);
    }

    pub fn handle_popupmenu_show(
        &self,
        event: PopupmenuShow,
        colors: &Colors,
        kinds: &mut popupmenu::Kinds,
    ) {
        self.imp().cmdline.popupmenu_show(event, colors, kinds);
    }

    pub fn cmdline_popupmenu_visible(&self) -> bool {
        self.imp().cmdline.poupmenu_visible()
    }

    pub fn handle_popupmenu_select(&self, event: PopupmenuSelect) {
        self.imp().cmdline.popupmenu_select(event);
    }

    pub fn handle_popupmenu_hide(&self) {
        self.imp().cmdline.popupmenu_hide();
    }
}
