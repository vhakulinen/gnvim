use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use nvim::types::uievents::{DefaultColorsSet, HlGroupSet};
use nvim::types::UiEvent;
use nvim::types::{ModeInfo, OptionSet, UiOptions};

use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{
    gdk,
    glib::{self, clone},
};

use gio_compat::CompatRead;
use nvim::rpc::RpcReader;

use crate::colors::{Color, Colors, HlGroup};
use crate::components::shell::Shell;
use crate::font::Font;
use crate::nvim::Neovim;
use crate::{arguments::BoxedArguments, spawn_local, SCALE};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/application.ui")]
pub struct AppWindow {
    im_context: gtk::IMMulticontext,
    event_controller_key: gtk::EventControllerKey,
    #[template_child(id = "shell")]
    shell: TemplateChild<Shell>,

    css_provider: gtk::CssProvider,

    args: RefCell<BoxedArguments>,
    nvim: Neovim,

    colors: Rc<RefCell<Colors>>,
    font: RefCell<Font>,
    mode_infos: RefCell<Vec<ModeInfo>>,

    /// Source id for debouncing nvim resizing.
    resize_id: Rc<Cell<Option<glib::SourceId>>>,
    /// When resize on flush is set, there were some operations on the previous
    /// ui events that changed our grid size (e.g. font chagned etc.).
    resize_on_flush: Cell<bool>,
    /// Set when attributes affecting our CSS changed, and we need to regenerate
    /// the css.
    css_on_flush: Cell<bool>,
    /// Our previous window size. Used to track when we need to tell neovim to
    /// resize itself.
    prev_win_size: Cell<(i32, i32)>,
}

impl AppWindow {
    async fn io_loop(&self, obj: super::AppWindow, reader: CompatRead) {
        use nvim::rpc::{message::Notification, Message};
        let mut reader: RpcReader<CompatRead> = reader.into();

        loop {
            let msg = reader.recv().await.unwrap();
            match msg {
                Message::Response(res) => {
                    self.nvim
                        .client()
                        .await
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
                                .for_each(|event| self.handle_ui_event(&obj, event))
                        }
                        _ => {
                            println!("Unexpected notification: {}", method);
                        }
                    }
                }
            }
        }
    }

    fn handle_hl_group_set(&self, event: HlGroupSet) {
        if let Some(group) = match event.name.as_ref() {
            "MsgSeparator" => Some(HlGroup::MsgSeparator),
            "Pmenu" => Some(HlGroup::Pmenu),
            "PmenuSel" => Some(HlGroup::PmenuSel),
            "PmenuSbar" => Some(HlGroup::PmenuSbar),
            "PmenuThumb" => Some(HlGroup::PmenuThumb),
            _ => None,
        } {
            self.colors.borrow_mut().set_hl_group(group, event.id);
            self.css_on_flush.set(true);
        }
    }

    fn handle_default_colors_set(&self, event: DefaultColorsSet) {
        let mut colors = self.colors.borrow_mut();
        colors.fg = Color::from_i64(event.rgb_fg);
        colors.bg = Color::from_i64(event.rgb_bg);
        colors.sp = Color::from_i64(event.rgb_sp);

        self.css_on_flush.set(true);
    }

    fn handle_ui_event(&self, obj: &super::AppWindow, event: UiEvent) {
        match event {
            // Global events
            UiEvent::SetTitle(events) => events.into_iter().for_each(|event| {
                obj.set_title(Some(&event.title));
            }),
            UiEvent::SetIcon(_) => {}
            UiEvent::ModeInfoSet(events) => events.into_iter().for_each(|event| {
                self.mode_infos.replace(event.cursor_styles);
            }),
            UiEvent::OptionSet(events) => events.into_iter().for_each(|event| {
                self.handle_option_set(obj, event);
            }),
            UiEvent::ModeChange(events) => events.into_iter().for_each(|event| {
                let modes = self.mode_infos.borrow();
                let mode = modes
                    .get(event.mode_idx as usize)
                    .expect("invalid mode_idx");
                self.shell.handle_mode_change(mode);
            }),
            UiEvent::MouseOn => {}
            UiEvent::MouseOff => {}
            UiEvent::BusyStart => {
                self.shell.busy_start();
            }
            UiEvent::BusyStop => {
                self.shell.busy_stop();
            }
            UiEvent::Suspend => {}
            UiEvent::UpdateMenu => {}
            UiEvent::Bell => {}
            UiEvent::VisualBell => {}
            UiEvent::Flush => {
                self.shell.handle_flush(&self.colors.borrow());

                if self.resize_on_flush.take() {
                    self.resize_nvim();
                }

                if self.css_on_flush.take() {
                    let colors = self.colors.borrow();
                    let linespace = self.font.borrow().linespace() / SCALE;
                    let pmenu = colors.get_hl_group(&HlGroup::Pmenu);
                    let pmenu_sel = colors.get_hl_group(&HlGroup::PmenuSel);
                    let pmenu_thumb = colors.get_hl_group(&HlGroup::PmenuThumb);
                    let pmenu_bar = colors.get_hl_group(&HlGroup::PmenuSbar);
                    let msgsep = colors.get_hl_group(&HlGroup::MsgSeparator);
                    // TODO(ville): It might be possible to make the font
                    // be set in CSS, instead of through custom property.
                    // Tho' at least linespace value (e.g. line-height css
                    // property) was added as recently as gtk version 4.6.
                    self.css_provider.load_from_data(
                        format!(
                            r#"
                                .app-window, .external-window {{
                                    background-color: #{bg};
                                }}

                                .msg-win.scrolled {{
                                    border-top: 1px solid #{msgsep};
                                }}

                                .popupmenu-listview,
                                .popupmenu-row {{
                                    color: #{pmenu_fg};
                                    background-color: #{pmenu_bg};

                                    padding-top: {linespace_top}px;
                                    padding-bottom: {linespace_bottom}px;
                                }}

                                .popupmenu-listview > :selected,
                                .popupmenu-listview > :selected > .popupmenu-row {{
                                    color: #{pmenu_sel_fg};
                                    background-color: #{pmenu_sel_bg};
                                }}

                                .popupmenu scrollbar {{
                                    background-color: #{pmenusbar_bg};
                                }}

                                .popupmenu slider {{
                                    background-color: #{pmenuthumb_bg};
                                    border-color: #{pmenuthumb_bg};
                                }}
                            "#,
                            bg = colors.bg.as_hex(),
                            msgsep = msgsep.fg().as_hex(),
                            pmenu_fg = pmenu.fg().as_hex(),
                            pmenu_bg = pmenu.bg().as_hex(),
                            pmenu_sel_fg = pmenu_sel.fg().as_hex(),
                            pmenu_sel_bg = pmenu_sel.bg().as_hex(),
                            pmenusbar_bg = pmenu_bar.bg().as_hex(),
                            pmenuthumb_bg = pmenu_thumb.bg().as_hex(),
                            linespace_top = (linespace / 2.0).ceil().max(0.0),
                            linespace_bottom = (linespace / 2.0).floor().max(0.0),
                        )
                        .as_bytes(),
                    );
                }
            }

            // linegrid events
            UiEvent::GridResize(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_resize(event);
            }),
            UiEvent::DefaultColorsSet(events) => events
                .into_iter()
                .for_each(|event| self.handle_default_colors_set(event)),
            UiEvent::HlAttrDefine(events) => events.into_iter().for_each(|event| {
                let mut colors = self.colors.borrow_mut();
                colors.hls.insert(event.id, event.rgb_attrs.into());
            }),
            UiEvent::HlGroupSet(events) => events.into_iter().for_each(|event| {
                self.handle_hl_group_set(event);
            }),
            UiEvent::GridLine(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_line(event);
            }),
            UiEvent::GridClear(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_clear(event);
            }),
            UiEvent::GridDestroy(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_grid_destroy(event)),
            UiEvent::GridCursorGoto(events) => events.into_iter().for_each(|event| {
                self.shell.handle_grid_cursor_goto(event);
            }),
            UiEvent::GridScroll(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_grid_scroll(event)),

            // multigrid events
            UiEvent::WinPos(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_win_pos(event, &self.font.borrow())),
            UiEvent::WinFloatPos(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_float_pos(event, &self.font.borrow())),
            UiEvent::WinExternalPos(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_win_external_pos(event, obj.upcast_ref())),
            UiEvent::WinHide(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_win_hide(event)),
            UiEvent::WinClose(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_win_close(event)),
            UiEvent::MsgSetPos(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_msg_set_pos(event, &self.font.borrow())),
            // TODO(ville): Scrollbars?
            UiEvent::WinViewport(_) => {}

            // popupmenu events
            UiEvent::PopupmenuShow(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_popupmenu_show(event)),
            UiEvent::PopupmenuSelect(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_popupmenu_select(event)),
            UiEvent::PopupmenuHide => self.shell.handle_popupmenu_hide(),

            event => panic!("Unhandled ui event: {}", event),
        }
    }

    fn handle_option_set(&self, obj: &super::AppWindow, event: OptionSet) {
        match event {
            OptionSet::Linespace(linespace) => {
                let font = Font::new(&self.font.borrow().guifont(), linespace as f32);
                obj.set_property("font", &font);

                self.resize_on_flush.set(true);
                self.css_on_flush.set(true);
            }
            OptionSet::Guifont(guifont) => {
                let font = Font::new(&guifont, self.font.borrow().linespace() / SCALE);
                obj.set_property("font", &font);

                self.resize_on_flush.set(true);
                self.css_on_flush.set(true);
            }
            OptionSet::Unknown(_) => {}
        }
    }

    fn resize_nvim(&self) {
        let (cols, rows) = self
            .font
            .borrow()
            .grid_size_for_allocation(&self.shell.allocation());

        let id = glib::timeout_add_local(
            Duration::from_millis(crate::WINDOW_RESIZE_DEBOUNCE_MS),
            clone!(@weak self.nvim as nvim, @weak self.resize_id as resize_id => @default-return Continue(false), move || {
                spawn_local!(clone!(@weak nvim => async move {
                    let res = nvim
                        .client()
                        .await
                        .nvim_ui_try_resize_grid(1, cols.max(1) as i64, rows.max(1) as i64)
                        .await
                        .unwrap();

                    res.await.expect("nvim_ui_try_resize failed");
                }));

                // Clear after our selves, so we don't try to remove
                // our id once we're already done.
                resize_id.replace(None);

                Continue(false)
            }),
        );

        // Cancel the earlier timeout if it exists.
        if let Some(id) = self.resize_id.replace(Some(id)).take() {
            id.remove();
        }
    }

    fn send_nvim_input(&self, input: String) {
        spawn_local!(clone!(@weak self.nvim as nvim => async move {
            let res = nvim
                .client()
                .await
                .nvim_input(input)
                .await
                .expect("call to nvim failed");

            // TODO(ville): nvim_input handle the returned bytes written value.
            res.await.expect("nvim_input failed");
        }));
    }

    fn im_commit(&self, input: &str) {
        // NOTE(ville): "<" needs to be escaped for nvim_input (see `:h nvim_input`)
        let input = input.replace('<', "<lt>");
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

        // If the input is a modifier only event, ignore it.
        if evt
            .downcast_ref::<gdk::KeyEvent>()
            .map(|evt| evt.is_modifier())
            .unwrap_or(false)
        {
            return gtk::Inhibit(false);
        }

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

        let args = self.args.borrow();
        let reader = self
            .nvim
            .open(args.nvim.as_ref(), &args.files, &args.nvim_args);

        // Start io loop.
        spawn_local!(clone!(@strong obj as app => async move {
            app.imp().io_loop(app.clone(), reader).await;
        }));

        // Call nvim_ui_attach.
        spawn_local!(clone!(@weak self.nvim as nvim => async move {
            let res = nvim
                .client()
                .await
                .nvim_ui_attach(80, 30, UiOptions {
                    rgb: true,
                    ext_linegrid: true,
                    ext_multigrid: true,
                    ext_popupmenu: true,
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

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::new(
                    "font",
                    "font",
                    "Font",
                    Font::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecObject::new(
                    "nvim",
                    "nvim",
                    "Neovim client",
                    Neovim::static_type(),
                    glib::ParamFlags::READABLE,
                ),
                glib::ParamSpecBoxed::new(
                    "args",
                    "args",
                    "Arguments",
                    BoxedArguments::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "font" => self.font.borrow().to_value(),
            "nvim" => self.nvim.to_value(),
            "args" => self.args.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be object Font"));
            }
            "args" => {
                self.args.replace(
                    value
                        .get()
                        .expect("font value must be object BoxedArguments"),
                );
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for AppWindow {
    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        let prev = self.prev_win_size.get();
        // TODO(ville): Check for rows/col instead.
        // NOTE(ville): If we try to resize nvim unconditionally, we'll
        // end up in a infinite loop.
        if prev != (width, height) {
            self.prev_win_size.set((width, height));
            self.resize_nvim();
        }
    }
}

impl WindowImpl for AppWindow {}

impl ApplicationWindowImpl for AppWindow {}

fn event_to_nvim_input(keyval: gdk::Key, state: gdk::ModifierType) -> Option<String> {
    let mut input = crate::input::modifier_to_nvim(&state);
    let keyname = keyval.name()?;

    if keyname.chars().count() > 1 {
        let n = crate::input::keyname_to_nvim_key(keyname.as_str())?;
        input.push_str(n);
    } else {
        input.push(keyval.to_unicode()?);
    }

    Some(format!("<{}>", input))
}
