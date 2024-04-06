PREFIX ?= /usr/local
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
	cargo build $(CARGOARGS) --bin gnvim --profile $(PROFILE)

install:
	@echo "Installing gnvim binary"
	install -D ./target/$(PROFILE)/gnvim "$(DESTDIR)$(PREFIX)/bin/gnvim"
	@echo "Installing runtime"
	install -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"
	@echo "Installing desktop file"
	install -D ./desktop/gnvim.desktop "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	sed -i "s/@icon@/$(APPID)/" "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	@echo "Installing icons"
	install -D ./desktop/gnvim_128.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	install -D ./desktop/gnvim_48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	install -D ./desktop/gnvim-logo.svg "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"
	$(MAKE) install-schema

install-schema:
	@echo "Installing schema"
	@install -D \
		./ui/data/com.github.vhakulinen.gnvim.gschema.xml.in \
		"$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/com.github.vhakulinen.gnvim.gschema.xml"
	@sed -i \
		"s/@appid@/$(APPID)/" \
		"$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/com.github.vhakulinen.gnvim.gschema.xml"
	@glib-compile-schemas "$(DESTDIR)$(PREFIX)/share/glib-2.0/schemas"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm -rf "$(DESTDIR)$(PREFIX)/share/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"

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
		org.gnome.Sdk//46 \
		org.gnome.Platform//46 \
		org.freedesktop.Sdk.Extension.rust-stable//23.08 \
		org.freedesktop.Sdk.Extension.llvm16//23.08

install-flatpak:
	@flatpak-builder \
		--user \
		--install \
		--force-clean \
		$(BUILDDIR) \
		$(MANIFEST)
