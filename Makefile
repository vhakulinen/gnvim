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

uninstall:
	rm "$(DESTDIR)$(PREFIX)/bin/gnvim"
	rm "$(DESTDIR)$(PREFIX)/share/gnvim"
