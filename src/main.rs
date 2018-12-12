#![cfg_attr(feature = "unstable", feature(test))]

extern crate cairo;

extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate neovim_lib;
extern crate pango;
extern crate pangocairo;

use gio::prelude::*;

use neovim_lib::neovim::{Neovim, UiAttachOptions};
use neovim_lib::session::Session as NeovimSession;
use neovim_lib::NeovimApi;

use std::env;
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

mod nvim_bridge;
mod thread_guard;
mod ui;

static GNVIM_RUNTIME_PATH_VAR: &str = "GNVIM_RUNTIME_PATH";
static GNVIM_RUNTIME_PATH: &str = "/usr/local/share/gnvim/runtime";

fn build(app: &gtk::Application) {
    let (tx, rx) = channel();

    let bridge = nvim_bridge::NvimBridge::new(tx);

    let nvim_path = env::args()
        .find(|arg| arg.starts_with("--nvim"))
        .and_then(|arg| arg.split("=").nth(1).map(str::to_owned))
        .unwrap_or(String::from("nvim"));

    let rtp = env::var(GNVIM_RUNTIME_PATH_VAR)
        .unwrap_or(GNVIM_RUNTIME_PATH.to_string());

    let mut cmd = Command::new(&nvim_path);
    cmd.arg("--embed")
        .arg("--cmd")
        .arg("let g:gnvim=1")
        .arg("--cmd")
        .arg("set termguicolors")
        .arg("--cmd")
        .arg(format!("let &rtp.=',{}'", rtp));

    let print_nvim_cmd = env::args().find(|arg| arg == "--print-nvim-cmd");
    if print_nvim_cmd.is_some() {
        println!("nvim cmd: {:?}", cmd);
    }

    let mut session = NeovimSession::new_child_cmd(&mut cmd).unwrap();
    session.start_event_loop_handler(bridge);

    let mut nvim = Neovim::new(session);
    nvim.subscribe("Gnvim")
        .expect("Failed to subscribe to 'Gnvim' events");

    let mut ui_opts = UiAttachOptions::new();
    ui_opts.set_rgb(true);
    ui_opts.set_linegrid_external(true);
    ui_opts.set_popupmenu_external(true);
    ui_opts.set_tabline_external(true);
    ui_opts.set_cmdline_external(true);
    ui_opts.set_wildmenu_external(true);
    nvim.ui_attach(80, 30, &ui_opts)
        .expect("Failed to attach UI");

    let ui = ui::UI::init(app, rx, Arc::new(Mutex::new(nvim)));
    ui.start();
}

fn main() {
    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);
    flags.insert(gio::ApplicationFlags::HANDLES_OPEN);
    let app =
        gtk::Application::new("com.github.vhakulinen.gnvim", flags).unwrap();

    app.connect_activate(|app| {
        build(app);
    });

    app.run(&vec![]);
}
