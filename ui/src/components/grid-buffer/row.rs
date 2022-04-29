use std::{
    cell::{Ref, RefCell},
    iter::Peekable,
    rc::Rc,
};

use gtk::{graphene, gsk, pango, prelude::*};

use nvim::types::uievents::GridLine;

use crate::{colors::Colors, font::Font};

#[derive(Debug)]
pub struct CellNodes {
    pub fg: gsk::RenderNode,
    pub bg: gsk::RenderNode,
}

#[derive(Debug, Clone)]
pub struct Cell {
    // TODO(ville): Use Cow for text.
    pub text: String,
    pub hl_id: i64,
    pub double_width: bool,
    /// Cell's render nodes. Render nodes might be shared across cells (e.g.
    /// for ligatures).
    pub nodes: Rc<RefCell<Option<CellNodes>>>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: String::from(" "),
            hl_id: 0,
            double_width: false,

            nodes: Rc::new(RefCell::new(None)),
        }
    }
}

impl Cell {
    pub fn clear_nodes(&mut self) {
        self.nodes.borrow_mut().take();
    }

    /// Width of this cell on the grid. A cell might have length of 1, 2 or
    /// zero. Normal, double width, zero width. Zero width happens when the
    /// cell is right of a double width cell.
    pub fn width(&mut self) -> i64 {
        if self.double_width {
            2
        } else if self.text.is_empty() {
            0
        } else {
            1
        }
    }
}

struct LineSegment<'a> {
    hl_id: i64,
    cells: Vec<&'a mut Cell>,
    dirty: bool,
    width: i64,
}

#[derive(Default, Debug, Clone)]
pub struct Row {
    pub cells: Vec<Cell>,
}

impl Row {
    pub fn clear(&mut self) {
        self.cells = vec![Cell::default(); self.cells.len()];
    }

    pub fn update(&mut self, event: &GridLine) {
        let mut hl_id = event
            .data
            .get(0)
            .expect("grid line event cant be empty")
            .hl_id
            .expect("first item should have hl_id");

        let start = event.col_start as usize;

        let mut iter = self.cells.iter_mut().skip(start);
        let mut data_iter = event.data.iter().peekable();
        while let Some(data) = data_iter.next() {
            if let Some(id) = data.hl_id {
                hl_id = id;
            }

            let double_width = data_iter
                .peek()
                .map(|peek| peek.text.is_empty())
                .unwrap_or(false);

            for _ in 0..data.repeat.unwrap_or(1) {
                let mut cell = iter.next().expect("too long grid line event");
                cell.hl_id = hl_id;
                cell.text = data.text.clone();
                cell.double_width = double_width;
                cell.clear_nodes();
            }
        }
    }

    pub fn render_node_iter<'a>(&'a self) -> RenderNodeIter<'a> {
        RenderNodeIter {
            inner: self.cells.iter().peekable(),
        }
    }

    pub fn generate_nodes(
        &mut self,
        ctx: &pango::Context,
        colors: &Colors,
        font: &Font,
        y_offset: f32,
        height: f32,
    ) {
        // Gather cells into continuous segments based on hl ids.
        let mut segments = self
            .cells
            .iter_mut()
            // TODO(ville): Try to use a iterator to avoid unnecessary allocations.
            .fold(Vec::<LineSegment>::new(), |mut acc, cell| {
                let dirty = cell.nodes.borrow().is_none();
                let width = cell.width();

                // TODO(ville): Render double width characters independently

                match acc.last_mut() {
                    Some(prev) if prev.hl_id == cell.hl_id => {
                        prev.cells.push(cell);
                        prev.dirty = dirty || prev.dirty;
                        prev.width += width;
                    }
                    _ => acc.push(LineSegment {
                        hl_id: cell.hl_id,
                        cells: vec![cell],
                        width,
                        dirty,
                    }),
                };

                acc
            });

        let attrs = pango::AttrList::new();
        attrs.insert(pango::AttrFontDesc::new(&font.font_desc()));
        // TODO(ville): Add bold, italics etc. attrs.

        let scale = pango::SCALE as f32;
        let ascent = font.ascent();

        let mut x_offset = 0.0_f32;
        for segment in segments.iter_mut() {
            if !segment.dirty {
                x_offset += font.char_width() * segment.cells.len() as f32;
                continue;
            }

            let snapshot_fg = gtk::Snapshot::new();
            let snapshot_bg = gtk::Snapshot::new();

            let text = segment
                .cells
                .iter()
                .map(|cell| cell.text.clone())
                .collect::<String>();

            let items = pango::itemize(&ctx, &text, 0, text.len() as i32, &attrs, None);

            let fg = colors.get_hl_fg(segment.hl_id);
            let bg = colors.get_hl_bg(segment.hl_id);

            let mut width = 0.0_f32;
            for item in items {
                let a = item.analysis();
                let offset = item.offset() as usize;
                let len = item.length() as usize;
                let mut glyphs = pango::GlyphString::new();
                let text = &text[offset..offset + len];

                pango::shape(text, a, &mut glyphs);

                let node = gsk::TextNode::new(
                    &a.font(),
                    &mut glyphs,
                    &fg,
                    // TODO(ville): Double check that the x and y values are correct.
                    &graphene::Point::new(x_offset + width, y_offset + ascent),
                );

                // Empty glyphs (e.g. whitespace) won't get any nodes.
                if let Some(node) = node {
                    snapshot_fg.append_node(node.upcast());
                }

                width += glyphs.width() as f32 / scale;
            }

            snapshot_bg.append_node(
                gsk::ColorNode::new(&bg, &graphene::Rect::new(x_offset, y_offset, width, height))
                    .upcast(),
            );

            let nodes = Rc::new(RefCell::new(Some(CellNodes {
                fg: snapshot_fg
                    .to_node()
                    .unwrap_or(gsk::ContainerNode::new(&[]).upcast()),
                bg: snapshot_bg
                    .to_node()
                    .unwrap_or(gsk::ContainerNode::new(&[]).upcast()),
            })));
            segment.cells.iter_mut().for_each(|cell| {
                cell.nodes = nodes.clone();
            });

            x_offset += font.char_width() * segment.width as f32;
        }
    }
}

pub struct RenderNodeIter<'a> {
    inner: Peekable<std::slice::Iter<'a, Cell>>,
}

impl<'a> Iterator for RenderNodeIter<'a> {
    type Item = Ref<'a, Option<CellNodes>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cell) = self.inner.next() {
            if let Some(next) = self.inner.peek() {
                if Rc::ptr_eq(&cell.nodes, &next.nodes) {
                    continue;
                }
            }

            return Some(cell.nodes.borrow());
        }

        None
    }
}
