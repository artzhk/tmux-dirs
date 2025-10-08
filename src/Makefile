BUILDDIR := ../bin

INSTALLBIN ?= $(HOME)/.local/bin

.PHONY: build clean

build: 
	@[ -d  $(BUILDDIR) ] || mkdir -p $(BUILDDIR)
	@cargo build --release --target-dir $(BUILDDIR)

clean: 
	@cargo clean
