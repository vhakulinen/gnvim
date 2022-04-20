use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use futures::lock::Mutex;
use nvim::types::uievents::UiOptions;
use nvim::types::UiEvent;

use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{
    gdk, gio,
    glib::{self, clone},
};

use gio_compat::CompatRead;
use gio_compat::CompatWrite;
use nvim::rpc::RpcReader;

use crate::colors::{Color, Colors};
use crate::components::shell::Shell;
use crate::font::Font;
use crate::{nvim_unlock, spawn_local};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/application.ui")]
pub struct AppWindow {
    im_context: gtk::IMMulticontext,
    event_controller_key: gtk::EventControllerKey,
    #[template_child(id = "shell")]
    shell: TemplateChild<Shell>,

    css_provider: gtk::CssProvider,

    nvim: Rc<Mutex<Option<nvim::Client<CompatWrite>>>>,

    colors: Rc<RefCell<Colors>>,
    font: Rc<RefCell<Font>>,

    /// Source id for debouncing nvim resizing.
    resize_id: Rc<Cell<Option<glib::SourceId>>>,
}

impl AppWindow {
    fn open_nvim(&self) -> (nvim::Client<CompatWrite>, CompatRead) {
        let mut flags = gio::SubprocessFlags::empty();
        flags.insert(gio::SubprocessFlags::STDIN_PIPE);
        flags.insert(gio::SubprocessFlags::STDOUT_PIPE);

        let p = gio::Subprocess::newv(
            &[
                std::ffi::OsStr::new("nvim"),
                std::ffi::OsStr::new("--embed"),
            ],
            flags,
        )
        .expect("failed to open nvim subprocess");

        let writer: CompatWrite = p
            .stdin_pipe()
            .expect("get stdin pipe")
            .dynamic_cast::<gio::PollableOutputStream>()
            .expect("cast to PollableOutputStream")
            .into_async_write()
            .expect("convert to async write")
            .into();

        let reader: CompatRead = p
            .stdout_pipe()
            .expect("get stdout pipe")
            .dynamic_cast::<gio::PollableInputStream>()
            .expect("cast to PollableInputStream")
            .into_async_read()
            .expect("covert to async read")
            .into();

        (nvim::Client::new(writer), reader)
    }

    async fn io_loop(&self, reader: CompatRead) {
        use nvim::rpc::{message::Notification, Message};
        let mut reader: RpcReader<CompatRead> = reader.into();

        loop {
            let msg = reader.recv().await.unwrap();
            match msg {
                Message::Response(res) => {
                    self.nvim
                        .lock()
                        .await
                        .as_mut()
                        .expect("nvim not set")
                        .handle_response(res)
                        .expect("failed to handle nvim response");
                }
                Message::Request(req) => {
                    println!("Got request from nvim: {:?}", req);
                }
                Message::Notification(Notification { method, params, .. }) => {
                    match method.as_ref() {
                        "redraw" => {
                            let events = nvim::decode_redraw_params(params)
                                .expect("failed to decode redraw notification");

                            events
                                .into_iter()
                                .for_each(|event| self.handle_ui_event(event))
                        }
                        _ => {
                            println!("Unexpected notification: {}", method);
                        }
                    }
                }
            }
        }
    }

    fn handle_ui_event(&self, event: UiEvent) {
        match event {
            UiEvent::OptionSet(_) => {}
            UiEvent::DefaultColorsSet(events) => events.into_iter().for_each(|event| {
                let mut colors = self.colors.borrow_mut();
                colors.fg = Color::from_i64(event.rgb_fg);
                colors.bg = Color::from_i64(event.rgb_bg);
                colors.sp = Color::from_i64(event.rgb_sp);

                self.css_provider.load_from_data(
                    format!(
                        r#"
                            .app-window {{
                                background-color: #{bg};
                            }}
                        "#,
                        bg = colors.bg.as_hex(),
                    )
                    .as_bytes(),
                );
            }),
            UiEvent::HlAttrDefine(events) => events.into_iter().for_each(|event| {
                let mut colors = self.colors.borrow_mut();
                colors.hls.insert(event.id, event.rgb_attrs);
            }),
            UiEvent::HlGroupSet(_) => {}
            UiEvent::GridResize(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_resize(event);
            }),
            UiEvent::GridClear(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_clear(event);
            }),
            UiEvent::GridLine(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_line(event);
            }),
            UiEvent::GridCursorGoto(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_cursor_goto(
                    event,
                    &self.font.borrow(),
                    &self.colors.borrow(),
                );
            }),
            UiEvent::GridScroll(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_grid_scroll(event)),
            UiEvent::UpdateMenu => {}
            UiEvent::WinViewport(_) => {}
            UiEvent::ModeInfoSet(_) => {}
            UiEvent::ModeChange(_) => {}
            UiEvent::Flush => {
                self.shell
                    .handle_flush(&self.colors.borrow(), &self.font.borrow());
            }
            UiEvent::SetIcon(_) => {}
            UiEvent::SetTitle(_) => {}
            UiEvent::MouseOn => {}
            UiEvent::MouseOff => {}
            event => panic!("Unhandled ui event: {}", event),
        }
    }

    fn send_nvim_input(&self, input: String) {
        spawn_local!(clone!(@weak self.nvim as nvim => async move {
            let res = nvim_unlock!(nvim)
                .nvim_input(input)
                .await
                .expect("call to nvim failed");

            // TODO(ville): nvim_input handle the returned bytes written value.
            res.await.expect("nvim_input failed");
        }));
    }

    fn im_commit(&self, input: &str) {
        // NOTE(ville): "<" needs to be escaped for nvim_input (see `:h nvim_input`)
        let input = input.replace("<", "<lt>");
        self.send_nvim_input(input);
    }

    fn key_pressed(
        &self,
        eck: &gtk::EventControllerKey,
        keyval: gdk::Key,
        _keycode: u32,
        state: gdk::ModifierType,
    ) -> gtk::Inhibit {
        let evt = eck.current_event().expect("failed to get event");
        if self.im_context.filter_keypress(&evt) {
            gtk::Inhibit(true)
        } else {
            if let Some(input) = event_to_nvim_input(keyval, state) {
                self.send_nvim_input(input);
                return gtk::Inhibit(true);
            } else {
                println!(
                    "Failed to turn input event into nvim key (keyval: {})",
                    keyval,
                )
            }

            gtk::Inhibit(false)
        }
    }

    fn key_released(&self, eck: &gtk::EventControllerKey) {
        let evt = eck.current_event().expect("failed to get event");
        self.im_context.filter_keypress(&evt);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AppWindow {
    const NAME: &'static str = "AppWindow";
    type Type = super::AppWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Shell::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for AppWindow {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().expect("couldn't get display"),
            &self.css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let (client, reader) = self.open_nvim();

        *self.nvim.try_lock().expect("can't get lock") = Some(client);

        // Start io loop.
        spawn_local!(clone!(@strong obj as app => async move {
            app.imp().io_loop(reader).await;
        }));

        // Call nvim_ui_attach.
        spawn_local!(clone!(@weak self.nvim as nvim => async move {
            let res = nvim_unlock!(nvim).nvim_ui_attach(80, 30,
                UiOptions{
                    rgb: true,
                    ext_linegrid: true,
                    //ext_multigrid: true,
                    ..Default::default()
                }
            ).await.expect("call to nvim failed");

            res.await.expect("nvim_ui_attach failed");
        }));

        // TODO(ville): Figure out if we should use preedit or not.
        self.im_context.set_use_preedit(false);
        self.event_controller_key
            .set_im_context(Some(&self.im_context));

        self.im_context
            .connect_commit(clone!(@weak obj => move |_, input| {
                obj.imp().im_commit(input)
            }));

        self.event_controller_key.connect_key_pressed(clone!(
        @weak obj,
        => @default-return gtk::Inhibit(false),
        move |eck, keyval, keycode, state| {
            obj.imp().key_pressed(eck, keyval, keycode, state)
        }));

        self.event_controller_key
            .connect_key_released(clone!(@weak obj => move |eck, _, _, _| {
                obj.imp().key_released(eck)
            }));

        obj.add_controller(&self.event_controller_key);
    }
}

impl WidgetImpl for AppWindow {
    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        let (cols, rows) = self
            .font
            .borrow()
            .grid_size_for_allocation(&self.shell.allocation());

        let id = glib::timeout_add_local(
            Duration::from_millis(10),
            clone!(@weak self.nvim as nvim, @weak self.resize_id as resize_id => @default-return Continue(false), move || {
                spawn_local!(clone!(@weak nvim => async move {
                    let res = nvim_unlock!(nvim)
                        .nvim_ui_try_resize(cols as i64, rows as i64)
                        .await
                        .unwrap();

                    res.await.expect("nvim_ui_try_resize failed");
                }));

                // Clear after our selves, so we don't try to remove
                // our id once we're already done.
                resize_id.replace(None);

                return Continue(false)
            }),
        );

        // Cancel the earlier timeout if it exists.
        if let Some(id) = self.resize_id.replace(Some(id)).take() {
            id.remove();
        }
    }
}

impl WindowImpl for AppWindow {}

impl ApplicationWindowImpl for AppWindow {}

fn keyname_to_nvim_key(s: &str) -> Option<&str> {
    // Originally sourced from python-gui.
    match s {
        "asciicircum" => Some("^"), // fix #137
        "slash" => Some("/"),
        "backslash" => Some("\\"),
        "dead_circumflex" => Some("^"),
        "at" => Some("@"),
        "numbersign" => Some("#"),
        "dollar" => Some("$"),
        "percent" => Some("%"),
        "ampersand" => Some("&"),
        "asterisk" => Some("*"),
        "parenleft" => Some("("),
        "parenright" => Some(")"),
        "underscore" => Some("_"),
        "plus" => Some("+"),
        "minus" => Some("-"),
        "bracketleft" => Some("["),
        "bracketright" => Some("]"),
        "braceleft" => Some("{"),
        "braceright" => Some("}"),
        "dead_diaeresis" => Some("\""),
        "dead_acute" => Some("\'"),
        "less" => Some("<"),
        "greater" => Some(">"),
        "comma" => Some(","),
        "period" => Some("."),
        "space" => Some("Space"),
        "BackSpace" => Some("BS"),
        "Insert" => Some("Insert"),
        "Return" => Some("CR"),
        "Escape" => Some("Esc"),
        "Delete" => Some("Del"),
        "Page_Up" => Some("PageUp"),
        "Page_Down" => Some("PageDown"),
        "Enter" => Some("CR"),
        "ISO_Left_Tab" => Some("Tab"),
        "Tab" => Some("Tab"),
        "Up" => Some("Up"),
        "Down" => Some("Down"),
        "Left" => Some("Left"),
        "Right" => Some("Right"),
        "Home" => Some("Home"),
        "End" => Some("End"),
        "F1" => Some("F1"),
        "F2" => Some("F2"),
        "F3" => Some("F3"),
        "F4" => Some("F4"),
        "F5" => Some("F5"),
        "F6" => Some("F6"),
        "F7" => Some("F7"),
        "F8" => Some("F8"),
        "F9" => Some("F9"),
        "F10" => Some("F10"),
        "F11" => Some("F11"),
        "F12" => Some("F12"),
        _ => None,
    }
}

fn event_to_nvim_input(keyval: gdk::Key, state: gdk::ModifierType) -> Option<String> {
    let mut input = String::from("");

    let keyname = keyval.name()?;

    if state.contains(gdk::ModifierType::SHIFT_MASK) {
        input.push_str("S-");
    }
    if state.contains(gdk::ModifierType::CONTROL_MASK) {
        input.push_str("C-");
    }
    if state.contains(gdk::ModifierType::ALT_MASK) {
        input.push_str("A-");
    }

    if keyname.chars().count() > 1 {
        let n = keyname_to_nvim_key(keyname.as_str())?;
        input.push_str(n);
    } else {
        input.push(keyval.to_unicode()?);
    }

    Some(format!("<{}>", input))
}
