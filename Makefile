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
	install -D ./target/$(PROFILE)/gnvim "$(DESTDIR)$(PREFIX)/bin/gnvim"
	install -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"
	install -D ./desktop/gnvim.desktop "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	sed -i "s/@icon@/$(APPID)/" "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	install -D ./desktop/gnvim_128.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	install -D ./desktop/gnvim_48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	install -D ./desktop/gnvim-logo.svg "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm -rf "$(DESTDIR)$(PREFIX)/share/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/applications/$(APPID).desktop"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/$(APPID).png"
	rm "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/$(APPID).svg"

run-base:
	@flatpak-builder \
		--force-clean \
		--user \
		--keep-build-dirs \
		--stop-at=gnvim \
		$(DEVDIR) \
		$(MANIFEST)

run: run-base
	@cargo run --bin flatpak-helper \
		-- $(DEVDIR) cargo run \
		--bin gnvim \
		--features flatpak \
		-- --new

shell: run-base
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
