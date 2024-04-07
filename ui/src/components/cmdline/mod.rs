use gtk::{prelude::*, subclass::prelude::*};

use nvim::types::uievents::{
    CmdlineBlockAppend, CmdlineBlockShow, CmdlinePos, CmdlineShow, CmdlineSpecialChar,
    PopupmenuSelect, PopupmenuShow,
};

use crate::colors::Colors;

use super::popupmenu;

mod imp;

glib::wrapper! {
    pub struct Cmdline(ObjectSubclass<imp::Cmdline>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Cmdline {
    pub fn show(&self, event: CmdlineShow, colors: &Colors) {
        let imp = self.imp();

        let buf = imp.main.buffer();
        buf.set_text("");
        let mut iter = buf.start_iter();

        let prompt = format!(
            "{}{}{}",
            event.firstc,
            " ".repeat(event.indent as usize),
            event.prompt,
        );
        buf.insert_markup(&mut iter, &prompt);
        imp.prompt_len.set(prompt.chars().count() as i32);

        // TODO(ville): Figure out how to render "special" chars.
        // TODO(ville): Make things single line only.

        let content = event
            .content
            .iter()
            .map(|item| colors.get_hl(&item.hl_id).pango_markup(&item.text))
            .collect::<String>();
        buf.insert_markup(&mut iter, &content);

        self.set_cursor_pos(event.pos as i32);
    }

    pub fn pos(&self, event: CmdlinePos) {
        self.set_cursor_pos(event.pos as i32);
    }

    pub fn special_char(&self, event: CmdlineSpecialChar) {
        let imp = self.imp();
        let buf = imp.main.buffer();
        let mark = buf.mark("cursor").expect("cursor mark not set");
        let mut iter = buf.iter_at_mark(&mark);

        buf.insert(&mut iter, &event.c);
    }

    fn set_cursor_pos(&self, pos: i32) {
        let imp = self.imp();
        let buf = imp.main.buffer();

        let iter = imp.main.buffer().iter_at_offset(imp.prompt_len.get() + pos);

        let mark = buf.mark("cursor").unwrap_or_else(|| {
            let mark = buf.create_mark(Some("cursor"), &iter, false);
            mark.set_visible(true);
            mark
        });

        buf.move_mark(&mark, &iter);
        imp.main.scroll_to_mark(&mark, 0.0, false, 0.0, 0.0);
    }

    pub fn set_linespace(&self, space: f32) {
        let imp = self.imp();
        let space = space / 2.0;
        let above = space.ceil() as i32;
        let below = space.floor() as i32;

        imp.main.set_pixels_above_lines(above);
        imp.main.set_pixels_below_lines(below);

        imp.block.set_pixels_above_lines(above);
        imp.block.set_pixels_below_lines(below);
    }

    pub fn block_show(&self, event: CmdlineBlockShow, colors: &Colors) {
        let imp = self.imp();

        let buf = imp.block.buffer();
        buf.set_text("");
        let mut iter = buf.start_iter();

        event.lines.iter().enumerate().for_each(|(i, line)| {
            let line = line
                .iter()
                .map(|item| colors.get_hl(&item.hl_id).pango_markup(&item.text))
                .collect::<String>();

            if i > 0 {
                buf.insert(&mut iter, "\n");
            }

            buf.insert_markup(&mut iter, &line);
        });

        imp.block.show();
    }

    pub fn block_append(&self, event: CmdlineBlockAppend, colors: &Colors) {
        let imp = self.imp();

        let content = event
            .lines
            .iter()
            .map(|item| colors.get_hl(&item.hl_id).pango_markup(&item.text))
            .collect::<String>();

        let buf = imp.block.buffer();
        let mut iter = buf.end_iter();

        buf.insert(&mut iter, "\n");
        buf.insert_markup(&mut iter, &content);
    }

    pub fn block_hide(&self) {
        self.imp().block.hide();
    }

    pub fn popupmenu_show(
        &self,
        event: PopupmenuShow,
        colors: &Colors,
        kinds: &mut popupmenu::Kinds,
    ) {
        let imp = self.imp();

        imp.popupmenu.set_items(event.items, colors, kinds);
        imp.popupmenu.set_visible(true);
    }

    pub fn poupmenu_visible(&self) -> bool {
        self.imp().popupmenu.is_visible()
    }

    pub fn popupmenu_select(&self, event: PopupmenuSelect) {
        self.imp().popupmenu.select(event.selected);
    }

    pub fn popupmenu_hide(&self) {
        self.imp().popupmenu.set_visible(false);
    }
}
