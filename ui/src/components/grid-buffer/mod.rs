use std::cell::{Ref, RefMut};

use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};
use nvim::types::uievents::GridScroll;

use crate::{colors::Colors, font::Font};

mod imp;
pub mod row;

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

        let ctx = self.pango_context();

        let h = font.height();
        for (i, row) in imp.rows.borrow_mut().iter_mut().enumerate() {
            row.generate_nodes(&ctx, colors, font, i as f32 * h, h);
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

    fn scroll_region(event: &GridScroll) -> (Box<dyn Iterator<Item = i64>>, i64) {
        if event.rows > 0 {
            let top = event.top + event.rows;
            let bot = event.bot;

            (Box::new(top..bot), -event.rows)
        } else {
            let top = event.top;
            let bot = event.bot + event.rows;

            (Box::new((top..bot).rev()), event.rows.abs())
        }
    }

    pub fn scroll(&self, event: GridScroll) {
        assert_eq!(
            event.cols, 0,
            "at the moment of writing, grid_scroll event documents cols to be always zero"
        );

        let left = event.left as usize;
        let right = event.right as usize;
        let (iter, count) = GridBuffer::scroll_region(&event);

        let mut rows = self.imp().rows.borrow_mut();
        for i in iter {
            let dst = (i + count) as usize;
            let i = i as usize;

            assert!(i != dst, "cant get two mutable references to same element");
            let (src, dst) = unsafe {
                let src: &mut Row = &mut *(rows.get_mut(i).expect("bad scroll region") as *mut _);
                let dst: &mut Row = &mut *(rows.get_mut(dst).expect("bad scroll region") as *mut _);
                (src, dst)
            };

            // Clear render nodes, so they get redrawn.
            src.cells[left..right]
                .iter_mut()
                .for_each(Cell::clear_nodes);
            dst.cells[left..right]
                .iter_mut()
                .for_each(Cell::clear_nodes);

            dst.cells[left..right].swap_with_slice(&mut src.cells[left..right]);
        }
    }
}

impl Default for GridBuffer {
    fn default() -> Self {
        Self::new()
    }
}
