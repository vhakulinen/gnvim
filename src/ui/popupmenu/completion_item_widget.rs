use gtk;
use gtk::prelude::*;

use nvim_bridge::{CompletionItem, CompletionItemKind};
use ui::color::Color;

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
    pub image: gtk::Image,
    /// Kind of the item
    pub kind: CompletionItemKind,
    /// Root container.
    pub row: gtk::ListBoxRow,
}

impl CompletionItemWidgetWrap {
    pub fn create(
        item: CompletionItem,
        show_kind: bool,
        css_provider: &gtk::CssProvider,
        icon_fg: &Color,
        size: f64,
    ) -> Self {
        let margin = (size / 3.0) as i32;

        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);

        let image = gtk::Image::new();
        if show_kind {
            let buf = get_icon_pixbuf(&item.kind, icon_fg, size);
            image.set_from_pixbuf(Some(&buf));
            image.set_tooltip_text(Some(
                format!("kind: '{}'", item.kind_raw).as_str(),
            ));
            image.set_margin_start(margin);
            grid.attach(&image, 0, 0, 1, 1);
        }

        let menu = gtk::Label::new(Some(item.menu.as_str()));
        menu.set_halign(gtk::Align::End);
        menu.set_hexpand(true);
        menu.set_margin_end(margin);
        menu.set_ellipsize(pango::EllipsizeMode::End);
        grid.attach(&menu, 2, 0, 1, 1);

        let word = gtk::Label::new(Some(item.word.as_str()));
        word.set_ellipsize(pango::EllipsizeMode::End);
        grid.attach(&word, 1, 0, 1, 1);

        let info = gtk::Label::new(Some(shorten_info(&item.info).as_str()));
        info.set_halign(gtk::Align::Start);
        info.set_ellipsize(pango::EllipsizeMode::End);

        if !show_kind {
            word.set_margin_start(5);
            info.set_margin_start(5);
        }

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

        add_css_provider!(css_provider, grid, word, image, info, row, menu);

        let kind = item.kind.clone();
        CompletionItemWidgetWrap {
            item,
            info,
            row,
            image,
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

pub fn get_icon_pixbuf(
    kind: &CompletionItemKind,
    color: &Color,
    size: f64,
) -> gdk_pixbuf::Pixbuf {
    let contents = get_icon_name_for_kind(kind, &color, size);
    let stream = gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from(
        contents.as_bytes(),
    ));
    let buf =
        gdk_pixbuf::Pixbuf::new_from_stream(&stream, None::<&gio::Cancellable>)
            .unwrap();

    buf
}

fn get_icon_name_for_kind(
    kind: &CompletionItemKind,
    color: &Color,
    size: f64,
) -> String {
    let color = color.to_hex();

    let size = size * 1.1;

    use self::CompletionItemKind::*;
    match kind {
        Constructor => icon!("../../../assets/icons/box.svg", color, size),
        Method => icon!("../../../assets/icons/box.svg", color, size),
        Function => icon!("../../../assets/icons/box.svg", color, size),
        Field => icon!("../../../assets/icons/chevrons-right.svg", color, size),
        Event => icon!("../../../assets/icons/zap.svg", color, size),
        Operator => icon!("../../../assets/icons/sliders.svg", color, size),
        Variable => icon!("../../../assets/icons/disc.svg", color, size),
        Class => icon!("../../../assets/icons/share-2.svg", color, size),
        Interface => icon!("../../../assets/icons/book-open.svg", color, size),
        Struct => icon!("../../../assets/icons/align-left.svg", color, size),
        TypeParameter => icon!("../../../assets/icons/type.svg", color, size),
        Module => icon!("../../../assets/icons/code.svg", color, size),
        Property => icon!("../../../assets/icons/key.svg", color, size),
        Unit => icon!("../../../assets/icons/compass.svg", color, size),
        Constant => icon!("../../../assets/icons/shield.svg", color, size),
        Value => icon!("../../../assets/icons/database.svg", color, size),
        Enum => icon!("../../../assets/icons/database.svg", color, size),
        EnumMember => icon!("../../../assets/icons/tag.svg", color, size),
        Keyword => icon!("../../../assets/icons/link-2.svg", color, size),
        Text => icon!("../../../assets/icons/at-sign.svg", color, size),
        Color => icon!("../../../assets/icons/aperture.svg", color, size),
        File => icon!("../../../assets/icons/file.svg", color, size),
        Reference => icon!("../../../assets/icons/link.svg", color, size),
        Snippet => icon!("../../../assets/icons/file-text.svg", color, size),
        Folder => icon!("../../../assets/icons/folder.svg", color, size),

        _ => icon!("../../../assets/icons/help-circle.svg", color, size),
    }
}
