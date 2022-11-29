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

```
$ # Install cargo (e.g. the rust toolchain)
$ # Install gtk4 dev files, e.g. apt install gtk-4-dev
$ make build
$ sudo make install
```

# Development

Gnvim comes with custom rpc client which uses code generation for generating
bindings to the Neovim API. This is done by the `scripts/generate-bindings.sh`
script and requires the `moreutils` package.
