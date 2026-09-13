# Host OS, for platform specific install steps.
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
# Homebrew's glib only searches $(brew --prefix)/share and /usr/share for
# GSettings schemas, so install under the brew prefix by default.
BREW_PREFIX := $(shell brew --prefix 2>/dev/null)
PREFIX ?= $(if $(BREW_PREFIX),$(BREW_PREFIX),/usr/local)
else
PREFIX ?= /usr/local
endif
# Rust build profile.  See Cargo.toml for alternatives.
PROFILE ?= optimized
# Build dir for building flatpak
BUILDDIR := "builddir"
# Build dir for running gnvim in flatpak
DEVDIR := "devdir"
MANIFEST := "com.github.vhakulinen.gnvim.Devel.yml"
# App ID used for the build.
#
# Propagates to the gnvim build script.
APPID := "com.github.vhakulinen.gnvim.Devel"

# When building flatpak, use FLATPAK_ID as APPID.
ifneq ($(FLATPAK_ID),)
	# NOTE: assign to _, so make doesn't interpret the line as being part of
	# a recipe.
	_ = $(info Overriding APPID with FLATPAK_ID)
	APPID = $(FLATPAK_ID)
endif

build:
	PREFIX=$(PREFIX) cargo build $(CARGOARGS) --bin gnvim --profile $(PROFILE)

install:
	@echo "Installing gnvim binary"
	install -d "$(DESTDIR)$(PREFIX)/bin"
	install ./target/$(PROFILE)/gnvim "$(DESTDIR)$(PREFIX)/bin/gnvim"
	@echo "Installing runtime"
	install -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"
ifneq ($(UNAME_S),Darwin)
	@echo "Installing desktop file"
	install -d "$(DESTDIR)$(PREFIX)/share/applications"
	sed "s/@icon@/$(APPID)/" ./desktop/gnvim.desktop \
		> "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	@echo "Installing icons"
	install -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps"
	install ./desktop/gnvim_128.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	install -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps"
	install ./desktop/gnvim_48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	install -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps"
	install ./desktop/gnvim-logo.svg "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"
endif
	$(MAKE) install-schema

install-schema:
	@echo "Installing schema"
	@install -d "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas"
	@sed "s/@appid@/$(APPID)/" \
		./ui/data/com.github.vhakulinen.gnvim.gschema.xml.in \
		> "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/com.github.vhakulinen.gnvim.gschema.xml"
	@glib-compile-schemas "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm -rf "$(DESTDIR)$(PREFIX)/share/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/com.github.vhakulinen.gnvim.gschema.xml"
	glib-compile-schemas "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas"
ifneq ($(UNAME_S),Darwin)
	rm "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"
endif

dev-base:
	@flatpak-builder \
		--force-clean \
		--user \
		--keep-build-dirs \
		--stop-at=gnvim \
		$(DEVDIR) \
		$(MANIFEST)
	@cargo run --bin flatpak-helper \
		-- $(DEVDIR) make install-schema

run: dev-base
	@cargo run --bin flatpak-helper \
		-- $(DEVDIR) cargo run \
		--bin gnvim \
		--features flatpak \
		-- --new

shell: dev-base
	@cargo run --bin flatpak-helper \
		-- $(DEVDIR) sh

install-flatpak-deps:
	@flatpak install --user \
		org.gnome.Sdk//47 \
		org.gnome.Platform//47 \
		org.freedesktop.Sdk.Extension.rust-stable//24.08 \
		org.freedesktop.Sdk.Extension.llvm18//24.08

install-flatpak:
	@flatpak-builder \
		--user \
		--install \
		--force-clean \
		$(BUILDDIR) \
		$(MANIFEST)
