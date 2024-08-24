use gtk::{cairo, gdk, graphene, gsk, pango, prelude::*};

use crate::colors::{Color, Colors};
use crate::font::Font;
use crate::SCALE;

/// Creates text render nodes for `text`.
pub fn render_text(
    ctx: &pango::Context,
    text: &str,
    color: &Color,
    attrs: &pango::AttrList,
    x: f32,
    baseline: f32,
) -> gsk::RenderNode {
    let items = pango::itemize(ctx, text, 0, text.len() as i32, attrs, None);

    let mut width = 0.0_f32;
    let nodes = items
        .iter()
        .filter_map(|item| {
            let a = item.analysis();
            let offset = item.offset() as usize;
            let len = item.length() as usize;
            let mut glyphs = pango::GlyphString::new();
            let text = &text[offset..offset + len];

            pango::shape(text, a, &mut glyphs);

            let node = gsk::TextNode::new(
                &a.font(),
                &glyphs,
                color,
                &graphene::Point::new(x + width / SCALE, baseline),
            );

            width += glyphs.width() as f32;

            // Empty glyphs (e.g. whitespace) won't get any nodes.
            node.map(|node| node.upcast())
        })
        .collect::<Vec<gsk::RenderNode>>();

    gsk::ContainerNode::new(&nodes).upcast()
}

pub fn render_underline(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
) -> gsk::RenderNode {
    let y = baseline - font.underline_position() / SCALE;

    let node = gsk::ColorNode::new(
        color,
        &graphene::Rect::new(x, y, width, font.underline_thickness() / SCALE),
    );

    node.upcast()
}

pub fn render_underlineline(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
) -> [gsk::RenderNode; 2] {
    render_underline(font, color, x, baseline, width);

    let baseline2 = baseline + font.underline_thickness() / SCALE * 2.0;
    render_underline(font, color, x, baseline2, width);

    [
        render_underline(font, color, x, baseline, width),
        render_underline(font, color, x, baseline2, width),
    ]
}

pub fn render_strikethrough(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
) -> gsk::RenderNode {
    let y = baseline - font.strikethrough_position() / SCALE;

    let node = gsk::ColorNode::new(
        color,
        &graphene::Rect::new(x, y, width, font.strikethrough_thickness() / SCALE),
    );

    node.upcast()
}

pub fn render_underdash(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
) -> gsk::RenderNode {
    let y = baseline - font.underline_position() / SCALE;
    let h = font.descent() / SCALE;
    let dash_width = (font.char_width() * 0.3 / SCALE) as f64;
    let thickness = font.underline_thickness() / SCALE;

    let node = gsk::CairoNode::new(&graphene::Rect::new(x, y, width, h));
    let ctx = node.draw_context();

    let x = x as f64;
    let y = (y + thickness) as f64;
    ctx.move_to(x, y);
    ctx.line_to(x + width as f64, y);
    ctx.set_line_width(thickness as f64);
    ctx.set_dash(&[dash_width], 0.0);
    ctx.set_source_rgba(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
        color.alpha() as f64,
    );
    ctx.stroke().expect("failed to draw with cairo");

    node.upcast()
}

pub fn render_underdot(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
) -> gsk::RenderNode {
    let y = baseline - font.underline_position() / SCALE;
    // NOTE(ville): The dot line is significatly harder to make out compared to
    // a solid line, so make the dot size bigger (compared to plain underline).
    let h = 2.0 * font.underline_thickness() / SCALE;

    // Create a dot, and color it.
    let bounds = graphene::Rect::new(x, y, h, h);
    let dot_node = gsk::RoundedClipNode::new(
        gsk::ColorNode::new(color, &bounds),
        &gsk::RoundedRect::from_rect(bounds, h),
    );

    // Add transparent "dummy" block so we can add some spacing between dots.
    let space_node = gsk::ColorNode::new(
        &gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
        &graphene::Rect::new(x, y, h * 1.5, h),
    );

    // Repeat our dot across the given area.
    let node = gsk::RepeatNode::new(
        &graphene::Rect::new(x, y, width, h),
        gsk::ContainerNode::new(&[dot_node.upcast(), space_node.upcast()]),
        None,
    );

    node.upcast()
}

pub fn render_undercurl(
    font: &Font,
    color: &Color,
    x: f32,
    baseline: f32,
    width: f32,
    cell_count: i64,
) -> gsk::RenderNode {
    let y = baseline - font.underline_position() / SCALE;
    let h = font.descent() / SCALE;

    let bounds = graphene::Rect::new(x, y, width, h);
    let node = gsk::CairoNode::new(&bounds);
    let ctx = node.draw_context();

    let x = x as f64;
    let y = y as f64;
    let w = (font.char_width() / SCALE) as f64;
    let h = h as f64;

    let y_top = y - h * 0.5;
    let y_bot = y + h * 1.5;
    let y_mid = y + h * 0.5;

    ctx.set_line_width((font.underline_thickness() / SCALE) as f64);
    ctx.move_to(x, y_mid);
    for i in 0..cell_count {
        let x = x + w * i as f64;

        let x1 = x + w * 0.25;
        let y1 = y_bot;
        let x2 = x + w * 0.25;
        let y2 = y_top;
        let x3 = x + w * 0.5;
        let y3 = y_mid;

        ctx.curve_to(x1, y1, x2, y2, x3, y3);

        let x1 = x + w * 0.75;
        let y1 = y_bot;
        let x2 = x + w * 0.75;
        let y2 = y_top;
        let x3 = x + w;
        let y3 = y_mid;

        ctx.curve_to(x1, y1, x2, y2, x3, y3);
    }
    ctx.set_source_rgba(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
        color.alpha() as f64,
    );
    ctx.set_line_cap(cairo::LineCap::Square);
    ctx.stroke().expect("failed to draw with cairo");

    node.upcast()
}

pub fn create_hl_attrs(hl_id: &i64, colors: &Colors, font: &Font) -> pango::AttrList {
    let attrs = pango::AttrList::new();

    attrs.insert(pango::AttrFontDesc::new(&font.font_desc()));

    if let Some(hl) = colors.get_hl(hl_id).hl_attr() {
        if hl.bold.unwrap_or(false) {
            attrs.insert(pango::AttrInt::new_weight(pango::Weight::Bold));
        }

        if hl.italic.unwrap_or(false) {
            attrs.insert(pango::AttrInt::new_style(pango::Style::Italic));
        }
    }

    attrs
}
