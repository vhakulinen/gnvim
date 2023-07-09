use std::cell::{Cell, RefCell};
use std::ffi::OsStr;

use nvim::dict;
use nvim::rpc::Message;
use nvim::serde::Deserialize;
use nvim::types::uievents::{
    DefaultColorsSet, HlGroupSet, MsgHistoryShow, MsgShow, PopupmenuSelect, PopupmenuShow,
};
use nvim::types::UiEvent;
use nvim::types::{OptionSet, UiOptions};
use nvim::NeovimApi;

use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{
    gdk,
    glib::{self, clone},
};

use nvim::rpc::{message::Notification, RpcReader};

use crate::api::GnvimEvent;
use crate::boxed::{ModeInfo, ShowTabline};
use crate::colors::{Color, Colors, HlGroup};
use crate::components::{Messages, Omnibar, Overflower, Shell, Tabline};
use crate::font::Font;
use crate::nvim::Neovim;
use crate::warn;
use crate::{arguments::Arguments, spawn_local, SCALE};

#[derive(CompositeTemplate, Default, glib::Properties)]
#[properties(wrapper_type = super::AppWindow)]
#[template(resource = "/com/github/vhakulinen/gnvim/application.ui")]
pub struct AppWindow {
    #[property(get, set)]
    im_context: RefCell<gtk::IMMulticontext>,
    #[property(get, set)]
    event_controller_key: RefCell<gtk::EventControllerKey>,
    #[template_child(id = "shell")]
    shell: TemplateChild<Shell>,
    #[template_child(id = "tabline")]
    tabline: TemplateChild<Tabline>,
    #[template_child(id = "omnibar")]
    omnibar: TemplateChild<Omnibar>,
    #[template_child(id = "messages")]
    messages: TemplateChild<Messages>,

    css_provider: gtk::CssProvider,

    #[property(get, set, construct_only)]
    args: RefCell<Arguments>,
    #[property(get)]
    nvim: Neovim,

    colors: RefCell<Colors>,

    #[property(get, set)]
    font: RefCell<Font>,
    mode_infos: RefCell<Vec<ModeInfo>>,
    #[property(get, set)]
    show_tabline: RefCell<ShowTabline>,

    /// When resize on flush is set, there were some operations on the previous
    /// ui events that changed our grid size (e.g. font chagned etc.).
    resize_on_flush: Cell<bool>,
    /// Set when attributes affecting our CSS changed, and we need to regenerate
    /// the css.
    css_on_flush: Cell<bool>,
    /// Idle callback source id for scrolling the message view after msg_show
    /// event,
    scroll_messages_source_id: Cell<Option<glib::SourceId>>,
}

impl AppWindow {
    fn process_nvim_event(&self, msg: Message) {
        match msg {
            Message::Response(res) => {
                self.nvim
                    .handle_response(res)
                    .expect("failed to handle nvim response");
            }
            Message::Request(req) => {
                println!("Got request from nvim: {:?}", req);
            }
            Message::Notification(Notification { method, params, .. }) => match method.as_ref() {
                "redraw" => {
                    let events = nvim::decode_redraw_params(params)
                        .expect("failed to decode redraw notification");

                    events
                        .into_iter()
                        .for_each(|event| self.handle_ui_event(event))
                }
                "gnvim" => match params {
                    rmpv::Value::Array(params) => params
                        .into_iter()
                        .map(GnvimEvent::deserialize)
                        .for_each(|res| match res {
                            Ok(event) => self.handle_gnvim_event(event),
                            Err(err) => warn!("failed to parse gnvim event: {:?}", err),
                        }),
                    params => warn!("unexpected gnvim params: {:?}", params),
                },
                _ => {
                    println!("Unexpected notification: {}", method);
                }
            },
        }
    }

    async fn io_loop<R: futures::AsyncRead + Unpin>(&self, reader: R) {
        let mut reader: RpcReader<R> = reader.into();

        loop {
            match reader.recv().await {
                Ok(msg) => self.process_nvim_event(msg),
                Err(_) => {
                    self.obj()
                        .application()
                        .expect("application not set")
                        .quit();
                    break;
                }
            };
        }
    }

    fn handle_hl_group_set(&self, event: HlGroupSet) {
        if let Some(group) = match event.name.as_ref() {
            "MsgSeparator" => Some(HlGroup::MsgSeparator),
            "Pmenu" => Some(HlGroup::Pmenu),
            "PmenuSel" => Some(HlGroup::PmenuSel),
            "PmenuSbar" => Some(HlGroup::PmenuSbar),
            "PmenuThumb" => Some(HlGroup::PmenuThumb),
            "TabLine" => Some(HlGroup::TabLine),
            "TabLineFill" => Some(HlGroup::TabLineFill),
            "TabLineSel" => Some(HlGroup::TabLineSel),
            "Menu" => Some(HlGroup::Menu),
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

    fn handle_popupmenu_show(&self, event: PopupmenuShow) {
        if event.grid == -1 {
            self.omnibar.handle_popupmenu_show(event)
        } else {
            self.shell.handle_popupmenu_show(event)
        }
    }

    fn handle_popupmenu_select(&self, event: PopupmenuSelect) {
        if self.omnibar.cmdline_popupmenu_visible() {
            self.omnibar.handle_popupmenu_select(event)
        } else {
            self.shell.handle_popupmenu_select(event)
        }
    }

    fn handle_popupmenu_hide(&self) {
        if self.omnibar.cmdline_popupmenu_visible() {
            self.omnibar.handle_popupmenu_hide()
        } else {
            self.shell.handle_popupmenu_hide()
        }
    }

    fn handle_gnvim_event(&self, event: GnvimEvent) {
        match event {
            GnvimEvent::EchoRepeat(echo_repeat) => {
                let msg = vec![
                    rmpv::Value::from(vec![rmpv::Value::from(echo_repeat.msg)]);
                    echo_repeat.times
                ];

                spawn_local!(clone!(@weak self.nvim as nvim => async move {
                    let res = nvim
                        .nvim_echo(msg, false, &dict![])
                        .await
                        .unwrap();

                    res.await.expect("nvim_echo failed");
                }));
            }
            GnvimEvent::GtkDebugger => {
                self.enable_debugging(true);
            }
            GnvimEvent::CursorBlinkTransition(t) => {
                self.shell.set_cursor_blink_transition(t);
            }
            GnvimEvent::CursorPositionTransition(t) => {
                self.shell.set_cursor_position_transition(t);
            }
            GnvimEvent::ScrollTransition(t) => {
                self.shell.set_scroll_transition(t);
            }
            GnvimEvent::MessageKinds(event) => {
                self.messages.set_kinds(event.kinds);
            }
        }
    }

    fn handle_ui_event(&self, event: UiEvent) {
        match event {
            // Global events
            UiEvent::SetTitle(events) => events.into_iter().for_each(|event| {
                self.obj().set_title(Some(&event.title));
            }),
            UiEvent::SetIcon(_) => {}
            UiEvent::ModeInfoSet(events) => events.into_iter().for_each(|event| {
                self.mode_infos
                    .replace(event.cursor_styles.into_iter().map(Into::into).collect());
            }),
            UiEvent::OptionSet(events) => events.into_iter().for_each(|event| {
                self.handle_option_set(event);
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
                self.tabline.flush();

                if self.resize_on_flush.take() {
                    self.shell.resize_nvim();
                }

                if self.css_on_flush.take() {
                    let colors = self.colors.borrow();
                    let linespace = self.font.borrow().linespace() / SCALE;
                    let pmenu = colors.get_hl_group(&HlGroup::Pmenu);
                    let pmenu_sel = colors.get_hl_group(&HlGroup::PmenuSel);
                    let pmenu_thumb = colors.get_hl_group(&HlGroup::PmenuThumb);
                    let pmenu_bar = colors.get_hl_group(&HlGroup::PmenuSbar);
                    let msgsep = colors.get_hl_group(&HlGroup::MsgSeparator);
                    let tablinefill = colors.get_hl_group(&HlGroup::TabLineFill);
                    let tabline = colors.get_hl_group(&HlGroup::TabLine);
                    let tablinesel = colors.get_hl_group(&HlGroup::TabLineSel);
                    // TODO(ville): Figure out better headerbar colors.
                    let menu = colors.get_hl_group(&HlGroup::Menu);
                    // TODO(ville): It might be possible to make the font
                    // be set in CSS, instead of through custom property.
                    // Tho' at least linespace value (e.g. line-height css
                    // property) was added as recently as gtk version 4.6.
                    self.css_provider.load_from_data(&format!(
                        include_str!("style.css"),
                        bg = colors.bg.as_hex(),
                        fg = colors.fg.as_hex(),
                        msgsep = msgsep.fg().as_hex(),
                        pmenu_fg = pmenu.fg().as_hex(),
                        pmenu_bg = pmenu.bg().as_hex(),
                        pmenu_sel_fg = pmenu_sel.fg().as_hex(),
                        pmenu_sel_bg = pmenu_sel.bg().as_hex(),
                        pmenusbar_bg = pmenu_bar.bg().as_hex(),
                        pmenuthumb_bg = pmenu_thumb.bg().as_hex(),
                        tabline_bg = tabline.bg().as_hex(),
                        tabline_fg = tabline.fg().as_hex(),
                        tablinefill_bg = tablinefill.bg().as_hex(),
                        tablinesel_bg = tablinesel.bg().as_hex(),
                        tablinesel_fg = tablinesel.fg().as_hex(),
                        linespace_top = (linespace / 2.0).ceil().max(0.0),
                        linespace_bottom = (linespace / 2.0).floor().max(0.0),
                        menu_bg = menu.bg().as_hex(),
                        menu_fg = menu.fg().as_hex(),
                        omnibar_pad = 5,
                        font = self.font.borrow().to_css(),
                    ));
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
            UiEvent::WinExternalPos(events) => events.into_iter().for_each(|event| {
                self.shell
                    .handle_win_external_pos(event, self.obj().upcast_ref())
            }),
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
            UiEvent::WinViewport(events) => events
                .into_iter()
                .for_each(|event| self.shell.handle_win_viewport(event)),

            // popupmenu events
            UiEvent::PopupmenuShow(events) => events
                .into_iter()
                .for_each(|event| self.handle_popupmenu_show(event)),
            UiEvent::PopupmenuSelect(events) => events
                .into_iter()
                .for_each(|event| self.handle_popupmenu_select(event)),
            UiEvent::PopupmenuHide => self.handle_popupmenu_hide(),

            // tabline events
            UiEvent::TablineUpdate(events) => events
                .into_iter()
                .for_each(|event| self.tabline.handle_tabline_update(event)),

            // cmdline events
            UiEvent::CmdlineShow(events) => events.into_iter().for_each(|event| {
                self.omnibar
                    .handle_cmdline_show(event, &self.colors.borrow())
            }),
            UiEvent::CmdlineHide(events) => events
                .into_iter()
                .for_each(|event| self.omnibar.handle_cmdline_hide(event)),
            UiEvent::CmdlinePos(events) => events
                .into_iter()
                .for_each(|event| self.omnibar.handle_cmdline_pos(event)),
            UiEvent::CmdlineSpecialChar(events) => events
                .into_iter()
                .for_each(|event| self.omnibar.handle_cmdline_special_char(event)),
            UiEvent::CmdlineBlockShow(events) => events.into_iter().for_each(|event| {
                self.omnibar
                    .handle_cmdline_block_show(event, &self.colors.borrow())
            }),
            UiEvent::CmdlineBlockHide => self.omnibar.handle_cmdline_block_hide(),
            UiEvent::CmdlineBlockAppend(events) => events.into_iter().for_each(|event| {
                self.omnibar
                    .handle_cmdline_block_append(event, &self.colors.borrow())
            }),

            UiEvent::MsgShow(events) => self.handle_message_show(events),
            UiEvent::MsgClear => self.messages.handle_message_clear(),
            UiEvent::MsgShowmode(_events) => {
                // Noop.
            }
            UiEvent::MsgShowcmd(_events) => {
                // Noop.
            }
            UiEvent::MsgRuler(_events) => {
                // Noop.
            }
            UiEvent::MsgHistoryShow(events) => self.handle_message_history_show(events),
            UiEvent::MsgHistoryClear => {
                self.messages.handle_message_history_clear();
            }

            event => panic!("Unhandled ui event: {}", event),
        }
    }

    /// Scroll the messages to the bottom on next idle callback, giving any
    /// pending widget changes a chance to take place.
    fn scroll_messages_to_bottom(&self) {
        let id = glib::idle_add_local_once(clone!(@weak self as this => move || {
            this.messages.scroll_to_bottom();

            // Remove our source id.
            this.scroll_messages_source_id.take();
        }));

        if let Some(prev) = self.scroll_messages_source_id.replace(Some(id)) {
            prev.remove();
        }
    }

    fn handle_message_history_show(&self, events: Vec<MsgHistoryShow>) {
        let colors = self.colors.borrow();
        events.into_iter().for_each(|event| {
            self.messages.handle_message_history_show(event, &colors);
        });

        self.scroll_messages_to_bottom();
    }

    fn handle_message_show(&self, events: Vec<MsgShow>) {
        let colors = self.colors.borrow();
        self.messages.handle_message_show(events, &colors);
        self.scroll_messages_to_bottom();
    }

    fn handle_option_set(&self, event: OptionSet) {
        match event {
            OptionSet::Linespace(linespace) => {
                let font = Font::new(&self.font.borrow().guifont(), linespace as f32);
                self.obj().set_property("font", &font);

                self.resize_on_flush.set(true);
                self.css_on_flush.set(true);

                self.omnibar.set_cmdline_linespace(linespace as f32);
            }
            OptionSet::Guifont(guifont) => {
                let font = Font::new(&guifont, self.font.borrow().linespace() / SCALE);
                self.obj().set_property("font", &font);

                self.resize_on_flush.set(true);
                self.css_on_flush.set(true);
            }
            OptionSet::ShowTabline(show) => {
                self.obj()
                    .set_property("show-tabline", ShowTabline::from(show).to_value());

                self.resize_on_flush.set(true);
                self.css_on_flush.set(true);
            }
            OptionSet::Unknown(_) => {}
        }
    }

    async fn send_nvim_input(&self, input: String) {
        let res = self
            .nvim
            .nvim_input(&input)
            .await
            .expect("call to nvim failed");

        // TODO(ville): nvim_input handle the returned bytes written value.
        res.await.expect("nvim_input failed");
    }
}

#[gtk::template_callbacks]
impl AppWindow {
    #[template_callback]
    async fn im_commit(&self, input: &str) {
        // NOTE(ville): "<" needs to be escaped for nvim_input (see `:h nvim_input`)
        let input = input.replace('<', "<lt>");
        self.send_nvim_input(input).await;
    }

    #[template_callback]
    fn key_pressed(
        &self,
        keyval: gdk::Key,
        _keycode: u32,
        state: gdk::ModifierType,
    ) -> glib::Propagation {
        let evt = self
            .event_controller_key
            .borrow()
            .current_event()
            .expect("failed to get event");

        // If the input is a modifier only event, ignore it.
        if evt
            .downcast_ref::<gdk::KeyEvent>()
            .map(|evt| evt.is_modifier())
            .unwrap_or(false)
        {
            return glib::Propagation::Proceed;
        }

        if self.im_context.borrow().filter_keypress(&evt) {
            glib::Propagation::Stop
        } else {
            if let Some(input) = event_to_nvim_input(keyval, state) {
                spawn_local!(clone!(@weak self as this => async move {
                    this.send_nvim_input(input).await;
                }));

                return glib::Propagation::Stop;
            } else {
                println!(
                    "Failed to turn input event into nvim key (keyval: {})",
                    keyval,
                )
            }

            glib::Propagation::Proceed
        }
    }

    #[template_callback]
    fn key_released(&self) {
        let evt = self
            .event_controller_key
            .borrow()
            .current_event()
            .expect("failed to get event");
        self.im_context.borrow().filter_keypress(&evt);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AppWindow {
    const NAME: &'static str = "AppWindow";
    type Type = super::AppWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Overflower::ensure_type();
        Omnibar::ensure_type();
        Shell::ensure_type();
        Tabline::ensure_type();
        Messages::ensure_type();

        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for AppWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("couldn't get display"),
            &self.css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let uiopts = UiOptions {
            rgb: true,
            ext_linegrid: true,
            ext_multigrid: true,
            ext_popupmenu: true,
            ext_tabline: true,
            ext_cmdline: true,
            ext_messages: true,
            stdin_fd: self.args.borrow().stdin_fd,
            ..Default::default()
        };
        let args = self.args.borrow().nvim_cmd_args();
        let args: Vec<&OsStr> = args.iter().map(|a| a.as_ref()).collect();
        let reader = self.nvim.open(&args, uiopts.stdin_fd.is_some());

        // Start io loop.
        spawn_local!(clone!(@strong obj as app => async move {
            app.imp().io_loop(reader).await;
        }));

        // Call nvim_ui_attach.
        spawn_local!(clone!(@weak self.nvim as nvim => async move {
            let res = nvim
                .nvim_set_client_info(
                    "gnvim",
                    // TODO(ville): Tell the version in client info.
                    &dict![],
                    "ui",
                    &dict![],
                    &dict![],
                ).await.expect("call to nvim failed");

            res.await.expect("nvim_set_client_info failed");

            let res = nvim
                .nvim_ui_attach(80, 30, uiopts)
                .await.expect("call to nvim failed");

            res.await.expect("nvim_ui_attach failed");
        }));

        // TODO(ville): Figure out if we should use preedit or not.
        self.im_context.borrow().set_use_preedit(false);

        self.event_controller_key
            .borrow()
            .set_im_context(Some(&*self.im_context.borrow()));

        obj.add_controller(self.event_controller_key.borrow().clone());
    }
}

impl WidgetImpl for AppWindow {
    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        self.omnibar.set_max_height(height);
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
