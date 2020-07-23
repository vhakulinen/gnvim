use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use gtk::prelude::*;

use webkit2gtk as webkit;
use webkit2gtk::{SettingsExt, WebViewExt};

use pulldown_cmark as md;

use syntect::dumps::from_binary;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::thread_guard::ThreadGuard;
use crate::ui::color::Color;
use crate::ui::common::{
    get_preferred_horizontal_position, get_preferred_vertical_position,
};
use crate::ui::font::{Font, FontUnit};

pub enum Gravity {
    Up,
    Down,
}

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

const MAX_WIDTH: i32 = 700;
const MAX_HEIGHT: i32 = 300;

struct State {
    anchor: gdk::Rectangle,
    available_area: gdk::Rectangle,
    force_gravity: Option<Gravity>,
    scale: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            anchor: gdk::Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            available_area: gdk::Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            force_gravity: None,
            scale: 1.0,
        }
    }
}

/// Cursor tooltip to display markdown documents on given grid position.
/// Internally uses `syntect` to do code highlighting.
pub struct CursorTooltip {
    css_provider: gtk::CssProvider,
    frame: gtk::Frame,
    fixed: gtk::Fixed,
    webview: webkit::WebView,
    state: Arc<ThreadGuard<State>>,

    fg: Color,
    bg: Color,
    font: Font,

    /// Our current syntax set.
    syntax_set: SyntaxSet,
    /// Our current theme set.
    theme_set: ThemeSet,

    /// Currently selected theme.
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

        let state = Arc::new(ThreadGuard::new(State::default()));

        let frame_weak = frame.downgrade();
        let fixed_weak = fixed.downgrade();
        webview.connect_load_changed(
            clone!(frame_weak, fixed_weak, state => move |webview, e| {
                if let webkit::LoadEvent::Finished = e {
                    webview_load_finished(
                        webview,
                        frame_weak.clone(),
                        fixed_weak.clone(),
                        state.clone(),
                    );
                }
            }),
        );

        let settings = WebViewExt::get_settings(&webview).unwrap();
        settings.set_enable_javascript(true);

        parent.add_overlay(&fixed);
        parent.set_overlay_pass_through(&fixed, true);

        fixed.show_all();

        fixed.connect_size_allocate(
            clone!(state, webview => move |fixed, alloc| {
                let mut state = state.borrow_mut();
                let ctx = fixed.get_pango_context().unwrap();
                let res = pangocairo::functions::context_get_resolution(&ctx);
                state.scale = res / 96.0; // 96.0 picked from GTK's own source code.
                webview.set_zoom_level(state.scale);

                state.available_area = *alloc;
            }),
        );

        let syntax_set: SyntaxSet =
            from_binary(include_bytes!("../../sublime-syntaxes/all.pack"));
        let theme_set = ThemeSet::load_defaults();

        let current_theme = theme_set.themes["base16-ocean.dark"].clone();

        CursorTooltip {
            css_provider,
            frame,
            fixed,
            webview,
            state,

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

    /// Get list of available code highlighting styles.
    pub fn get_styles(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }

    /// Set the current code highlighting style.
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

    pub fn is_visible(&self) -> bool {
        self.frame.is_visible()
    }

    pub fn load_style(&mut self, path: String) -> Result<(), &str> {
        let path = Path::new(&path);
        let theme =
            ThemeSet::get_theme(&path).or(Err("Failed to load theme file"))?;

        let name = if let Some(name) = theme.clone().name {
            name
        } else {
            return Err("Failed to get theme name");
        };
        self.theme_set.themes.insert(name, theme);

        Ok(())
    }

    /// Parse markdown parser events into a form where we have syntax highlighting.
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
                                .unwrap_or_else(|| {
                                    self.syntax_set.find_syntax_plain_text()
                                })
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
        let mut state = self.state.borrow_mut();
        state.anchor = *rect;
    }

    /// Forces the gravity of the tooltip to be above or below of current
    /// anchor position.
    pub fn force_gravity(&mut self, gravity: Option<Gravity>) {
        let mut state = self.state.borrow_mut();
        state.force_gravity = gravity;
    }

    /// Refreshes the position of the tooltip element.
    pub fn refresh_position(&self) {
        let alloc = self.frame.get_allocation();
        let state = self.state.borrow_mut();

        set_position(
            &self.frame,
            &self.fixed,
            &state,
            alloc.width,
            alloc.height,
        );
    }
}

/// Ensures the correct `frame` position and size inside `fixed`.
fn set_position(
    frame: &gtk::Frame,
    fixed: &gtk::Fixed,
    state: &State,
    width: i32,
    height: i32,
) {
    let mut available_area = state.available_area;

    match state.force_gravity {
        Some(Gravity::Up) => {
            available_area.height = state.anchor.y;
        }
        Some(Gravity::Down) => {
            available_area.y = state.anchor.y + state.anchor.height;
        }
        _ => {}
    }

    let (x, width) = get_preferred_horizontal_position(
        &available_area,
        &state.anchor,
        width,
    );
    let (y, height) =
        get_preferred_vertical_position(&available_area, &state.anchor, height);

    fixed.move_(frame, x, y);

    frame.set_size_request(width, height);
}

/// Once the webview has loaded its content, we need to check how much
/// height and width does the rendered content take. After this, we can set
/// the size of the webview's container.
fn webview_load_finished(
    webview: &webkit::WebView,
    frame: glib::WeakRef<gtk::Frame>,
    fixed: glib::WeakRef<gtk::Fixed>,
    state: Arc<ThreadGuard<State>>,
) {
    let widgets = ThreadGuard::new((frame, fixed, state.clone()));

    let cb =
        move |width: Option<f64>,
              res: Result<webkit::JavascriptResult, webkit::Error>| {
            let res = res.unwrap();
            let height = match (res.get_value(), res.get_global_context()) {
                (Some(val), Some(ctx)) => val.to_number(&ctx),
                _ => None,
            };

            let widgets = widgets.borrow();
            let state = state.borrow();
            // NOTE(ville): Extra height coming from GTK styles
            //              (parent container's border).
            let extra_height = 2;
            let height = height
                .map_or(MAX_HEIGHT, |v| (v * state.scale) as i32 + extra_height)
                .min(MAX_HEIGHT);
            let width = width
                .map_or(MAX_WIDTH, |v| (v * state.scale) as i32)
                .min(MAX_WIDTH);

            let frame_weak = &widgets.0;
            let fixed_weak = &widgets.1;
            let frame = upgrade_weak!(frame_weak);
            let fixed = upgrade_weak!(fixed_weak);

            frame.show();

            set_position(&frame, &fixed, &state, width, height);
        };

    let webview_ref = ThreadGuard::new(webview.clone());
    webview.run_javascript("
        let el = document.getElementById('wrapper');
        el.style.width = '-webkit-max-content';
        let width = el.getBoundingClientRect().width;
        el.style.width = '';
        // Add some extra (16) to adjust for padding.
        width + 16",
        None::<&gio::Cancellable>,
        move |res: Result<webkit::JavascriptResult, webkit::Error>| {

            let res = res.unwrap();
            let width = match (res.get_value(), res.get_global_context()) {
                (Some(val), Some(ctx)) => val.to_number(&ctx),
                _ => None,
            };

            webview_ref.borrow().run_javascript(
                "document.getElementById('wrapper').getBoundingClientRect().height",
                None::<&gio::Cancellable>,
                move |res| {
                    cb(width, res);
                },
            );
        },
    );
}

/// Filters some HTML element attributes. Only allows `style` attribute
/// for `span` element, with allowed CSS styles that are outputted by
/// `syntect` HTML renderer.
fn attribute_filter<'u>(
    element: &str,
    attribute: &str,
    value: &'u str,
) -> Option<Cow<'u, str>> {
    match (element, attribute) {
        ("span", "style") => {
            // Allowed CSS properties (other than colors).
            let mut allowed_fixed = HashMap::new();
            allowed_fixed.insert("text-decorator", ["underline"]);
            allowed_fixed.insert("font-weight", ["bold"]);
            allowed_fixed.insert("font-style", ["italic"]);

            // Allowed (color) CSS properties.
            let allowed_color = ["color", "background-color"];

            let mut new = String::new();

            for attrs in value.split(';') {
                if let [prop, val] = attrs.split(':').collect::<Vec<&str>>()[..]
                {
                    if let Some(vals) = allowed_fixed.get(&prop) {
                        if vals.contains(&val) {
                            new.push_str(&prop);
                            new.push_str(":");
                            new.push_str(val);
                            new.push_str(";");
                        }
                    } else if allowed_color.contains(&prop) {
                        // Some tinfoil hat action going on with the colors.
                        // Parse the colors "properly" so we know that we
                        // have a valid color value.
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
