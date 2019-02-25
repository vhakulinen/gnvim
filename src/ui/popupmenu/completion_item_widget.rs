
use gtk;
use gtk::prelude::*;

use nvim_bridge::CompletionItem;
use ui::color::Color;

macro_rules! icon {
    ($file:expr, $color:expr) => {
        format!(include_str!($file), $color,)
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
    ) -> Self {
        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);

        let buf = get_icon_pixbuf(&item.kind.as_str(), icon_fg);
        let kind = gtk::Image::new_from_pixbuf(&buf);
        kind.set_tooltip_text(format!("kind: '{}'", item.kind).as_str());

        kind.set_halign(gtk::Align::Start);
        kind.set_margin_start(5);
        kind.set_margin_end(5);
        grid.attach(&kind, 0, 0, 1, 1);

        let menu = gtk::Label::new(item.menu.as_str());
        menu.set_halign(gtk::Align::End);
        menu.set_hexpand(true);
        menu.set_margin_start(5);
        menu.set_margin_end(5);
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

        add_css_provider!(css_provider, grid, kind, word, info, row, menu);

        CompletionItemWidgetWrap {
            item,
            info,
            row,
            kind,
            menu,
        }
    }
}

/// Returns first line of `info`.
fn shorten_info(info: &String) -> String {
    let lines = info.split("\n").collect::<Vec<&str>>();
    let first_line = lines.get(0).unwrap();
    first_line.to_string()
}

pub fn get_icon_pixbuf(kind: &str, color: &Color) -> gdk_pixbuf::Pixbuf {
    let contents = get_icon_name_for_kind(kind, &color);
    let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
        contents.as_bytes(),
    ));
    let buf = gdk_pixbuf::Pixbuf::new_from_stream(&stream, None).unwrap();

    buf
}

fn get_icon_name_for_kind(kind: &str, color: &Color) -> String {
    let color = color.to_hex();

    match kind {
        "method" | "function" | "constructor" => {
            icon!("../../../assets/icons/box.svg", color)
        }
        "field" => icon!("../../../assets/icons/chevrons-right.svg", color),
        "event" => icon!("../../../assets/icons/zap.svg", color),
        "operator" => icon!("../../../assets/icons/sliders.svg", color),
        "variable" => icon!("../../../assets/icons/disc.svg", color),
        "class" => icon!("../../../assets/icons/share-2.svg", color),
        "interface" => icon!("../../../assets/icons/book-open.svg", color),
        "struct" => icon!("../../../assets/icons/align-left.svg", color),
        "type parameter" => icon!("../../../assets/icons/type.svg", color),
        "module" => icon!("../../../assets/icons/code.svg", color),
        "property" => icon!("../../../assets/icons/key.svg", color),
        "unit" => icon!("../../../assets/icons/compass.svg", color),
        "constant" => icon!("../../../assets/icons/shield.svg", color),
        "value" | "enum" => icon!("../../../assets/icons/database.svg", color),
        "enum member" => icon!("../../../assets/icons/tag.svg", color),
        "keyword" => icon!("../../../assets/icons/link-2.svg", color),
        "text" => icon!("../../../assets/icons/at-sign.svg", color),
        "color" => icon!("../../../assets/icons/aperture.svg", color),
        "file" => icon!("../../../assets/icons/file.svg", color),
        "reference" => icon!("../../../assets/icons/link.svg", color),
        "snippet" => icon!("../../../assets/icons/file-text.svg", color),
        "folder" => icon!("../../../assets/icons/folder.svg", color),

        _ => icon!("../../../assets/icons/help-circle.svg", color),
    }
}
