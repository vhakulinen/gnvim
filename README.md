# GNvim - Neovim GUI with rust and gtk

Highly experimental.

# Running

By default, gnvim will use `nvim` to run neovim. If you want to change that,
you can use `--nvim` argument like so: `cargo run -- --nvim=/path/to/bin/nvim`.

For now, gnvim has hard coded path to load its own vim files and that path
is set to `~/src/gnvim/runtime`.

## Requirements

GNvim requires

    * Stable rust to compile
    * Latest nvim master
    * Gtk version 3.18 or higher

There are some benchmarks for internal data structures, but to run those you'll
need nightly rust. To run those benchmarks, use `cargo bench --features=unstable`
command.
