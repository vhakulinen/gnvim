# GNvim - Rich Neovim GUI without any web bloat

Gnvim is still in active development! That said, I've used it daily for the
past 6 months, both at work and for developing gnvim.

[Find some screenshots here](https://github.com/vhakulinen/gnvim/wiki)

## Features

* No electron (!), build on GTK.
* Ligatures
* Custom cursor tooltip feature to display markdown documents.
  Useful for implementing features like hover information or signature help
  (see [gnvim-lsp](https://github.com/vhakulinen/gnvim-lsp)).
* A lot of the nvim external features implemented
    - Popupmenu
        * Own view for `preview` (`:h completeopt`).
    - Tabline
    - Cmdline
    - Wildmenu

More externalized features will follow as they are implemented for neovim.

## Requirements

GNvim requires

* Stable rust to compile
* Latest nvim master (gnvim 0.1.0 works with nvim 0.3.4)
* Gtk version 3.18 or higher

There are some benchmarks for internal data structures, but to run those you'll
need nightly rust. To run those benchmarks, use `cargo bench --features=unstable`
command.

# Install

You're required to have rust tool chain available. Once you have that, clone
this repo and run `make build` followed by `sudo make install`.

# Running

GNvim requires some runtime files to be present and loaded by nvim to work
properly. By default, gnvim will look this files from `/usr/local/share/gnvim/runtime`,
but this can be changed by `GNVIM_RUNTIME_PATH` environment variable.

By default, gnvim will use `nvim` to run neovim. If you want to change that,
you can use `--nvim` flag (e.g. `gnvim --nvim=/path/to/nvim`).

For debugging purposes, there is `--print-nvim-cmd` flag to tell gnvim to print
the executed nvim command.

See `gnvim --help` for all the cli arguments.
