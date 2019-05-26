use pango;
use gtk;
use gtk::prelude::*;

use ui::ui::HlDefs;
use ui::font::{Font, FontUnit};
use ui::color::Color;
use nvim_bridge::{MsgShow, MsgShowKind};

struct Message {
    frame: gtk::Box,
    label: gtk::Label,
    kind: gtk::Image,
}

impl Message {
    fn new(msg: &MsgShow, hl_defs: &HlDefs, css_provider: &gtk::CssProvider, size: f64) -> Self {
        let label = gtk::Label::new(None);

        let mut content = String::new();

        for chunk in msg.content.iter() {
            let hl = hl_defs.get(&chunk.0).unwrap();
            let markup = hl.pango_markup(
                &chunk.1,
                &hl_defs.default_fg,
                &hl_defs.default_bg,
                &hl_defs.default_sp,
            );

            content += &markup;
        }

        label.set_markup(&content);
        label.set_halign(gtk::Align::Start);
        label.set_line_wrap(true);
        label.set_line_wrap_mode(pango::WrapMode::WordChar);
        label.set_xalign(0.0);

        let frame = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        frame.set_halign(gtk::Align::End);
        frame.set_valign(gtk::Align::Start);

        let buf = get_icon_pixbuf(&msg.kind, &hl_defs.default_fg, size);
        let kind = gtk::Image::new_from_pixbuf(Some(&buf));

        frame.pack_start(&kind, false, true, 0);
        frame.pack_start(&label, false, true, 0);

        add_css_provider!(css_provider, frame, label, kind);

        Self {
            frame,
            label,
            kind,
        }
    }

    fn widget(&self) -> gtk::Widget {
        self.frame.clone().upcast::<gtk::Widget>()
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        self.frame.destroy();
    }
}

pub struct MessagesHandler {
    /// Our css provider.
    css_provider: gtk::CssProvider,
    /// Container where our message widegts will live.
    container: gtk::Box,

    messages: Vec<Message>,

    font: Font,
}

impl MessagesHandler {
    pub fn new(parent: &gtk::Overlay) -> Self {
        let css_provider = gtk::CssProvider::new();

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_valign(gtk::Align::Start);

        parent.add_overlay(&container);
        parent.set_overlay_pass_through(&container, true);

        // Work around some intial draw issues.
        container.show_all();
        container.hide();

        MessagesHandler {
            css_provider,
            container,
            messages: vec!(),
            font: Font::default(),
        }
    }

    pub fn show(&mut self, msg: &MsgShow, hl_defs: &HlDefs) {

        if msg.replace_last {
            self.messages.pop();
        }

        let msg = Message::new(msg, hl_defs, &self.css_provider, self.font.height as f64);
        self.container.pack_end(&msg.widget(), false, true, 0);
        self.messages.push(msg);

        self.container.show_all();
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.container.hide();
    }

    pub fn set_colors(&self, hl_defs: &HlDefs) {
        let css = format!(
            "box {{
                margin: 5px;
                min-width: 430px;
                background-color: #{bg};
                box-shadow: 0px 5px 5px 0px rgba(0, 0, 0, 0.75);
                border: 1px solid #{fg};
            }}

            image {{
                padding: 10px;
            }}

            label {{
                padding: 10px;
            }}

            {font_wild}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Point),
            bg = hl_defs.default_bg.to_hex(),
            fg = hl_defs.default_fg.to_hex(),
        );

        gtk::CssProvider::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    pub fn set_font(&mut self, font: Font, hl_defs: &HlDefs) {
        self.font = font;
        self.set_colors(hl_defs);
    }
}

fn get_icon_pixbuf(
    kind: &MsgShowKind,
    color: &Color,
    size: f64,
) -> gdk_pixbuf::Pixbuf {
    let contents = get_icon_name_for_kind(kind, &color, size);
    let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
        contents.as_bytes(),
    ));
    let buf = gdk_pixbuf::Pixbuf::new_from_stream(&stream, None::<&gio::Cancellable>).unwrap();

    buf
}

fn get_icon_name_for_kind(kind: &MsgShowKind, color: &Color, size: f64) -> String {
    let color = color.to_hex();

    let size = size * 1.5;

    match kind {
        MsgShowKind::Unknown => icon!("../../assets/icons/help-circle.svg", color, size),
        MsgShowKind::Confirm | MsgShowKind::ConfirmSub => icon!("../../assets/icons/check-square.svg", color, size),
        MsgShowKind::Emsg |
        MsgShowKind::EchoErr => icon!("../../assets/icons/x-octagon.svg", color, size),
        MsgShowKind::Echo |
        MsgShowKind::EchoMsg => icon!("../../assets/icons/message-circle.svg", color, size),
        MsgShowKind::Wmsg => icon!("../../assets/icons/alert-octagon.svg", color, size),
        MsgShowKind::QuickFix => icon!("../../assets/icons/zap.svg", color, size),
        MsgShowKind::ReturnPrompt => icon!("../../assets/icons/info.svg", color, size),
    }
}
