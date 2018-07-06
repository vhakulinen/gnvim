use gtk::DrawingArea;
use gtk::prelude::*;
use cairo;
use pango;
use pangocairo;

use nvim_bridge::GridLineSegment;
use ui::grid::context::Context;
use ui::ui::HlDefs;
use ui::color::{Highlight, Color};


pub fn put_line(da: &DrawingArea, context: &mut Context, line: &GridLineSegment, hl_defs: &mut HlDefs) {
    let row = line.row;
    let mut col = line.col_start;
    let cw = context.cell_metrics.width;
    let ch = context.cell_metrics.height;
    let cr = &context.cairo_context;

    for cell in line.cells.iter() {

        let s = cell.text.repeat(cell.repeat as usize);

        if let Some(hl_id) = cell.hl_id {
            context.current_hl = *hl_defs.get(&hl_id).unwrap();
        }

        let hl = &context.current_hl;

        let (fg, bg) = if hl.reverse {
            (
                hl.background.unwrap_or(context.default_bg),
                hl.foreground.unwrap_or(context.default_fg),
            )
        } else {
            (
                hl.foreground.unwrap_or(context.default_fg),
                hl.background.unwrap_or(context.default_bg),
            )
        };

        let len = s.chars().count();
        let x = col as f64 * cw;
        let y = row as f64 * ch;
        let w = len as f64 * cw;
        let h = ch;

        cr.save();
        cr.set_source_rgb(bg.r, bg.g, bg.b);
        cr.rectangle(x, y, w, h);
        cr.fill();
        cr.restore();

        let attrs = pango::AttrList::new();
        let items = pango::itemize(&context.pango_context, s.as_str(), 0, s.len() as i32, &attrs, None);

        cr.save();
        cr.set_operator(cairo::Operator::Over);
        cr.set_source_rgb(fg.r, fg.g, fg.b);
        let mut offset = 0.0;
        for item in items {

            let a = item.analysis();
            let font = a.font();
            let mut glyphs = pango::GlyphString::new();
            pango::shape(s.as_str(), &a, &mut glyphs);

            //cr.move_to(x + offset * context.cell_metrics.width, y + context.cell_metrics.ascent);
            cr.move_to(x + offset, y + context.cell_metrics.ascent);
            pangocairo::functions::show_glyph_string(&cr, &font, &mut glyphs);

            offset += glyphs.get_width() as f64;
        }
        cr.restore();

        col += len as u64;
        da.queue_draw_area(x as i32, y as i32, w as i32, h as i32);
    }
}

pub fn clear(da: &DrawingArea, ctx: &Context) {
    let cr = &ctx.cairo_context;
    let w = da.get_allocated_width();
    let h = da.get_allocated_height();
    let bg = &ctx.default_bg;

    cr.save();
    cr.set_source_rgb(bg.r, bg.g, bg.b);
    cr.rectangle(0.0, 0.0, w as f64, h as f64);
    cr.fill();
    cr.restore();

    da.queue_draw_area(0, 0, w, h);
}
