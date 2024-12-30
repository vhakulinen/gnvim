use std::cell::{Ref, RefMut};

use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};
use nvim::types::uievents::{GridLine, GridScroll};

use crate::colors::Colors;

mod imp;
pub mod row;

pub use imp::ViewportMargins;

use row::{Cell, RenderNodeIter, Row, ToRenderNode};

glib::wrapper! {
    pub struct GridBuffer(ObjectSubclass<imp::GridBuffer>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl GridBuffer {
    fn new() -> Self {
        glib::Object::new()
    }

    /// Returns the size of the grid, in rows and columns.
    ///
    /// Returns (cols, rows).
    pub fn grid_size(&self) -> (usize, usize) {
        let rows = self.get_rows();
        let cols = rows.first().expect("bad grid size").cells.len();
        let rows = rows.len();

        (cols, rows)
    }

    pub fn get_rows(&self) -> Ref<'_, Vec<Row>> {
        self.imp().rows.borrow()
    }

    fn get_rows_mut(&self) -> RefMut<'_, Vec<Row>> {
        self.imp().rows.borrow_mut()
    }

    pub fn update_row(&self, event: &GridLine) {
        let mut rows = self.get_rows_mut();
        let row = rows.get_mut(event.row as usize).expect("invalid row");

        let n = row.update(event);

        // Invalidate the margin nodes if needed.
        let left = event.col_start;
        let right = left + n as i64;
        let vp = self.viewport_margins();
        let mut nodes = self.imp().nodes.borrow_mut();
        if event.row < vp.top {
            nodes.margin_top.take();
        }
        if event.row > vp.bottom {
            nodes.margin_top.take();
        }
        if left < vp.left || right > vp.right {
            nodes.margin_sides.take();
        }

        self.set_dirty(true);
    }

    pub fn resize(&self, width: usize, height: usize) {
        self.set_size(imp::Size { width, height })
    }

    pub fn clear(&self) {
        for row in self.imp().rows.borrow_mut().iter_mut() {
            row.clear();
        }

        self.set_dirty(true);
    }

    pub fn flush(&self, colors: &Colors) {
        if !self.dirty() {
            return;
        }

        let imp = self.imp();

        let ctx = self.pango_context();

        let mut nodes = imp.nodes.borrow_mut();
        let font = imp.font.borrow();
        nodes.foreground = gsk::MaskNode::new(
            gsk::ContainerNode::new(
                &imp.rows
                    .borrow_mut()
                    .iter_mut()
                    .enumerate()
                    .map(|(i, row)| row.to_render_node(&ctx, colors, &font, i))
                    .collect::<Vec<_>>(),
            )
            .upcast(),
            imp.create_margins_mask(),
            gsk::MaskMode::Alpha,
        )
        .upcast();

        if nodes.margin_top.is_none()
            || nodes.margin_bottom.is_none()
            || nodes.margin_sides.is_none()
        {
            nodes.margins = gsk::MaskNode::new(
                self.margin_nodes(&mut nodes),
                imp.create_margins_mask(),
                gsk::MaskMode::InvertedAlpha,
            )
            .upcast();
        }

        let (alloc, _) = self.preferred_size();
        nodes.background = gsk::ColorNode::new(
            &colors.bg,
            &graphene::Rect::new(0.0, 0.0, alloc.width() as f32, alloc.height() as f32),
        )
        .upcast();

        self.set_dirty(false);
        self.queue_draw();
    }

    fn margin_nodes(&self, nodes: &mut RefMut<'_, imp::Nodes>) -> gsk::RenderNode {
        let font = self.font();
        let vp = self.viewport_margins();
        let rows = self.imp().rows.borrow();

        let top = vp.top as usize;
        let bottom_start = rows.len() - vp.bottom as usize;
        let right_start = self.size().width - vp.right as usize;

        let margins = gsk::ContainerNode::new(&[
            nodes
                .margin_top
                .get_or_insert_with(|| {
                    gsk::ContainerNode::new(
                        rows[..top]
                            .iter()
                            .enumerate()
                            .map(|(i, row)| {
                                RenderNodeIter::new(row.cells.iter().peekable())
                                    .to_render_node(font.row_to_y(i as f64) as f32)
                            })
                            .collect::<Vec<_>>()
                            .as_ref(),
                    )
                    .upcast()
                })
                .clone(),
            nodes
                .margin_bottom
                .get_or_insert_with(|| {
                    gsk::ContainerNode::new(
                        rows[bottom_start..]
                            .iter()
                            .enumerate()
                            .map(|(i, row)| {
                                RenderNodeIter::new(row.cells.iter().peekable())
                                    .to_render_node(font.row_to_y((bottom_start + i) as f64) as f32)
                            })
                            .collect::<Vec<_>>()
                            .as_ref(),
                    )
                    .upcast()
                })
                .clone(),
            nodes
                .margin_sides
                .get_or_insert_with(|| {
                    gsk::ContainerNode::new(
                        rows[top..bottom_start]
                            .iter()
                            .enumerate()
                            .filter_map(|(i, row)| {
                                let left = RenderNodeIter::new(
                                    row.cells[..vp.left as usize].iter().peekable(),
                                )
                                .to_render_node(font.row_to_y((top + i) as f64) as f32);
                                let right =
                                    RenderNodeIter::new(row.cells[right_start..].iter().peekable())
                                        .to_render_node(font.row_to_y((top + i) as f64) as f32);

                                Some(gsk::ContainerNode::new(&[left, right]).upcast())
                            })
                            .collect::<Vec<_>>()
                            .as_ref(),
                    )
                    .upcast()
                })
                .clone(),
        ]);

        margins.upcast()
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

        // Move the contents and clear out the cell's render nodes.
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

            dst.clear_render_node();
            src.clear_render_node();
        }

        self.set_dirty(true);
    }
}

impl Default for GridBuffer {
    fn default() -> Self {
        Self::new()
    }
}
