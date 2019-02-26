ifeq ($(PREFIX),)
    PREFIX := /usr/local
endif

build:
	cargo build --release

syntect-pack:
	git submodule update --init
	find sublime-syntaxes/sources \
	    -name '*.sublime-syntax' \
	    -exec cp -- "{}" sublime-syntaxes/syntaxes/ \;
	cargo run --example build-syntect-pack

install:
	install -v -d "$(DESTDIR)$(PREFIX)/bin"
	install -v -t "$(DESTDIR)$(PREFIX)/bin" ./target/release/gnvim
	install -v -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"
	install -v -d "$(DESTDIR)$(PREFIX)/share/applications"
	sed -e "s|Exec=gnvim|Exec=$(DESTDIR)$(PREFIX)/bin/gnvim|" \
	    "./desktop/gnvim.desktop" \
	    >"$(DESTDIR)$(PREFIX)/share/applications/gnvim.desktop"
	install -v -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps"
	install -v -d "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps"
	cp ./desktop/gnvim_128.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/128x128/apps/gnvim.png"
	cp ./desktop/gnvim_48.png "$(DESTDIR)$(PREFIX)/share/icons/hicolor/48x48/apps/gnvim.png"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/gnvim"
