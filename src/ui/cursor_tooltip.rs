use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use webkit2gtk as webkit;
use webkit2gtk::{SettingsExt, UserContentManagerExt, WebViewExt};

use ammonia;
use pulldown_cmark as md;

use syntect::dumps::from_binary;
use syntect::highlighting::{Color as SyntectColor, Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::Scope;
use syntect::parsing::SyntaxSet;

use thread_guard::ThreadGuard;
use ui::color::Color;
use ui::font::{Font, FontUnit};

lazy_static! {
    /// Our custom ammonia builder to clean untrusted HTML.
    static ref AMMONIA: ammonia::Builder<'static> = {
        let mut attrs = HashMap::new();
        let mut set = HashSet::new();
        set.insert("style");
        attrs.insert("span", set);

        let mut builder = ammonia::Builder::default();
        builder.tag_attributes(attrs);
        builder.attribute_filter(attribute_filter);

        builder
    };
}

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

    syntax_set: SyntaxSet,
    theme_set: ThemeSet,

    current_theme: Theme,
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
        parent.set_overlay_pass_through(&fixed, true);

        fixed.show_all();

        let available_area_ref = available_area.clone();
        fixed.connect_size_allocate(move |_, alloc| {
            let mut a = available_area_ref.borrow_mut();
            *a = alloc.clone();
        });

        let syntax_set: SyntaxSet =
            from_binary(include_bytes!("../../sublime-syntaxes/all.pack"));
        let theme_set = ThemeSet::load_defaults();

        let current_theme = theme_set.themes["base16-ocean.dark"].clone();

        CursorTooltip {
            css_provider,
            frame,
            webview,
            position,

            fg: Color::default(),
            bg: Color::default(),
            font: Font::default(),

            syntax_set,
            theme_set,
            current_theme,
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

    pub fn get_styles(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }

    pub fn set_style(&mut self, style: &str) {
        if let Some(theme) = self.theme_set.themes.get(style) {
            self.current_theme = theme.clone();
        }
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    pub fn hide(&self) {
        self.frame.hide();
    }

    fn parse_events<'a>(&self, parser: md::Parser<'a>) -> Vec<md::Event<'a>> {
        let mut syntax = self.syntax_set.find_syntax_plain_text();

        let mut events = Vec::new();
        let mut to_highlight = String::new();
        let mut in_code_block = false;

        for event in parser {
            match event {
                md::Event::Start(md::Tag::CodeBlock(lang)) => {
                    syntax = self
                        .syntax_set
                        // Try to find the syntax by token.
                        .find_syntax_by_token(&lang)
                        .unwrap_or({
                            // If its not found, try more relaxed way of finding it.
                            self.syntax_set
                                .syntaxes()
                                .iter()
                                .rev()
                                .find(|&syntax| {
                                    syntax
                                        .name
                                        .to_lowercase()
                                        .contains(&lang.to_string())
                                })
                                // And if not still found, use the plain text one.
                                .unwrap_or(
                                    self.syntax_set.find_syntax_plain_text(),
                                )
                        });

                    in_code_block = true;
                }
                md::Event::End(md::Tag::CodeBlock(_)) => {
                    if in_code_block {
                        let html = syntect::html::highlighted_html_for_string(
                            &to_highlight,
                            &self.syntax_set,
                            &syntax,
                            &self.current_theme,
                        );
                        events.push(md::Event::Html(Cow::Owned(html)));
                    }
                    in_code_block = false;

                    to_highlight.clear();
                }
                md::Event::Text(text) => {
                    if in_code_block {
                        to_highlight.push_str(&text);
                    } else {
                        events.push(md::Event::Text(text));
                    }
                }
                e => {
                    events.push(e);
                }
            }
        }

        events
    }

    pub fn show(&mut self, content: String) {
        self.webview.get_user_content_manager().unwrap();

        // Parse the content (that should be markdown document).
        let mut opts = md::Options::empty();
        opts.insert(md::Options::ENABLE_TABLES);
        let parser = md::Parser::new_ext(&content, opts);

        // And parse the parser events so that we have highlighting for code blocks.
        let events = self.parse_events(parser);

        // And turn the markdown events into HTML.
        let mut parsed = String::new();
        md::html::push_html(&mut parsed, events.into_iter());

        // Finally, clean up the html (e.g. remove any javascript and such).
        let html = AMMONIA.clean(&parsed).to_string();

        let all = format!(
            "<!DOCTYPE html>
            <html> 
            <head>
                <meta charset=\"utf8\">
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
            </body>
        </html>",
            content = html,
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
        webview.clone(),
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

                let (x, width) =
                    get_preferred_horizontal_position(&area, &pos, MAX_WIDTH);
                let (y, height) =
                    get_preferred_vertical_position(&area, &pos, height);

                widgets.1.move_(&widgets.0, x, y);

                widgets.0.show();
                widgets.0.set_size_request(width, height);
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

fn attribute_filter<'u>(
    element: &str,
    attribute: &str,
    value: &'u str,
) -> Option<Cow<'u, str>> {
    match (element, attribute) {
        ("span", "style") => {
            let mut allowed_fixed = HashMap::new();
            allowed_fixed.insert("text-decorator", ["underline"]);
            allowed_fixed.insert("font-weight", ["bold"]);
            allowed_fixed.insert("font-style", ["italic"]);
            let allowed_color = ["color", "background-color"];

            let mut new = String::new();

            for attrs in value.split(";") {
                if let [prop, val] = attrs.split(":").collect::<Vec<&str>>()[..]
                {
                    if let Some(vals) = allowed_fixed.get(&prop) {
                        if vals.contains(&val) {
                            new.push_str(&prop);
                            new.push_str(":");
                            new.push_str(val);
                            new.push_str(";");
                        }
                    } else if allowed_color.contains(&prop) {
                        if let Ok(color) =
                            Color::from_hex_string(val.to_string())
                        {
                            new.push_str(&prop);
                            new.push_str(":#");
                            new.push_str(&color.to_hex());
                            new.push_str(";");
                        }
                    }
                }
            }

            Some(new.into())
        }
        _ => None,
    }
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
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 10,
        };

        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 10);
    }

    #[test]
    fn test_get_preferred_horizontal_position2() {
        // Case 2: Width is trucated.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 5,
        };

        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 5);
    }

    #[test]
    fn test_get_preferred_horizontal_position3() {
        // Case 3: X is moved to left.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle {
            x: 15,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 15;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 5);
        assert_eq!(w, 15);
    }

    #[test]
    fn test_get_preferred_horizontal_position4() {
        // Case 4: X is moved to left and width is truncated
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle {
            x: 15,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 150;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 20);
    }
}
