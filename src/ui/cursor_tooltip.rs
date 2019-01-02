use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use webkit2gtk as webkit;
use webkit2gtk::SettingsExt;
use webkit2gtk::WebViewExt;

//use pulldown_cmark::{Parser, html};
use ammonia;
use pulldown_cmark as md;

use thread_guard::ThreadGuard;
use ui::color::Color;
use ui::font::{Font, FontUnit};

const MAX_WIDTH: i32 = 600;
const MAX_HEIGHT: i32 = 300;

pub struct CursorTooltip {
    css_provider: gtk::CssProvider,

    frame: gtk::Frame,
    webview: webkit::WebView,

    position: Rc<RefCell<gdk::Rectangle>>,

    fg: Color,
    bg: Color,

    font: Font,
}

impl CursorTooltip {
    pub fn new(parent: &gtk::Overlay) -> Self {
        let css_provider = gtk::CssProvider::new();

        let context = webkit::WebContext::get_default().unwrap();
        let webview = webkit::WebView::new_with_context(&context);

        let frame = gtk::Frame::new(None);
        frame.add(&webview);

        add_css_provider!(&css_provider, frame);

        let fixed = gtk::Fixed::new();
        fixed.put(&frame, 0, 0);

        let position = Rc::new(RefCell::new(gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }));
        let available_area = Rc::new(RefCell::new(gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }));

        let frame_ref = frame.clone();
        let fixed_ref = fixed.clone();
        let position_ref = position.clone();
        let available_area_ref = available_area.clone();
        webview.connect_load_changed(move |webview, e| match e {
            webkit::LoadEvent::Finished => {
                webview_load_finished(
                    webview,
                    frame_ref.clone(),
                    fixed_ref.clone(),
                    position_ref.clone(),
                    available_area_ref.clone(),
                );
            }
            _ => {}
        });

        let settings = WebViewExt::get_settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);
        settings.set_enable_javascript(true);

        parent.add_overlay(&fixed);

        fixed.show_all();

        let available_area_ref = available_area.clone();
        fixed.connect_size_allocate(move |_, alloc| {
            let mut a = available_area_ref.borrow_mut();
            *a = alloc.clone();
        });

        CursorTooltip {
            css_provider,
            frame,
            webview,

            position,

            fg: Color::default(),
            bg: Color::default(),
            font: Font::default(),
        }
    }

    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;

        let css = format!(
            "* {{
            border: 1px solid #{fg};
            border-radius: 0;
        }}",
            fg = fg.to_hex()
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    pub fn hide(&self) {
        self.frame.hide();
    }

    pub fn show(&self, content: String) {
        let js_path = "./runtime/web-resources/highlight.pack.js";
        let css_path = "./runtime/web-resources/styles/nord.css";

        let js = fs::read_to_string(js_path).unwrap();
        let css = fs::read_to_string(css_path).unwrap();

        let parser = md::Parser::new(&content);
        let mut target = String::new();
        md::html::push_html(&mut target, parser);

        let clean = ammonia::clean(&target);

        let all = format!(
            "<!DOCTYPE html>
            <html> 
            <head>
                <meta charset=\"utf8\">
                <script>{js}</script>
                <style>{css}</style>
                <style>
                    * {{
                        color: #{fg};
                        background-color: #{bg};
                        word-wrap: break-word;
                    }}

                    #wrapper {{
                        height: 100%;
                        padding: 8px;
                    }}

                    #content *:first-child {{
                        margin-top: 0px;
                    }}

                    #content *:last-child {{
                        margin-bottom: 0px;
                    }}

                    #content pre:first-child code {{
                        padding: 0px !important;
                    }}

                    body {{
                        margin: 0px;
                        padding: 0px;
                    }}

                    {font}
                </style>
            </head>
            <body>
                <div id=\"wrapper\">
                    <div id=\"content\">
                        {content}
                    </div>
                </div>
                <script>hljs.initHighlightingOnLoad();</script>
            </body>
        </html>",
            js = js,
            css = css,
            content = clean,
            fg = self.fg.to_hex(),
            bg = self.bg.to_hex(),
            font = self.font.as_wild_css(FontUnit::Point)
        );

        self.webview.load_html(&all, None);
    }

    pub fn move_to(&mut self, rect: &gdk::Rectangle) {
        let mut pos = self.position.borrow_mut();
        *pos = rect.clone();
    }
}

fn webview_load_finished(
    webview: &webkit::WebView,
    frame: gtk::Frame,
    fixed: gtk::Fixed,
    position: Rc<RefCell<gdk::Rectangle>>,
    available_area: Rc<RefCell<gdk::Rectangle>>,
) {
    let widgets = ThreadGuard::new((
        frame.clone(),
        fixed.clone(),
        position.clone(),
        available_area.clone(),
    ));

    let cb = move |res: Result<webkit::JavascriptResult, webkit::Error>| {
        if !res.is_ok() {
            return;
        }

        let res = res.unwrap();
        if let (Some(val), Some(ctx)) =
            (res.get_value(), res.get_global_context())
        {
            if let Some(height) = val.to_number(&ctx) {
                let widgets = widgets.borrow();
                // NOTE(ville): Extra height coming from GTK styles
                //              (parent container's border).
                let extra_height = 2;
                let mut height = (height as i32 + extra_height).min(MAX_HEIGHT);

                let pos = widgets.2.borrow();
                let area = widgets.3.borrow();

                let (x, width) = get_preferred_horizontal_position(&area, &pos, MAX_WIDTH);
                let (y, height) =
                    get_preferred_vertical_position(&area, &pos, height);

                widgets.1.move_(&widgets.0, x, y);

                widgets.0.set_size_request(width, height);
                widgets.0.show();
            }
        }
    };

    webview.run_javascript(
        "document.getElementById('wrapper').getBoundingClientRect().height",
        None,
        cb,
    );
}

fn get_preferred_horizontal_position(
    area: &gdk::Rectangle,
    pos: &gdk::Rectangle,
    mut width: i32,
) -> (i32, i32) {
    let mut x = pos.x;

    let rigth = x + width;
    // If we're overflowing to the right...
    if rigth > area.width {
        let overflow = rigth - area.width;
        // Move our x position to the left, but not father that 0.
        x = (x - overflow).max(0);

        // And set our width to be either the original width, or truncate
        // it to area.width it happens to be smaller (otherwise we'd still
        // overflow).
        width = width.min(area.width);
    }

    (x, width)
}

fn get_preferred_vertical_position(
    area: &gdk::Rectangle,
    pos: &gdk::Rectangle,
    mut height: i32,
) -> (i32, i32) {
    let mut y = pos.y - height;

    if y < area.y {
        let max_above = area.y + pos.y;
        let max_below = area.height - (pos.y + pos.height);

        if max_above > max_below {
            y = area.y;
            height = max_above;
        } else {
            y = pos.y + pos.height;
            height = height.min(max_below);
        }
    }

    return (y, height);
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_preferred_vertical_position1() {
        // Case 1: there is room just fine in the obvious position.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 300,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 30,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 0);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_vertical_position2() {
        // Case 2: there is no room above the `pos`, so we should position our
        // selves below the pos.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 300,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 15);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_vertical_position3() {
        // Case 3: there is no room above the `pos`, so we should position our
        // selves below the pos but in this case, we need to truncate our height too.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 35,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 15);
        assert_eq!(h, 20);
    }

    #[test]
    fn test_get_preferred_vertical_position4() {
        // Case 4: there is no room above the `pos`, but below it there is even less
        // space. We should go above, but truncate our height.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 40,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 30,
            width: 300,
            height: 50,
        };
        let height = 80;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 0);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_horizontal_position1() {
        // Case 1: Everything fits.
        let area = gdk::Rectangle{
            x: 0,
            y: 0,
            height: 0,
            width: 10,
        };

        let pos = gdk::Rectangle{
            x: 0, y: 0, width: 0, height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 10);
    }

    #[test]
    fn test_get_preferred_horizontal_position2() {
        // Case 2: Width is trucated.
        let area = gdk::Rectangle{
            x: 0,
            y: 0,
            height: 0,
            width: 5,
        };

        let pos = gdk::Rectangle{
            x: 0, y: 0, width: 0, height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 5);
    }

    #[test]
    fn test_get_preferred_horizontal_position3() {
        // Case 3: X is moved to left.
        let area = gdk::Rectangle{
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle{
            x: 15, y: 0, width: 0, height: 0,
        };

        let width = 15;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 5);
        assert_eq!(w, 15);
    }

    #[test]
    fn test_get_preferred_horizontal_position4() {
        // Case 4: X is moved to left and width is truncated
        let area = gdk::Rectangle{
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle{
            x: 15, y: 0, width: 0, height: 0,
        };

        let width = 150;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 20);
    }
}
