use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use pango;
use gtk;
use glib;
use gtk::prelude::*;
use neovim_lib::{Neovim, neovim_api::{Tabpage, NeovimApi}};
use ui::color::Color;

use nvim_bridge;
#[macro_use]
use ui;

pub struct Tabline {
    notebook: gtk::Notebook,
    css_provider: gtk::CssProvider,
    nvim: Arc<Mutex<Neovim>>,
    switch_tab_signal: glib::SignalHandlerId,

    tabpage_data: Rc<RefCell<Box<Vec<Tabpage>>>>,
}

impl Tabline {
    pub fn new(nvim: Arc<Mutex<Neovim>>) -> Self {
        let notebook = gtk::Notebook::new();
        notebook.set_show_border(false);

        let css_provider = gtk::CssProvider::new();
        add_css_provider!(&css_provider, notebook);

        let tabpage_data = Rc::new(RefCell::new(Box::new(vec!())));
        let tabpage_data_ref = tabpage_data.clone();
        let nvim_ref = nvim.clone();
        let switch_tab_signal = notebook.connect_switch_page(move |_, _, page_num| {

            let pages = tabpage_data_ref.borrow();
            if let Some(ref page) = pages.get(page_num as usize) {
                let mut nvim = nvim_ref.lock().unwrap();
                nvim.set_current_tabpage(&page).unwrap();
            } else {
                println!("Failed to get tab page {}", page_num);
            }
        });

        Tabline {
            notebook,
            css_provider,
            nvim,
            switch_tab_signal,
            tabpage_data,
        }
    }

    pub fn get_widget(&self) -> gtk::Widget {
        self.notebook.clone().upcast()
    }

    pub fn update(&self, current: Tabpage, tabs: Vec<(Tabpage, String)>) {
        glib::signal_handler_block(&self.notebook, &self.switch_tab_signal);
        for child in self.notebook.get_children() {
            self.notebook.remove(&child);
        }
        glib::signal_handler_unblock(&self.notebook, &self.switch_tab_signal);

        if tabs.len() < 2 {
            self.notebook.hide();
            return;
        }

        glib::signal_handler_block(&self.notebook, &self.switch_tab_signal);
        for tab in tabs.iter() {
            let tab_label = gtk::Label::new(tab.1.as_str());
            tab_label.set_hexpand(true);
            tab_label.set_ellipsize(pango::EllipsizeMode::End);
            add_css_provider!(&self.css_provider, tab_label);

            self.notebook.append_page(
                &gtk::Box::new(gtk::Orientation::Vertical, 0),
                Some(&tab_label));
        }

        self.notebook.show_all();

        let mut nvim = self.nvim.lock().unwrap();
        let page =  current.get_number(&mut nvim).unwrap() - 1;
        self.notebook.set_current_page(Some(page as u32));

        self.tabpage_data.replace(
            Box::new(tabs.iter().map(|t| t.0.clone()).collect()));


        glib::signal_handler_unblock(&self.notebook, &self.switch_tab_signal);
    }

    pub fn get_height(&self) -> i32 {
        self.notebook.get_preferred_height().0
    }

    pub fn set_font(&self, font: &pango::FontDescription) {
        gtk::WidgetExt::override_font(&self.notebook, font);
    }

    pub fn set_colors(&self, colors: &nvim_bridge::TablineColors) {
        if gtk::get_minor_version() < 20 {
            self.set_colors_pre20(colors);
        } else {
            self.set_colors_post20(colors);
        }
    }

    fn set_colors_post20(&self, colors: &nvim_bridge::TablineColors) {
        let css = format!(
            "header {{
                padding: 0px;
                box-shadow: none;
            }}
            label {{
                color: #{normal_fg};
                background: transparent;
            }}
            tab {{
                padding: 5px;
                outline: none;
                background-color: #{normal_bg};
                border: none;
                box-shadow: inset -2px -70px 10px -70px rgba(0,0,0,0.75);
            }}
            tab:checked {{
                border: none;
                box-shadow: inset 73px 0px 0px -70px #{selected_fg};
            }}
            tab:checked, tab:checked > label {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}
            tab:hover {{
                box-shadow: inset 73px 0px 0px -70px #{selected_fg};
            }}
            ",
            normal_fg=colors.fg.to_hex(),
            normal_bg=colors.bg.to_hex(),
            selected_fg=colors.sel_fg.to_hex(),
            selected_bg=colors.sel_bg.to_hex(),
        );

        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }

    fn set_colors_pre20(&self, colors: &nvim_bridge::TablineColors) {
        let css = format!(
            "GtkNotebook {{
                padding: 0px;
                background-color: #{normal_bg};

                -GtkNotebook-initial-gap: 0;
                -GtkNotebook-tab-overlap: 1;
                -GtkNotebook-has-tab-gap: false;
            }}
            GtkLabel {{
                color: #{normal_fg};
                background: transparent;
                font-weight: normal;
                border: none;
            }}
            tab {{
                padding: 5px;
                outline: none;
                background-color: #{normal_bg};
                border: none;
                box-shadow: inset -2px -70px 10px -70px rgba(0,0,0,0.75);
            }}
            tab:active {{
                border: none;
                box-shadow: inset 73px 0px 0px -70px #{selected_fg};
            }}
            tab:active, tab:active > GtkLabel {{
                color: #{selected_fg};
                background-color: #{selected_bg};
            }}
            tab:hover {{
                box-shadow: inset 73px 0px 0px -70px #{selected_fg};
            }}
            ",
            normal_fg=colors.fg.to_hex(),
            normal_bg=colors.bg.to_hex(),
            selected_fg=colors.sel_fg.to_hex(),
            selected_bg=colors.sel_bg.to_hex(),
        );

        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes()).unwrap();
    }
}
