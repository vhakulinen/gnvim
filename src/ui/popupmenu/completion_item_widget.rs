use gtk;
use gtk::prelude::*;

use nvim_bridge::CompletionItem;
use ui::color::Color;

macro_rules! icon {
    ($file:expr, $color:expr, $size:expr) => {
        format!(include_str!($file), $size, $size, $color,)
    };
}

/// Wraps completion item into a structure which contains the item and some
/// of the widgets to display it.
pub struct CompletionItemWidgetWrap {
    /// Actual completion item.
    pub item: CompletionItem,
    /// Label displaying `info` for this item in the list.
    pub info: gtk::Label,
    /// Label displaying `menu` for this item in the list.
    pub menu: gtk::Label,
    /// Image of the item in the row.
    pub kind: gtk::Image,
    /// Root container.
    pub row: gtk::ListBoxRow,
}

impl CompletionItemWidgetWrap {
    pub fn create(
        item: CompletionItem,
        css_provider: &gtk::CssProvider,
        icon_fg: &Color,
        size: f64,
    ) -> Self {
        let margin = (size / 3.0) as i32;

        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);

        let menu = gtk::Label::new(item.menu.as_str());
        menu.set_halign(gtk::Align::End);
        menu.set_hexpand(true);
        menu.set_margin_end(margin);
        menu.set_ellipsize(pango::EllipsizeMode::End);
        grid.attach(&menu, 2, 0, 1, 1);

        let word = gtk::Label::new(item.word.as_str());
        word.set_ellipsize(pango::EllipsizeMode::End);
        grid.attach(&word, 1, 0, 1, 1);

        let info = gtk::Label::new(shorten_info(&item.info).as_str());
        info.set_halign(gtk::Align::Start);
        info.set_ellipsize(pango::EllipsizeMode::End);

        info.connect_realize(|info| {
            info.hide();
        });
        menu.connect_realize(|menu| {
            menu.hide();
        });

        grid.attach(&info, 1, 1, 2, 1);

        // NOTE(ville): We only need to explicitly create this row widget
        //              so we can set css provider to it.
        let row = gtk::ListBoxRow::new();
        row.add(&grid);

        let buf = get_icon_pixbuf(&item.kind.as_str(), icon_fg, size);
        match buf {
            Ok(buff) => {
                let kind = gtk::Image::new_from_pixbuf(&buff);
                kind.set_tooltip_text(format!("kind: '{}'", item.kind).as_str());
                kind.set_margin_start(margin);
                grid.attach(&kind, 0, 0, 1, 1);
                add_css_provider!(css_provider, grid, word, kind, info, row, menu);
                return CompletionItemWidgetWrap {
                    item,
                    info,
                    row,
                    kind,
                    menu,
                };
            },
            Err(_) => {
                add_css_provider!(css_provider, grid, word, info, row, menu);
                return CompletionItemWidgetWrap {
                    item,
                    info,
                    row,
                    kind: gtk::Image::new(),
                    menu,
                }
            },
        };
    }
}

/// Returns first line of `info`.
fn shorten_info(info: &String) -> String {
    let lines = info.split("\n").collect::<Vec<&str>>();
    let first_line = lines.get(0).unwrap();
    first_line.to_string()
}

pub fn get_icon_pixbuf(
    kind: &str,
    color: &Color,
    size: f64,
) -> Result<gdk_pixbuf::Pixbuf, gdk_pixbuf::Error> {
    let contents = get_icon_name_for_kind(kind, &color, size);
    let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
        contents.as_bytes(),
    ));
    let buf = gdk_pixbuf::Pixbuf::new_from_stream(&stream, None);

    buf
}

// pub fn get_icon_pixbuf(
//     kind: &str,
//     color: &Color,
//     size: f64,
// ) -> gdk_pixbuf::Pixbuf {
//     let contents = get_icon_name_for_kind(kind, &color, size);
//     let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
//         contents.as_bytes(),
//     ));
//     let buf = gdk_pixbuf::Pixbuf::new_from_stream(&stream, None).unwrap();
// 
//     buf
// }

fn get_icon_name_for_kind(kind: &str, color: &Color, size: f64) -> String {
    let color = color.to_hex();

    let size = size * 1.1;

    match kind {
        "method" | "function" | "constructor" => {
            icon!("../../../assets/icons/box.svg", color, size)
        }
        "field" => {
            icon!("../../../assets/icons/chevrons-right.svg", color, size)
        }
        "event" => icon!("../../../assets/icons/zap.svg", color, size),
        "operator" => icon!("../../../assets/icons/sliders.svg", color, size),
        "variable" => icon!("../../../assets/icons/disc.svg", color, size),
        "class" => icon!("../../../assets/icons/share-2.svg", color, size),
        "interface" => {
            icon!("../../../assets/icons/book-open.svg", color, size)
        }
        "struct" => icon!("../../../assets/icons/align-left.svg", color, size),
        "type parameter" => {
            icon!("../../../assets/icons/type.svg", color, size)
        }
        "module" => icon!("../../../assets/icons/code.svg", color, size),
        "property" => icon!("../../../assets/icons/key.svg", color, size),
        "unit" => icon!("../../../assets/icons/compass.svg", color, size),
        "constant" => icon!("../../../assets/icons/shield.svg", color, size),
        "value" | "enum" => {
            icon!("../../../assets/icons/database.svg", color, size)
        }
        "enum member" => icon!("../../../assets/icons/tag.svg", color, size),
        "keyword" => icon!("../../../assets/icons/link-2.svg", color, size),
        "text" => icon!("../../../assets/icons/at-sign.svg", color, size),
        "color" => icon!("../../../assets/icons/aperture.svg", color, size),
        "file" => icon!("../../../assets/icons/file.svg", color, size),
        "reference" => icon!("../../../assets/icons/link.svg", color, size),
        "snippet" => icon!("../../../assets/icons/file-text.svg", color, size),
        "folder" => icon!("../../../assets/icons/folder.svg", color, size),

       // _ => icon!("../../../assets/icons/help-circle.svg", color, size),
        _ => String::from("None"),
    }
}
