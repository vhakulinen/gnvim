use gtk::{graphene, gsk, pango, prelude::*};

use crate::colors::{Color, Colors};
use crate::font::Font;
use crate::SCALE;

/// Creates text render nodes for `text`, and adds the to `snapshot`.
pub fn render_text(
    snapshot: &gtk::Snapshot,
    ctx: &pango::Context,
    text: &str,
    color: &Color,
    attrs: &pango::AttrList,
    x: f32,
    y: f32,
) {
    let items = pango::itemize(ctx, text, 0, text.len() as i32, attrs, None);

    items.iter().fold(0.0_f32, |width, item| {
        let a = item.analysis();
        let offset = item.offset() as usize;
        let len = item.length() as usize;
        let mut glyphs = pango::GlyphString::new();
        let text = &text[offset..offset + len];

        pango::shape(text, a, &mut glyphs);

        let node = gsk::TextNode::new(
            &a.font(),
            &mut glyphs,
            color,
            &graphene::Point::new(x + width / SCALE, y),
        );

        // Empty glyphs (e.g. whitespace) won't get any nodes.
        if let Some(node) = node {
            snapshot.append_node(node.upcast());
        }

        width + glyphs.width() as f32
    });
}

pub fn create_hl_attrs(hl_id: i64, colors: &Colors, font: &Font) -> pango::AttrList {
    let attrs = pango::AttrList::new();

    attrs.insert(pango::AttrFontDesc::new(&font.font_desc()));

    if let Some(hl) = colors.get_hl(hl_id) {
        // TODO(ville): Rest of the attributes.

        if hl.bold.unwrap_or(false) {
            attrs.insert(pango::AttrInt::new_weight(pango::Weight::Bold));
        }

        if hl.italic.unwrap_or(false) {
            attrs.insert(pango::AttrInt::new_style(pango::Style::Italic));
        }
    }

    attrs
}
