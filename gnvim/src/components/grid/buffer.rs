use gtk::{gdk, graphene, gsk, pango, prelude::*};

use nvim::types::uievents::GridLine;

use crate::colors::Colors;

#[derive(Default, Debug, Clone)]
pub struct Cell {
    pub text: String,
    pub hl_id: i64,
    pub double_width: bool,
}

struct LineSegment {
    hl_id: i64,
    text: String,
}

#[derive(Default, Debug)]
pub struct Row {
    pub cells: Vec<Cell>,
    pub fg_nodes: Vec<gsk::RenderNode>,
    pub bg_nodes: Vec<gsk::RenderNode>,

    dirty: bool,
}

impl Row {
    pub fn update(&mut self, event: &GridLine) {
        self.dirty = true;

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
            }
        }
    }

    pub fn generate_nodes(
        &mut self,
        ctx: &pango::Context,
        colors: &Colors,
        y_offset: f32,
        height: f32,
    ) {
        if !self.dirty {
            return;
        }

        self.fg_nodes.clear(); // Make sure the glyphs are cleared.
        self.bg_nodes.clear(); // Make sure the glyphs are cleared.

        let segments = self
            .cells
            .iter()
            .fold(Vec::<LineSegment>::new(), |mut acc, cell| {
                let next = match acc.last_mut() {
                    Some(prev) if prev.hl_id == cell.hl_id => {
                        prev.text.push_str(cell.text.as_ref());
                        None
                    }
                    _ => Some(LineSegment {
                        text: cell.text.clone(),
                        hl_id: cell.hl_id,
                    }),
                };

                if let Some(next) = next {
                    acc.push(next);
                }

                acc
            });

        let attrs = pango::AttrList::new();
        let scale = pango::SCALE as f32;

        let mut x_offset = 0.0_f32;
        for segment in segments.iter() {
            let items = pango::itemize(
                &ctx,
                &segment.text,
                0,
                segment.text.len() as i32,
                &attrs,
                None,
            );

            let fg = colors.get_hl_fg(segment.hl_id);
            let bg = colors.get_hl_bg(segment.hl_id);

            let mut width = 0.0_f32;
            for item in items {
                let a = item.analysis();
                let item_offset = item.offset() as usize;
                let mut glyphs = pango::GlyphString::new();
                let text = &segment.text[item_offset..item_offset + item.length() as usize];

                pango::shape(text, a, &mut glyphs);

                let mut node = gsk::TextNode::new(
                    &a.font(),
                    &mut glyphs,
                    &gdk::RGBA::new(fg.r as f32, fg.g as f32, fg.b as f32, 1.0),
                    &graphene::Point::new(x_offset + width, y_offset + height),
                );

                if let Some(node) = node.take() {
                    self.fg_nodes.push(node.upcast());
                } else {
                    println!("Failed to create text node for text '{}'", text);
                }

                width += glyphs.width() as f32 / scale;
            }

            self.bg_nodes.push(
                gsk::ColorNode::new(
                    &gdk::RGBA::new(bg.r as f32, bg.g as f32, bg.b as f32, 1.0),
                    &graphene::Rect::new(x_offset, y_offset, width, height),
                )
                .upcast(),
            );

            x_offset += width;
        }
    }
}

#[derive(Default, Debug)]
pub struct Buffer {
    pub rows: Vec<Row>,
}

impl Buffer {
    pub fn get_row(&mut self, idx: usize) -> Option<&mut Row> {
        self.rows.get_mut(idx)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.rows.resize_with(height, Default::default);

        for row in self.rows.iter_mut() {
            row.cells.resize(width, Cell::default())
        }
    }
}
