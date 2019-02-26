#![cfg_attr(feature = "unstable", feature(test))]

#[macro_use]
extern crate lazy_static;
extern crate ammonia;
extern crate pulldown_cmark;
extern crate structopt;
extern crate syntect;

extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate neovim_lib;
extern crate pango;
extern crate pangocairo;
extern crate webkit2gtk;

use gio::prelude::*;

use neovim_lib::neovim::{Neovim, UiAttachOptions};
use neovim_lib::session::Session as NeovimSession;
use neovim_lib::NeovimApi;

use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use structopt::StructOpt;

include!(concat!(env!("OUT_DIR"), "/gnvim_version.rs"));

mod nvim_bridge;
mod thread_guard;
mod ui;

/// Gnvim is a graphical UI for neovim.
#[derive(StructOpt, Debug)]
#[structopt(
    name = "gnvim",
    raw(version = "VERSION"),
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

    /// Files to open. Files after the first one are opened in new tabs.
    #[structopt(value_name = "FILES")]
    open_files: Vec<String>,

    /// Arguments that are passed to nvim.
    #[structopt(value_name = "ARGS", last = true)]
    nvim_args: Vec<String>,
}

fn build(app: &gtk::Application, opts: &Options) {
    let (tx, rx) = channel();

    let bridge = nvim_bridge::NvimBridge::new(tx);

    let mut cmd = Command::new(&opts.nvim_path);
    cmd.arg("--embed")
        .arg("--cmd")
        .arg("let g:gnvim=1")
        .arg("--cmd")
        .arg("set termguicolors")
        .arg("--cmd")
        .arg(format!("let &rtp.=',{}'", opts.gnvim_rtp));

    // Pass arguments from cli to nvim.
    for arg in opts.nvim_args.iter() {
        cmd.arg(arg);
    }

    // Print the nvim cmd which is executed if asked.
    if opts.print_nvim_cmd {
        println!("nvim cmd: {:?}", cmd);
    }

    let mut session = NeovimSession::new_child_cmd(&mut cmd).unwrap();
    session.start_event_loop_handler(bridge);

    let mut nvim = Neovim::new(session);
    nvim.subscribe("Gnvim")
        .expect("Failed to subscribe to 'Gnvim' events");

    let api_info = nvim.get_api_info().expect("Failed to get API info");
    nvim.set_var("gnvim_channel_id", api_info[0].clone())
        .expect("Failed to set g:gnvim_channel_id");

    let mut ui_opts = UiAttachOptions::new();
    ui_opts.set_rgb(true);
    ui_opts.set_linegrid_external(true);
    ui_opts.set_popupmenu_external(true);
    ui_opts.set_tabline_external(true);
    ui_opts.set_cmdline_external(true);
    ui_opts.set_wildmenu_external(true);
    nvim.ui_attach(80, 30, &ui_opts)
        .expect("Failed to attach UI");

    // Open the first file using :e.
    if let Some(first) = opts.open_files.get(0) {
        nvim.command(format!("e {}", first).as_str()).unwrap();
    }
    // Open rest of the files into new tabs.
    for file in opts.open_files.iter().skip(1) {
        nvim.command(format!("tabe {}", file).as_str()).unwrap();
    }

    let ui = ui::UI::init(app, rx, Arc::new(Mutex::new(nvim)));
    ui.start();
}

fn main() {
    let opts = Options::from_args();

    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);
    flags.insert(gio::ApplicationFlags::HANDLES_OPEN);
    let app =
        gtk::Application::new("com.github.vhakulinen.gnvim", flags).unwrap();

    glib::set_application_name("GNvim");
    gtk::Window::set_default_icon_name("gnvim");

    app.connect_activate(move |app| {
        build(app, &opts);
    });

    app.run(&vec![]);
}
