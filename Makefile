ifeq ($(PREFIX),)
    PREFIX := /usr/local
endif

build:
	cargo build --release

install:
	install -v -d "$(DESTDIR)$(PREFIX)/bin"
	install -v -t "$(DESTDIR)$(PREFIX)/bin" ./target/release/gnvim
	install -v -d "$(DESTDIR)$(PREFIX)/share/gnvim"
	cp -r ./runtime "$(DESTDIR)$(PREFIX)/share/gnvim"

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/gnvim"
