use structopt::{clap, StructOpt};

include!(concat!(env!("OUT_DIR"), "/gnvim_version.rs"));

/// Gnvim is a graphical UI for neovim.
#[derive(StructOpt, Debug)]
#[structopt(
    name = "gnvim",
    version = VERSION,
    author = "Ville Hakulinen"
)]
pub struct Args {
    /// Prints the executed neovim command.
    #[structopt(long = "print-nvim-cmd")]
    pub print_nvim_cmd: bool,

    /// Path to neovim binary.
    #[structopt(long = "nvim", name = "BIN", default_value = "nvim")]
    pub nvim_path: String,

    /// Path for gnvim runtime files.
    #[structopt(
        long = "gnvim-rtp",
        default_value = "/usr/local/share/gnvim/runtime",
        env = "GNVIM_RUNTIME_PATH"
    )]
    pub gnvim_rtp: String,

    /// Files to open.
    #[structopt(value_name = "FILES")]
    pub open_files: Vec<String>,

    /// Arguments that are passed to nvim.
    #[structopt(value_name = "ARGS", last = true)]
    pub nvim_args: Vec<String>,

    /// Disables externalized popup menu
    #[structopt(long = "disable-ext-popupmenu")]
    pub disable_ext_popupmenu: bool,

    /// Disables externalized command line
    #[structopt(long = "disable-ext-cmdline")]
    pub disable_ext_cmdline: bool,

    /// Disables externalized tab line
    #[structopt(long = "disable-ext-tabline")]
    pub disable_ext_tabline: bool,

    /// Instruct GTK to prefer dark theme
    #[structopt(long = "gtk-prefer-dark-theme")]
    pub prefer_dark_theme: bool,

    /// Geometry of the window in widthxheight form
    #[structopt(long = "geometry", parse(try_from_str = parse_geometry), default_value = "1280x720")]
    pub geometry: (i32, i32),
}

impl Args {
    pub fn from_cli() -> Self {
        let args = Self::clap();
        Self::from_clap(&args.get_matches_safe().unwrap_or_else(|mut err| {
            if let clap::ErrorKind::UnknownArgument = err.kind {
                // Arg likely passed for nvim, notify user of how to pass args to nvim.
                err.message = format!(
                    "{}\n\nIf this is an argument for nvim, try moving \
                         it after a -- separator.",
                    err.message
                );
            }

            err.exit()
        }))
    }

    pub fn nvim_ui_opts(&self) -> nvim_rs::UiAttachOptions {
        let mut ui_opts = nvim_rs::UiAttachOptions::new();
        ui_opts.set_rgb(true);
        ui_opts.set_linegrid_external(true);
        ui_opts.set_multigrid_external(true);
        ui_opts.set_popupmenu_external(!self.disable_ext_popupmenu);
        ui_opts.set_tabline_external(!self.disable_ext_tabline);
        ui_opts.set_cmdline_external(!self.disable_ext_cmdline);

        ui_opts
    }

    pub fn nvim_cmd(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![
            self.nvim_path.clone(),
            "--embed".to_string(),
            "--cmd".to_string(),
            "let g:gnvim=1".to_string(),
            "--cmd".to_string(),
            "set termguicolors".to_string(),
            "--cmd".to_string(),
            format!("let &rtp.=',{}'", self.gnvim_rtp),
        ];

        // Pass arguments from cli to nvim.
        for arg in self.nvim_args.iter() {
            args.push(arg.to_string());
        }

        // Open files "normally" through nvim.
        for file in self.open_files.iter() {
            args.push(file.to_string());
        }

        args
    }
}

fn parse_geometry(input: &str) -> Result<(i32, i32), String> {
    let ret_tuple: Vec<&str> = input.split('x').collect();
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
