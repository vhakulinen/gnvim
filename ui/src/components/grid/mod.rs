use glib::Object;
use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

use nvim::types::uievents::{GridLine, GridResize};

use crate::{colors::Colors, font::Font};

mod buffer;
mod imp;

glib::wrapper! {
    pub struct Grid(ObjectSubclass<imp::Grid>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Grid {
    pub fn new(id: i64) -> Self {
        Object::new(&[("grid-id", &id)]).expect("Failed to create Grid")
    }

    pub fn put(&self, event: GridLine) {
        let imp = self.imp();

        let mut buffer = imp.buffer.borrow_mut();
        let row = buffer.get_row(event.row as usize).expect("invalid row");

        row.update(&event);
    }

    pub fn resize(&self, event: GridResize) {
        self.imp()
            .buffer
            .borrow_mut()
            .resize(event.width as usize, event.height as usize);
    }

    pub fn flush(&self, colors: &Colors, font: &Font) {
        let imp = self.imp();

        let h = font.height();
        for (i, row) in imp.buffer.borrow_mut().rows.iter_mut().enumerate() {
            row.generate_nodes(&self.pango_context(), colors, font, i as f32 * h, h);
        }

        let alloc = self.allocation();
        let mut nodes = imp.background_nodes.borrow_mut();

        nodes.clear();
        nodes.push(
            gsk::ColorNode::new(
                &colors.bg,
                &graphene::Rect::new(0.0, 0.0, alloc.width() as f32, alloc.height() as f32),
            )
            .upcast(),
        );

        self.queue_draw();
    }

    pub fn clear(&self) {
        self.imp().buffer.borrow_mut().clear();
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(0)
    }
}
