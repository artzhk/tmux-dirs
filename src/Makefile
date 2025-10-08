BUILDDIR := ../bin

INSTALLBIN ?= $(HOME)/.local/bin

.PHONY: build clean

install: 
	@[ -f $(INSTALLBIN) ] || mkdir -p $(INSTALLBIN)
	@mv -v $(BUILDDIR)/release/cli $(INTALLBIN)/tmux-dirs

build: 
	@[ -d  $(BUILDDIR) ] || mkdir -p $(BUILDDIR)
	@cargo build --release --target-dir $(BUILDDIR)

clean: 
	@cargo clean
