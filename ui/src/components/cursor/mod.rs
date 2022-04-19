use gtk::{glib, graphene, subclass::prelude::*, traits::WidgetExt};

use crate::{colors::Color, font::Font};

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

    pub fn move_to(&self, font: &Font, col: i64, row: i64, color: Color) {
        // TODO(ville): double_width
        let w = font.char_width();
        let h = font.height();

        let x = col as f32 * w;
        let y = row as f32 * h;

        let pos = graphene::Rect::new(x, y, w, h);

        let imp = self.imp();
        imp.pos.replace(pos);
        imp.color.replace(color);

        self.queue_draw();
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
