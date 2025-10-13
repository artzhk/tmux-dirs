BUILDDIR := ../bin

.PHONY: build clean

build: 
	@[ -d  $(BUILDDIR) ] || mkdir -p $(BUILDDIR)
	@cargo build --release --target-dir $(BUILDDIR)

clean: 
	@cargo clean
