use gtk::gio;

use crate::APPID;

glib::wrapper! {
    pub struct App(ObjectSubclass<imp::App>)
        @extends gio::Application, adw::Application;
}

#[derive(Default, glib::Boxed, Clone, Copy)]
#[boxed_type(name = "fd-boxed")]
pub struct Fd(Option<i32>);

impl std::ops::Deref for Fd {
    type Target = Option<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for App {
    fn default() -> Self {
        let mut flags = gio::ApplicationFlags::empty();
        flags.insert(gio::ApplicationFlags::HANDLES_OPEN);

        glib::Object::builder()
            .property("application-id", APPID)
            .property("flags", flags)
            .build()
    }
}

mod imp {
    #[cfg(feature = "flatpak")]
    use std::process::Command;
    use std::{cell::RefCell, io::IsTerminal};

    use adw::subclass::prelude::*;
    use gtk::{gio, prelude::*};

    use crate::{components::appwindow::AppWindow, debug};

    #[cfg(not(feature = "flatpak"))]
    fn default_rtp() -> Option<String> {
        Some(String::from("/usr/local/share/gnvim/runtime"))
    }

    #[cfg(feature = "flatpak")]
    fn default_rtp() -> Option<String> {
        glib::user_data_dir()
            .join("gnvim")
            .join("runtime")
            .to_str()
            .map(String::from)
    }

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::App)]
    pub struct App {
        stdin_fd: RefCell<super::Fd>,

        #[property(get)]
        window: RefCell<Option<AppWindow>>,

        #[property(get, set, construct, default = "nvim")]
        nvim: RefCell<String>,
        #[property(get, set, construct, default = default_rtp().as_deref())]
        rtp: RefCell<String>,

        #[property(get, set)]
        nvim_args: RefCell<Vec<String>>,
    }

    impl App {
        fn main_window(&self) -> AppWindow {
            self.window
                .borrow_mut()
                .get_or_insert_with(|| {
                    let obj = self.obj();

                    let mut args = vec![];

                    #[cfg(feature = "flatpak")]
                    args.extend_from_slice(&[
                        String::from("flatpak-spawn"),
                        String::from("--host"),
                    ]);

                    args.extend_from_slice(&[
                        obj.nvim(),
                        String::from("--embed"),
                        String::from("--cmd"),
                        format!("let &rtp.=',{}'", obj.rtp()),
                        String::from("--cmd"),
                        format!("let g:gnivm_rtp_path='{}'", obj.rtp()),
                    ]);

                    args.extend_from_slice(&obj.nvim_args());

                    glib::Object::builder()
                        .property("application", obj.upcast_ref::<adw::Application>())
                        .property("nvim-args", args)
                        .property("stdin-fd", *self.stdin_fd.borrow())
                        .build()
                })
                .clone()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for App {
        const NAME: &'static str = "App";
        type Type = super::App;
        type ParentType = adw::Application;
    }

    #[glib::derived_properties]
    impl ObjectImpl for App {
        fn constructed(&self) {
            let obj = self.obj();
            obj.add_main_option(
                "nvim",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::String,
                "Path to nvim binary",
                Some(&obj.nvim()),
            );
            obj.add_main_option(
                "rtp",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::String,
                "Path to gnvim runtime files [env: GNVIM_RUNTIME_PATH]",
                Some(&obj.rtp()),
            );
            obj.add_main_option(
                "version",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Print version",
                None,
            );
            obj.add_main_option(
                "nvim-args",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::String,
                "Arguments for neovim. Must be a string, i.e. --nvim-args \"--cmd 'echom 32'\"",
                None,
            );
            obj.add_main_option(
                "new",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Open a new instance",
                None,
            );
            obj.add_main_option(
                "no-stdin",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Ignore stdin pipe",
                None,
            );

            #[cfg(feature = "flatpak")]
            obj.add_main_option(
                "install-runtime-files",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Install GNvim's runtime files",
                None,
            );

            #[cfg(feature = "flatpak")]
            obj.add_main_option(
                "purne-runtime-files",
                glib::Char::from(0),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Remove GNvim's runtime files",
                None,
            );

            obj.set_option_context_parameter_string(Some("FILES..."));
            obj.set_option_context_summary(Some(
                "NOTE that nvim arguments are only passed to nvim when a new \
                instance is launched.\n\n\
                By default existing gnvim instance is used for opening files, \
                if one is found.",
            ));

            self.parent_constructed();
        }
    }

    impl AdwApplicationImpl for App {}

    impl ApplicationImpl for App {
        fn startup(&self) {
            debug!("Application::startup");
            self.parent_startup();
        }

        fn handle_local_options(&self, options: &glib::VariantDict) -> glib::ExitCode {
            debug!("Application::handle_local_options");
            let obj = self.obj();

            // Duplicate the stdin fd if the user is trying to pipe content to us.
            // See `:h ui-startup-stdin`.
            if !std::io::stdin().is_terminal() && !options.contains("no-stdin") {
                // Duplicate the fd for the nvim subprocess.
                *self.stdin_fd.borrow_mut() = super::Fd(crate::fd::dup_stdin());

                // We don't currently support opening stdin over dbus, so don't
                // try to reuse any existing instances.
                let mut flags = obj.flags();
                flags.insert(gio::ApplicationFlags::NON_UNIQUE);
                obj.set_flags(flags);
            }

            if options.contains("version") {
                println!("gnvim {}", env!("CARGO_PKG_VERSION"));
                return glib::ExitCode::from(0);
            }

            #[cfg(feature = "flatpak")]
            if options.contains("install-runtime-files") {
                let target = glib::user_data_dir().join("gnvim");
                println!("Installing runtime files to {}", target.to_string_lossy());
                Command::new("cp")
                    .arg("-r")
                    .arg("/app/share/gnvim")
                    .arg(target)
                    .spawn()
                    .expect("failed to start `cp`")
                    .wait()
                    .expect("failed to install runtime files");

                return glib::ExitCode::from(0);
            }

            #[cfg(feature = "flatpak")]
            if options.contains("purne-runtime-files") {
                let target = glib::user_data_dir().join("gnvim");
                println!("Purning {}", target.to_string_lossy());
                Command::new("rm")
                    .arg("-rI")
                    .arg(target)
                    .spawn()
                    .expect("failed to start `rm`")
                    .wait()
                    .expect("failed to purne runtime files");

                return glib::ExitCode::from(0);
            }

            if options.contains("new") {
                let mut flags = obj.flags();
                flags.insert(gio::ApplicationFlags::NON_UNIQUE);
                obj.set_flags(flags);
            }

            if let Some(nvim) = options
                .lookup::<String>("nvim")
                .expect("invalid nvim argument type")
            {
                debug!("nvim arg: {}", nvim);
                obj.set_nvim(nvim);
            }

            if let Some(rtp) = options
                .lookup::<String>("rtp")
                .expect("invalid rtp argument type")
            {
                debug!("rtp arg: {}", rtp);
                obj.set_rtp(rtp);
            }

            if let Some(nvim_args) = options
                .lookup::<String>("nvim-args")
                .expect("invalid nvim-args argument type")
            {
                debug!("nvim-args arg: {:?}", nvim_args);
                // FIXME: should not need to convert between String <-> OsString.
                obj.set_nvim_args(
                    glib::shell_parse_argv(&nvim_args)
                        .expect("failed to parse nvim-args")
                        .into_iter()
                        .map(|v| {
                            v.into_string()
                                .expect("nvim-args should be valid unicode strings")
                        })
                        .collect::<Vec<_>>(),
                );
            }

            self.parent_handle_local_options(options)
        }

        fn open(&self, files: &[gtk::gio::File], _hint: &str) {
            let win = self.main_window();
            win.open_files(files);
            win.present();
        }

        fn activate(&self) {
            self.main_window().present();
        }
    }

    impl GtkApplicationImpl for App {}
}
