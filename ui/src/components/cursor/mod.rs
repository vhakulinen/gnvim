use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

use crate::{colors::Colors, font::Font, SCALE};

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

    pub fn hide(&self, hide: bool) {
        self.imp().hide.replace(hide);
    }

    pub fn flush(&self, colors: &Colors, font: &Font) {
        let imp = self.imp();
        if imp.node.borrow().is_some() {
            return;
        }

        let hl_id = *imp.attr_id.borrow();
        let fg = colors.get_hl_fg(hl_id);
        let bg = colors.get_hl_bg(hl_id);
        // For hl id zero, we need to flip fg and bg.
        let (fg, bg) = if hl_id == 0 { (bg, fg) } else { (fg, bg) };

        let height = font.height();
        let ch = font.char_width();
        let pos = imp.pos.borrow();
        let x = pos.0 as f32 * ch / SCALE;
        let y = pos.1 as f32 * height / SCALE;
        let baseline = (pos.1 as f32 * height + font.baseline()) / SCALE;

        let snapshot = gtk::Snapshot::new();

        let width = if *imp.double_width.borrow() {
            ch * 2.0 / SCALE
        } else {
            ch / SCALE
        };
        let width = width * *imp.width_percentage.borrow();
        let rect = graphene::Rect::new(x, y, width, height / SCALE);

        // Clip the area where we're drawing. This avoids a issue when the cursor
        // is narrow, yet we're drawing our own _whole_ cell. Clipping clips
        // _our_ render node to our _width_ and thus' the underlying grid cell
        // will be visible instead.
        snapshot.push_clip(&rect);

        snapshot.append_node(gsk::ColorNode::new(&bg, &rect).upcast());

        let attrs = crate::render::create_hl_attrs(hl_id, colors, font);
        crate::render::render_text(
            &snapshot,
            &self.pango_context(),
            &imp.text.borrow(),
            &fg,
            &attrs,
            x,
            baseline,
        );

        snapshot.pop();

        let node = snapshot
            .to_node()
            .unwrap_or_else(|| gsk::ContainerNode::new(&[]).upcast());
        imp.node.replace(Some(node));

        self.queue_draw();
    }

    pub fn row(&self) -> i64 {
        return self.imp().pos.borrow().1;
    }

    pub fn col(&self) -> i64 {
        return self.imp().pos.borrow().0;
    }

    pub fn move_to(&self, cell: &Cell, col: i64, row: i64) {
        let imp = self.imp();
        imp.pos.replace((col, row));
        imp.text.replace(cell.text.clone());
        imp.double_width.replace(cell.double_width);

        // Clear the render node.
        imp.node.replace(None);
    }

    pub fn set_text(&self, text: String) {
        let imp = self.imp();
        imp.text.replace(text);
        imp.node.replace(None);
    }

    pub fn set_width_percentage(&self, p: f32) {
        self.imp().width_percentage.replace(p);
    }

    pub fn set_attr_id(&self, id: i64) {
        self.imp().attr_id.replace(id);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
