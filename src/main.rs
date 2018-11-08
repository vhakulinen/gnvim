#![feature(test)]

extern crate cairo;

extern crate gio;
extern crate glib;
extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate pangocairo;
extern crate neovim_lib;

use gio::prelude::*;

use neovim_lib::NeovimApi;
use neovim_lib::neovim::{Neovim, UiAttachOptions};
use neovim_lib::session::Session as NeovimSession;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::env;

mod nvim_bridge;
mod ui;
mod thread_guard;

fn build(app: &gtk::Application) {

    let (tx, rx) = channel();

    let bridge = nvim_bridge::NvimBridge::new(tx);

    let nvim_path = env::args()
        .find(|arg| {
            arg.starts_with("--nvim")
        })
        .and_then(|arg| {
            arg.split("=").nth(1).map(str::to_owned)
        })
        .unwrap_or(String::from("nvim"));

    println!("nvim: {:?}", nvim_path);

    let mut cmd = Command::new(&nvim_path);
    cmd.arg("--embed")
        .arg("--cmd")
        .arg("let g:gnvim=1")
        .arg("--cmd")
        .arg("set termguicolors")
        .arg("--cmd")
        .arg("let &rtp.=',~/src/gnvim/runtime'");

    let mut session = NeovimSession::new_child_cmd(&mut cmd).unwrap();
    session.start_event_loop_handler(bridge);

    let mut nvim = Neovim::new(session);
    let mut ui_opts = UiAttachOptions::new();
    ui_opts.set_rgb(true);
    ui_opts.set_linegrid_external(true);
    ui_opts.set_popupmenu_external(true);
    ui_opts.set_tabline_external(true);
    //ui_opts.set_cmdline_external(true);
    nvim.ui_attach(80, 30, &ui_opts).unwrap();

    nvim.subscribe("Gnvim").unwrap();
    nvim.command("call SetGuiColors()").unwrap();

    let ui = ui::UI::init(app, rx, Arc::new(Mutex::new(nvim)));
    ui.start();
}

fn main() {
    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);
    flags.insert(gio::ApplicationFlags::HANDLES_OPEN);
    let app = gtk::Application::new("com.github.vhakulinen.gnvim", flags).unwrap();

    app.connect_activate(|app| {
        build(app);
    });

    app.run(&vec![]);
}
