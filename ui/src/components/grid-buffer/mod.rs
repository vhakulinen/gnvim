use std::cell::{Ref, RefMut};

use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

use crate::{colors::Colors, font::Font};

mod imp;
mod row;

use row::{Cell, Row};

glib::wrapper! {
    pub struct GridBuffer(ObjectSubclass<imp::GridBuffer>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl GridBuffer {
    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create GridBuffer")
    }

    pub fn get_rows(&self) -> Ref<'_, Vec<Row>> {
        self.imp().rows.borrow()
    }

    pub fn get_rows_mut(&self) -> RefMut<'_, Vec<Row>> {
        self.imp().rows.borrow_mut()
    }

    pub fn resize(&self, width: usize, height: usize) {
        let mut rows = self.imp().rows.borrow_mut();
        rows.resize_with(height, Default::default);

        for row in rows.iter_mut() {
            row.cells.resize(width, Cell::default())
        }
    }

    pub fn clear(&self) {
        for row in self.imp().rows.borrow_mut().iter_mut() {
            row.clear();
        }
    }

    pub fn flush(&self, colors: &Colors, font: &Font) {
        let imp = self.imp();

        let h = font.height();
        for (i, row) in imp.rows.borrow_mut().iter_mut().enumerate() {
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
}

impl Default for GridBuffer {
    fn default() -> Self {
        Self::new()
    }
}
