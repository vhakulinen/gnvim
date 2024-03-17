use gtk::gio;

glib::wrapper! {
    pub struct App(ObjectSubclass<imp::App>)
        @extends gio::Application, gtk::Application;
}

#[derive(Default, glib::Boxed, Clone, Copy)]
#[boxed_type(name = "fd-boxed")]
pub struct Fd(pub Option<i32>);

impl App {
    pub fn new(stdin_fd: Option<i32>) -> Self {
        let mut flags = gio::ApplicationFlags::empty();
        flags.insert(gio::ApplicationFlags::HANDLES_OPEN);

        if stdin_fd.is_some() {
            // If the user is piping the content to us, don't try to use any
            // existing instance (because we don't have any support for that
            // at the moment).
            flags.insert(gio::ApplicationFlags::NON_UNIQUE);
        }

        glib::Object::builder()
            .property("application-id", "com.github.vhakulinen.gnvim")
            .property("flags", flags)
            .property("stdin-fd", Fd(stdin_fd))
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::{gio, prelude::*, subclass::prelude::*};

    use crate::{components::appwindow::AppWindow, debug};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::App)]
    pub struct App {
        #[property(get, set, nullable, construct_only)]
        stdin_fd: RefCell<super::Fd>,

        #[property(get)]
        window: RefCell<Option<AppWindow>>,

        #[property(get, set, construct, default = "nvim")]
        nvim: RefCell<String>,
        #[property(get, set, construct, default = "/usr/local/share/gnvim/runtime")]
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

                    let mut args: Vec<String> = vec![
                        obj.nvim(),
                        String::from("--embed"),
                        String::from("--cmd"),
                        String::from(format!("let &rtp.=',{}'", obj.rtp())),
                    ];

                    args.extend_from_slice(&obj.nvim_args());

                    glib::Object::builder()
                        .property("application", obj.upcast_ref::<gtk::Application>())
                        .property("nvim-args", args)
                        .property("stdin-fd", obj.stdin_fd())
                        .build()
                })
                .clone()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for App {
        const NAME: &'static str = "App";
        type Type = super::App;
        type ParentType = gtk::Application;
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

            obj.set_option_context_parameter_string(Some("FILES..."));

            self.parent_constructed();
        }
    }

    impl ApplicationImpl for App {
        fn startup(&self) {
            debug!("Application::startup");
            self.parent_startup();
        }

        fn handle_local_options(&self, options: &glib::VariantDict) -> glib::ExitCode {
            debug!("Application::handle_local_options");
            let obj = self.obj();

            if options.contains("version") {
                println!("gnvim {}", env!("CARGO_PKG_VERSION"));
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
                let nvim_args = glib::shell_parse_argv(&nvim_args)
                    .expect("failed to parse nvim-args")
                    .into_iter()
                    .map(|v| {
                        v.into_string()
                            .expect("nvim-args should be valid unicode strings")
                    })
                    .collect();
                obj.set_nvim_args(&nvim_args);
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
