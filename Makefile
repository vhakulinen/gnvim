ifeq ($(PREFIX),)
    PREFIX := /usr/local
endif

build:
	cargo build --release

install:
	install -d "$(DESTDIR)$(PREFIX)/bin"
	install ./target/release/gnvim "$(DESTDIR)$(PREFIX)/bin"
	install -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"
	install -d "$(DESTDIR)$(PREFIX)/share/applications"
	sed -e "s|Exec=gnvim|Exec=$(PREFIX)/bin/gnvim|" \
	    "./desktop/gnvim.desktop" \
	    >"$(DESTDIR)$(PREFIX)/share/applications/gnvim.desktop"
	install -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps"
	install -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps"
	cp ./desktop/gnvim_128.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/gnvim.png"
	cp ./desktop/gnvim_48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/gnvim.png"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm -rf "$(DESTDIR)$(PREFIX)/share/gnvim"
