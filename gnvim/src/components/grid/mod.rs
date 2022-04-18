use glib::Object;
use gtk::{glib, prelude::*, subclass::prelude::*};

use nvim::types::uievents::{GridLine, GridResize};

use crate::colors::Colors;

mod buffer;
mod imp;

glib::wrapper! {
    pub struct Grid(ObjectSubclass<imp::Grid>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Grid {
    pub fn new(id: i64) -> Self {
        Object::new(&[("grid-id", &id)]).expect("Failed to create Grid")
    }

    pub fn put(&self, event: GridLine) {
        let imp = self.imp();

        let mut buffer = imp.buffer.borrow_mut();
        let row = buffer.get_row(event.row as usize).expect("invalid row");

        row.update(&event);
    }

    pub fn resize(&self, event: GridResize) {
        self.imp()
            .buffer
            .borrow_mut()
            .resize(event.width as usize, event.height as usize);
    }

    pub fn flush(&self, colors: &Colors) {
        let h = 22.0;
        for (i, row) in self.imp().buffer.borrow_mut().rows.iter_mut().enumerate() {
            row.generate_nodes(&self.pango_context(), colors, i as f32 * h, h);
        }

        self.queue_draw();
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(0)
    }
}
