use gtk::{glib, graphene, gsk, prelude::*, subclass::prelude::*};

use crate::{colors::Colors, math::ease_out_cubic, warn, SCALE};

use super::grid_buffer::row::Cell;

mod blink;
mod imp;

glib::wrapper! {
    pub struct Cursor(ObjectSubclass<imp::Cursor>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Cursor {
    fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Cursor")
    }

    pub fn flush(&self, colors: &Colors) {
        let imp = self.imp();
        if imp.node.borrow().is_some() {
            return;
        }

        let font = imp.font.borrow();
        let hl_id = imp.attr_id.borrow();
        let hl = colors.get_hl(&hl_id);
        let fg = hl.fg();
        let bg = hl.bg();
        // For hl id zero, we need to flip fg and bg.
        let (fg, bg) = if *hl_id == 0 { (bg, fg) } else { (fg, bg) };

        let height = font.height();
        let ch = font.char_width();

        let snapshot = gtk::Snapshot::new();

        let width = if *imp.double_width.borrow() {
            ch * 2.0 / SCALE
        } else {
            ch / SCALE
        };
        let width = width * *imp.width_percentage.borrow();
        let rect = graphene::Rect::new(0.0, 0.0, width, height / SCALE);

        // Clip the area where we're drawing. This avoids a issue when the cursor
        // is narrow, yet we're drawing our own _whole_ cell. Clipping clips
        // _our_ render node to our _width_ and thus' the underlying grid cell
        // will be visible instead.
        snapshot.push_clip(&rect);

        snapshot.append_node(gsk::ColorNode::new(bg, &rect).upcast());

        let attrs = crate::render::create_hl_attrs(&hl_id, colors, &font);
        crate::render::render_text(
            &snapshot,
            &self.pango_context(),
            &imp.text.borrow(),
            fg,
            &attrs,
            0.0,
            font.baseline() / SCALE,
        );

        snapshot.pop();

        let node = snapshot
            .to_node()
            .unwrap_or_else(|| gsk::ContainerNode::new(&[]).upcast());

        imp.node.replace(Some(node));

        self.queue_draw();
    }

    pub fn row(&self) -> i64 {
        return self.imp().pos.borrow().grid.1;
    }

    pub fn col(&self) -> i64 {
        return self.imp().pos.borrow().grid.0;
    }

    pub fn move_to(&self, cell: &Cell, col: i64, row: i64) {
        // TODO(ville): To avoid jumpy initial cursor position, the position
        // should be set directly on the "first" call. See the previous
        // implementation: https://github.com/vhakulinen/gnvim/blob/6ba47373545c6e9be3677a3ae695415809cf2fdf/src/ui/grid/cursor.rs#L46-L49.

        let imp = self.imp();

        imp.text.replace(cell.text.clone());
        imp.double_width.replace(cell.double_width);
        imp.pos.borrow_mut().grid = (col, row);

        let start = self
            .frame_clock()
            .expect("failed to get frame clock")
            .frame_time() as f64;
        if let Some(ref mut blink) = *imp.blink.borrow_mut() {
            blink.reset_to_wait(start);
        }

        let font = imp.font.borrow();
        let target = (
            col as f64 * (font.char_width() / SCALE) as f64,
            row as f64 * (font.height() / SCALE) as f64,
        );
        let start_pos = imp.pos.borrow().pos;
        let end = start + imp.pos.borrow().transition;
        let old_id =
            imp.pos_tick
                .borrow_mut()
                .replace(self.add_tick_callback(move |this, clock| {
                    let now = clock.frame_time() as f64;
                    if now < start {
                        warn!("Clock going backwards");
                        return Continue(true);
                    }

                    let imp = this.imp();
                    if now < end {
                        let t = ease_out_cubic((now - start) / (end - start));
                        let col = start_pos.0 + ((target.0 - start_pos.0) * t);
                        let row = start_pos.1 + ((target.1 - start_pos.1) * t);
                        imp.pos.borrow_mut().pos = (col, row);
                        this.queue_draw();

                        Continue(true)
                    } else {
                        imp.pos.borrow_mut().pos = target;
                        this.queue_draw();

                        Continue(false)
                    }
                }));

        if let Some(old_id) = old_id {
            old_id.remove();
        }

        // Clear the render node.
        imp.node.replace(None);
    }

    pub fn set_text(&self, text: String) {
        let imp = self.imp();
        imp.text.replace(text);
        imp.node.replace(None);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
