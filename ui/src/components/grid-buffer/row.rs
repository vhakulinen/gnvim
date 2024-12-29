use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use gtk::{graphene, gsk, pango, prelude::*};

use nvim::types::uievents::GridLine;

use crate::{colors::Colors, font::Font, SCALE};

#[derive(Debug)]
pub struct CellNodes {
    pub fg: gsk::RenderNode,
    pub bg: gsk::RenderNode,
}

#[derive(Debug, Clone)]
pub struct Cell {
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
    pub fn width(&self) -> i64 {
        if self.double_width {
            2
        } else {
            i64::from(!self.text.is_empty())
        }
    }

    #[inline(always)]
    fn dirty(&self) -> bool {
        self.nodes.borrow().is_none()
    }
}

struct LineSegment<'a> {
    first: &'a mut Cell,
    rest: Vec<&'a mut Cell>,
}

impl<'a> LineSegment<'a> {
    fn new(cell: &'a mut Cell) -> Self {
        LineSegment {
            // Double width cell will always get one other cell joined to it.
            rest: if cell.double_width {
                Vec::with_capacity(1)
            } else {
                vec![]
            },
            first: cell,
        }
    }

    fn join(&mut self, cell: &'a mut Cell) {
        self.rest.push(cell);
    }

    fn set_nodes(&mut self, nodes: Rc<RefCell<Option<CellNodes>>>) {
        self.first.nodes = nodes.clone();
        self.rest
            .iter_mut()
            .for_each(|cell| cell.nodes = nodes.clone());
    }

    #[inline(always)]
    fn hl_id(&self) -> i64 {
        self.first.hl_id
    }

    #[inline(always)]
    fn width(&self) -> i64 {
        self.first.width() + self.rest.iter().map(|cell| cell.width()).sum::<i64>()
    }

    #[inline(always)]
    fn dirty(&self) -> bool {
        self.first.dirty() || self.rest.iter().any(|cell| cell.dirty())
    }

    fn text(&self) -> String {
        let mut text = self.first.text.clone();
        self.rest.iter().for_each(|cell| text += &cell.text);
        text
    }

    #[inline(always)]
    fn double_width(&self) -> bool {
        self.first.double_width
    }

    fn node(&self) -> Rc<RefCell<Option<CellNodes>>> {
        self.first.nodes.clone()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Row {
    pub cells: Vec<Cell>,
    node: Option<gsk::RenderNode>,
}

impl Row {
    pub fn clear(&mut self) {
        self.clear_render_node();
        self.cells = vec![Cell::default(); self.cells.len()];
    }

    pub fn clear_render_node(&mut self) {
        self.node.take();
    }

    pub fn to_render_node(
        &mut self,
        ctx: &pango::Context,
        colors: &Colors,
        font: &Font,
        row_index: usize,
    ) -> gsk::RenderNode {
        // If we have a node cached, return it.
        if let Some(node) = &self.node {
            return node.clone();
        }

        let (bg, fg) = self
            .generate_nodes(&ctx, colors, &font)
            .filter_map(|nodes| {
                nodes
                    .borrow()
                    .as_ref()
                    .map(|nodes| (nodes.bg.clone(), nodes.fg.clone()))
            })
            .collect::<(Vec<_>, Vec<_>)>();

        let node = gsk::TransformNode::new(
            gsk::ContainerNode::new(&[
                gsk::ContainerNode::new(&bg).upcast(),
                gsk::ContainerNode::new(&fg).upcast(),
            ]),
            &gsk::Transform::new().translate(&graphene::Point::new(
                0.0,
                font.row_to_y(row_index as f64) as f32,
            )),
        )
        .upcast();

        // Cache the node.
        self.node = Some(node.clone());

        node
    }

    pub fn update(&mut self, event: &GridLine) {
        self.clear_render_node();
        let mut hl_id = event
            .data
            .first()
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
                let cell = iter.next().expect("too long grid line event");
                cell.hl_id = hl_id;
                cell.text.clone_from(&data.text);
                cell.double_width = double_width;
                cell.clear_nodes();
            }
        }
    }

    /// Generate the render nodes for the whole row.
    ///
    /// Returned iterator must be consumed to complete the generation process.
    fn generate_nodes<'a>(
        &'a mut self,
        ctx: &'a pango::Context,
        colors: &'a Colors,
        font: &'a Font,
    ) -> impl Iterator<Item = Rc<RefCell<Option<CellNodes>>>> + 'a {
        let baseline = font.baseline() / SCALE;
        let bg_h = font.height() / SCALE;
        let ch = font.char_width();
        let mut x = 0.0_f32;

        SegmentIterator::new(self.cells.iter_mut()).map(move |mut segment| {
            let segment_width = segment.width();
            let width = segment_width as f32 * ch / SCALE;

            // If the segment doesn't need to be rerendered, return the
            // previous node.
            if !segment.dirty() {
                x += width;
                return segment.node();
            }

            let hl = colors.get_hl(&segment.hl_id());
            let attrs = crate::render::create_hl_attrs(&hl, font);

            let fg = hl.fg();
            let bg = hl.bg();
            let sp = hl.sp();
            let hl = hl.hl_attr();

            // Create glyphs.
            let mut nodes = vec![crate::render::render_text(
                ctx,
                &segment.text(),
                fg,
                &attrs,
                x,
                baseline,
            )];

            if hl.and_then(|hl| hl.underline).unwrap_or(false) {
                nodes.push(crate::render::render_underline(
                    font, sp, x, baseline, width,
                ));
            }

            if hl.and_then(|hl| hl.underlineline).unwrap_or(false) {
                nodes.extend(crate::render::render_underlineline(
                    font, sp, x, baseline, width,
                ));
            }

            if hl.and_then(|hl| hl.strikethrough).unwrap_or(false) {
                nodes.push(crate::render::render_strikethrough(
                    font, fg, x, baseline, width,
                ));
            }

            if hl.and_then(|hl| hl.undercurl).unwrap_or(false) {
                nodes.push(crate::render::render_undercurl(
                    font,
                    sp,
                    x,
                    baseline,
                    width,
                    segment_width,
                ));
            }

            if hl.and_then(|hl| hl.underdot).unwrap_or(false) {
                nodes.push(crate::render::render_underdot(font, sp, x, baseline, width));
            }

            if hl.and_then(|hl| hl.underdash).unwrap_or(false) {
                nodes.push(crate::render::render_underdash(
                    font, sp, x, baseline, width,
                ));
            }

            let nodes = Rc::new(RefCell::new(Some(CellNodes {
                fg: gsk::ContainerNode::new(&nodes).upcast(),
                bg: gsk::ColorNode::new(bg, &graphene::Rect::new(x, 0.0, width, bg_h)).upcast(),
            })));

            // Store the nodes into the segment's cells.
            segment.set_nodes(nodes.clone());

            x += width;

            return nodes;
        })
    }
}

struct SegmentIterator<'a> {
    cells: std::slice::IterMut<'a, Cell>,
    current_segment: Option<LineSegment<'a>>,
}

impl<'a> SegmentIterator<'a> {
    fn new(cells: std::slice::IterMut<'a, Cell>) -> Self {
        Self {
            cells,
            current_segment: None,
        }
    }
}

impl<'a> Iterator for SegmentIterator<'a> {
    type Item = LineSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cell) = self.cells.next() {
            // Double width cells are a special case.
            if cell.double_width {
                // Create a new segment.
                let mut new = LineSegment::new(cell);

                // Join the following cell. It _should_ be a empty cell. We
                // want it joined to handle the render nodes correctly.
                new.join(
                    self.cells
                        .next()
                        .expect("double width cell must be followed by an empty cell"),
                );

                // If there is a previous segment pending, return it first.
                if let Some(segment) = self.current_segment.take() {
                    self.current_segment = Some(new);
                    return Some(segment);
                }

                // Return the new segment.
                return Some(new);
            }

            match &mut self.current_segment {
                // Previous is a double width segment, return it.
                Some(segment) if segment.double_width() => {
                    return self.current_segment.replace(LineSegment::new(cell));
                }
                // Adjacent cells with same hl id should be joined.
                Some(ref mut segment) if segment.hl_id() == cell.hl_id => {
                    segment.join(cell);
                    continue;
                }
                // We can't join with previous segment, return it and start
                // collecting a new one.
                Some(_segment) => {
                    return self.current_segment.replace(LineSegment::new(cell));
                }
                // Start a new segment.
                None => {
                    self.current_segment = Some(LineSegment::new(cell));
                    continue;
                }
            }
        }

        self.current_segment.take()
    }
}
