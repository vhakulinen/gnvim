#![cfg_attr(feature = "unstable", feature(test))]

#[cfg(feature = "libwebkit2gtk")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "libwebkit2gtk")]
extern crate ammonia;
#[cfg(feature = "libwebkit2gtk")]
extern crate pulldown_cmark;
extern crate structopt;
#[cfg(feature = "libwebkit2gtk")]
extern crate syntect;

extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate log;
extern crate pango;
extern crate pangocairo;
#[cfg(feature = "libwebkit2gtk")]
extern crate webkit2gtk;

use gio::prelude::*;

use log::error;

use structopt::{clap, StructOpt};

include!(concat!(env!("OUT_DIR"), "/gnvim_version.rs"));

mod nvim_bridge;
mod nvim_gio;
mod thread_guard;
mod ui;

fn parse_geometry(input: &str) -> Result<(i32, i32), String> {
    let ret_tuple: Vec<&str> = input.split("x").collect();
    if ret_tuple.len() != 2 {
        Err(String::from("must be of form 'width'x'height'"))
    } else {
        match (ret_tuple[0].parse(), ret_tuple[1].parse()) {
            (Ok(x), Ok(y)) => Ok((x, y)),
            (_, _) => {
                Err(String::from("at least one argument wasn't an integer"))
            }
        }
    }
}

/// Gnvim is a graphical UI for neovim.
#[derive(StructOpt, Debug)]
#[structopt(
    name = "gnvim",
    version = VERSION,
    author = "Ville Hakulinen"
)]
struct Options {
    /// Prints the executed neovim command.
    #[structopt(long = "print-nvim-cmd")]
    print_nvim_cmd: bool,

    /// Path to neovim binary.
    #[structopt(long = "nvim", name = "BIN", default_value = "nvim")]
    nvim_path: String,

    /// Path for gnvim runtime files.
    #[structopt(
        long = "gnvim-rtp",
        default_value = "/usr/local/share/gnvim/runtime",
        env = "GNVIM_RUNTIME_PATH"
    )]
    gnvim_rtp: String,

    /// Files to open.
    #[structopt(value_name = "FILES")]
    open_files: Vec<String>,

    /// Arguments that are passed to nvim.
    #[structopt(value_name = "ARGS", last = true)]
    nvim_args: Vec<String>,

    /// Disables externalized popup menu
    #[structopt(long = "disable-ext-popupmenu")]
    disable_ext_popupmenu: bool,

    /// Disables externalized command line
    #[structopt(long = "disable-ext-cmdline")]
    disable_ext_cmdline: bool,

    /// Disables externalized tab line
    #[structopt(long = "disable-ext-tabline")]
    disable_ext_tabline: bool,

    /// Geometry of the window in widthxheight form
    #[structopt(long = "geometry", parse(try_from_str = parse_geometry), default_value = "1280x720")]
    geometry: (i32, i32),
}

enum Error {
    Start(nvim_gio::Error),
    Call(Box<nvim_rs::error::CallError>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Start(e) => write!(fmt, "Failed to start nvim: {}", e),
            Error::Call(e) => write!(fmt, "Call to nvim failed: {}", e),
        }
    }
}

impl From<nvim_gio::Error> for Error {
    fn from(arg: nvim_gio::Error) -> Self {
        Error::Start(arg)
    }
}

impl From<Box<nvim_rs::error::CallError>> for Error {
    fn from(arg: Box<nvim_rs::error::CallError>) -> Self {
        Error::Call(arg)
    }
}

async fn build(app: &gtk::Application, opts: &Options) -> Result<(), Error> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let bridge = nvim_bridge::NvimBridge::new(tx.clone());

    let rtp = format!("let &rtp.=',{}'", opts.gnvim_rtp);
    let mut args: Vec<&str> = vec![
        &opts.nvim_path,
        "--embed",
        "--cmd",
        "let g:gnvim=1",
        "--cmd",
        "set termguicolors",
        "--cmd",
        &rtp,
    ];

    // Pass arguments from cli to nvim.
    for arg in opts.nvim_args.iter() {
        args.push(arg);
    }

    // Open files "normally" through nvim.
    for file in opts.open_files.iter() {
        args.push(file);
    }

    // Print the nvim cmd which is executed if asked.
    if opts.print_nvim_cmd {
        println!("nvim cmd: {:?}", args);
    }

    let mut nvim = nvim_gio::new_child(
        bridge,
        args.iter().map(|a| std::ffi::OsStr::new(a)).collect(),
        tx,
    )
    .map_err(Error::from)?;

    nvim.subscribe("Gnvim").await.map_err(Error::from)?;

    let api_info = nvim.get_api_info().await.map_err(Error::from)?;
    nvim.set_var("gnvim_channel_id", api_info[0].clone())
        .await
        .map_err(Error::from)?;

    let mut ui_opts = nvim_rs::UiAttachOptions::new();
    ui_opts.set_rgb(true);
    ui_opts.set_linegrid_external(true);
    ui_opts.set_popupmenu_external(!opts.disable_ext_popupmenu);
    ui_opts.set_tabline_external(!opts.disable_ext_tabline);
    ui_opts.set_cmdline_external(!opts.disable_ext_cmdline);

    ui_opts.set_wildmenu_external(true);
    nvim.ui_attach(80, 30, &ui_opts)
        .await
        .map_err(Error::from)?;

    let ui = ui::UI::init(app, rx, opts.geometry, nvim);
    ui.start();

    Ok(())
}

fn main() {
    env_logger::init();

    let opts = Options::clap();
    let opts = Options::from_clap(&opts.get_matches_safe().unwrap_or_else(
        |mut err| {
            if let clap::ErrorKind::UnknownArgument = err.kind {
                // Arg likely passed for nvim, notify user of how to pass args to nvim.
                err.message = format!(
                    "{}\n\nIf this is an argument for nvim, try moving \
                     it after a -- separator.",
                    err.message
                );
                err.exit();
            } else {
                err.exit()
            }
        },
    ));

    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);
    flags.insert(gio::ApplicationFlags::HANDLES_OPEN);
    let app = gtk::Application::new(Some("com.github.vhakulinen.gnvim"), flags)
        .unwrap();

    gdk::set_program_class("GNvim");
    glib::set_application_name("GNvim");
    gtk::Window::set_default_icon_name("gnvim");

    app.connect_activate(move |app| {
        let opts = &opts;
        let c = glib::MainContext::default();
        c.block_on(async move {
            if let Err(err) = build(app, opts).await {
                error!("Failed to build UI: {}", err);
            }
        });
    });

    app.run(&vec![]);
}
