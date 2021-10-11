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

extern crate gtk;
extern crate pangocairo;
#[cfg(feature = "libwebkit2gtk")]
extern crate webkit2gtk;

use gtk::prelude::*;
use gtk::traits::SettingsExt;
use gtk::{gdk, gio, glib};

use log::error;

mod args;
mod error;
mod nvim_bridge;
mod nvim_gio;
mod thread_guard;
mod ui;

use crate::error::Error;

async fn build(app: &gtk::Application, args: &args::Args) -> Result<(), Error> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let bridge = nvim_bridge::NvimBridge::new(tx.clone());

    let cmd_args = args.nvim_cmd();

    // Print the nvim cmd which is executed if asked.
    if args.print_nvim_cmd {
        println!("nvim cmd: {:?}", cmd_args);
    }

    let mut nvim = nvim_gio::new_child(
        bridge,
        cmd_args.iter().map(|a| std::ffi::OsStr::new(a)).collect(),
        tx,
    )?;

    nvim.subscribe("Gnvim").await?;

    let api_info = nvim.get_api_info().await?;
    nvim.set_var("gnvim_channel_id", api_info[0].clone())
        .await?;

    nvim.ui_attach(80, 30, &args.nvim_ui_opts()).await?;

    let ui =
        ui::UI::init(app, rx, args.geometry, nvim).expect("failed to init ui");
    ui.start();

    Ok(())
}

fn main() {
    env_logger::init();

    let args = args::Args::from_cli();

    if let Err(err) = gtk::init() {
        error!("Failed to initialize gtk: {}", err);
        return;
    }

    let mut flags = gio::ApplicationFlags::empty();
    flags.insert(gio::ApplicationFlags::NON_UNIQUE);
    flags.insert(gio::ApplicationFlags::HANDLES_OPEN);
    let app = gtk::Application::new(Some("com.github.vhakulinen.gnvim"), flags);

    gdk::set_program_class("GNvim");
    glib::set_application_name("GNvim");
    gtk::Window::set_default_icon_name("gnvim");

    if args.prefer_dark_theme {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }
    }

    app.connect_activate(move |app| {
        let args = &args;
        let c = glib::MainContext::default();
        c.block_on(async move {
            if let Err(err) = build(app, args).await {
                error!("Failed to build UI: {:?}", err);
            }
        });
    });

    app.run();
}
