<p align="center">
	<img src="./desktop/gnvim_128.png" alt="GNvim Logo">
    <h3 align="center">GNvim - GTK4 Neovim GUI</h3>
</p>

Gnvim, opinionated Neovim GUI.

<p align="center">
	<img src="https://github.com/vhakulinen/gnvim/wiki/screenshot.png" alt="Screenshot of gnvim">
</p>

_For previous gtk3 version, checkout the `legacy` branch._

# Install

**NOTE**: gnvim requires nvim version 0.9.5 or higher.

## Flatpak

If you're using flatpak, you can build a GNvim flatpak locally:

```
$ # Install flatpak-builder
$ sudo apt install flatpak-builder
$ make install-flatpak-deps # Install required runtime and SDKs
$ make install-flatpak # Build and install the flatpak
$ # Optional: install gnvim runtime files (i.e. for `lua require('gnvim')`)
$ flatpak run com.github.vhakulinen.gnvim.Devel --install-runtime-files
```

## System

Install system wide:

```
$ # Install cargo (e.g. the rust toolchain)
$ # Install gtk4 dev files, e.g. apt install libgtk-4-dev (see below)
$ make build
$ sudo make install
```

More detailed instructions regarding gtk4 requirements, see the gtk-rs book's
[instructions](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)).

# Documentation

See `:h gnvim` and/or `runtime/doc/gnvim.txt`.

# Development

Gnvim comes with custom rpc client which uses code generation for generating
bindings to the Neovim API. This is done by the `scripts/generate-bindings.sh`
script and requires the `moreutils` package.
