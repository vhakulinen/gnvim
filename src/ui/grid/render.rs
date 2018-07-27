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
    let affected_segments = context.rows.get_mut(line.row as usize).unwrap().update(line);

    let row = line.row;
    let cw = context.cell_metrics.width;
    let ch = context.cell_metrics.height;
    let cr = &context.cairo_context;

    //let mut bg = Color::from_u64(0);
    //bg.r = 0.0;
    //bg.g = 1.0;

    //let offset = 0.3;
    for seg in affected_segments {
        //bg.r += offset;
        //if bg.r >= 1.0 {
            //bg.r = 0.0;
        //}
        let col = seg.start;

        //if let Some(hl_id) = seg.leaf.hl_id {
            //context.current_hl = *hl_defs.get(&hl_id).unwrap();
        //}
        context.current_hl = *hl_defs.get(&seg.leaf.hl_id()).unwrap();

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

        let s = &seg.leaf.text();
        let len = seg.len;
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
        let items = pango::itemize(&context.pango_context, s.as_str(), 0, s.len() as i32, &attrs , None);

        cr.save();
        cr.set_operator(cairo::Operator::Over);
        cr.set_source_rgb(fg.r, fg.g, fg.b);
        let mut offset = 0.0;
        for item in items {

            let a = item.analysis();
            let font = a.font();
            let mut glyphs = pango::GlyphString::new();
            pango::shape(s.as_str(), &a, &mut glyphs);

            //cr.move_to(x + offset * context.cell_metrics.width, y + context.cell_metrics.ascen t);
            cr.move_to(x + offset, y + context.cell_metrics.ascent);
            pangocairo::functions::show_glyph_string(&cr, &font, &mut glyphs);

            offset += glyphs.get_width() as f64;
        }

        cr.restore();

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

pub fn scroll(da: &DrawingArea, ctx: &mut Context, reg: [u64;4], count: i64) {
    let cr = &ctx.cairo_context;
    let cm = &ctx.cell_metrics;
    let bg = &ctx.default_bg;

    let s = cr.get_target();

    let top = reg[0];
    let bot = reg[1];
    let left = reg[2];
    let right = reg[3];

    let (
        src_top, src_bot,
        dst_top, dst_bot,
        clr_top, clr_bot,
        ) = if count > 0 {
        let ( src_top, src_bot ) = ((top as i64 + count) as f64, bot as f64);
        let ( dst_top, dst_bot ) = (top as f64, (bot as i64 - count) as f64);
        (
            src_top, src_bot,
            dst_top, dst_bot,
            dst_bot, src_bot,
        )
    } else {
        let ( src_top, src_bot ) = (top as f64, (bot as i64 + count) as f64);
        let ( dst_top, dst_bot ) = ((top as i64 - count) as f64, bot as f64);
        (
            src_top, src_bot,
            dst_top, dst_bot,
            src_top, dst_top,
        )
    };

    // Modify the rows stored data of the rows.
    let mut src = vec!();
    for i in src_top as usize..src_bot as usize {
        let row = ctx.rows.get(i).unwrap().clone();
        let part = row.copy_range(left as usize, right as usize).clone();
        src.push(part);
    }
    src.reverse();
    
    for i in dst_top as usize..dst_bot as usize {
        ctx.rows.get_mut(i).unwrap().insert_rope_at(left as usize, src.pop().unwrap());
    }

    for i in clr_top as usize..clr_bot as usize {
        ctx.rows.get_mut(i).unwrap().clear_range(left as usize, right as usize);
    }

    // Draw move the scrolled part on the cairo surface.
    cr.save();

    // Create pattern which we can then "safely" draw to the surface. On X11, the pattern part was
    // not needed but on wayland it is - I suppose it has something to do with the underlaying
    // backbuffer.
    cr.push_group();
    let (_, y) = get_coords(cm.height, cm.width, dst_top - src_top, 0.0);
    cr.set_source_surface(&s, 0.0, y);
    cr.set_operator(cairo::Operator::Source);
    let (x1, y1, x2, y2) = get_rect(cm.height, cm.width, dst_top, dst_bot, left as f64, right as f64);
    let w = x2 - x1;
    let h = y2 - y1;
    cr.rectangle(x1, y1, w, h);
    cr.fill();

    // Get the pattern.
    let mut p = cr.pop_group();

    // Draw the parttern.
    cr.set_source(&mut p);
    cr.set_operator(cairo::Operator::Source);
    cr.rectangle(x1, y1, w, h);
    cr.fill();
    da.queue_draw_area(x1 as i32, y1 as i32, w as i32, h as i32);

    // Clear the area that is left "dirty".
    let (x1, y1, x2, y2) = get_rect(cm.height, cm.width, clr_top, clr_bot, left as f64, right as f64);
    let w = x2 - x1;
    let h = y2 - y1;
    cr.rectangle(x1, y1, x2 - x1, y2 - y1);
    cr.set_source_rgb(bg.r, bg.g, bg.b);
    cr.fill();
    da.queue_draw_area(x1 as i32, y1 as i32, w as i32, h as i32);

    cr.restore();
}

pub fn get_rect(col_h: f64, col_w: f64, top: f64, bot: f64, left: f64, right: f64) -> (f64, f64, f64, f64) {
    let (x1, y1) = get_coords(col_h, col_w, top, left);
    let (x2, y2) = get_coords(col_h, col_w, bot, right);
    (x1, y1, x2, y2)
}

pub fn get_coords(h: f64, w: f64, row: f64, col: f64) -> (f64, f64) {
    let x = col * w;
    let y = row * h;
    (x, y)
}
