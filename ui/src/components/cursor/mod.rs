use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

use crate::{colors::Colors, font::Font};

use super::grid_buffer::row::Cell;

mod imp;

glib::wrapper! {
    pub struct Cursor(ObjectSubclass<imp::Cursor>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Cursor {
    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Cursor")
    }

    pub fn flush(&self, colors: &Colors, font: &Font) {
        let imp = self.imp();
        if imp.node.borrow().is_some() {
            return;
        }

        // NOTE(ville): Inverted colors.
        let fg = colors.get_hl_bg(*imp.hl_id.borrow());
        let bg = colors.get_hl_fg(*imp.hl_id.borrow());

        let height = font.height();
        let ch = font.char_width();
        let pos = imp.pos.borrow();
        let x = pos.0 as f32 * ch;
        let y = pos.1 as f32 * height;

        let snapshot = gtk::Snapshot::new();

        let width = if *imp.double_width.borrow() {
            ch * 2.0
        } else {
            ch
        };
        snapshot.append_node(
            gsk::ColorNode::new(&bg, &graphene::Rect::new(x, y, width, height)).upcast(),
        );

        let attrs = crate::render::create_hl_attrs(*imp.hl_id.borrow(), colors, font);

        crate::render::render_text(
            &snapshot,
            &self.pango_context(),
            &imp.text.borrow(),
            &fg,
            &attrs,
            x,
            y + font.ascent(),
        );

        let node = snapshot
            .to_node()
            .unwrap_or(gsk::ContainerNode::new(&[]).upcast());
        imp.node.replace(Some(node));

        self.queue_draw();
    }

    pub fn move_to(&self, cell: &Cell, col: i64, row: i64) {
        let imp = self.imp();
        imp.pos.replace((col, row));
        imp.hl_id.replace(cell.hl_id);
        imp.text.replace(cell.text.clone());
        imp.double_width.replace(cell.double_width);

        // Clear the render node.
        imp.node.replace(None);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
