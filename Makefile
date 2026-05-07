.PHONY: build build-universal install clean release-dry-run

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
BINARY  := pginsight

# Fast dev build for the current machine
build:
	cargo build --release

# Universal macOS binary (arm64 + x86_64)
build-universal:
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	lipo -create \
		target/aarch64-apple-darwin/release/$(BINARY) \
		target/x86_64-apple-darwin/release/$(BINARY) \
		-output $(BINARY)-universal
	lipo -info $(BINARY)-universal
	@echo "\nUniversal binary: ./$(BINARY)-universal"

# Install the universal binary to /usr/local/bin
install: build-universal
	cp $(BINARY)-universal /usr/local/bin/$(BINARY)
	@echo "Installed $(BINARY) $(VERSION) → /usr/local/bin/$(BINARY)"

# Package + print checksums (dry-run for what CI will do)
release-dry-run: build-universal
	tar -czf $(BINARY)-$(VERSION)-macos-universal.tar.gz $(BINARY)-universal
	shasum -a 256 $(BINARY)-$(VERSION)-macos-universal.tar.gz
	@echo "\nTag and push to trigger the real release:"
	@echo "  git tag v$(VERSION) && git push origin v$(VERSION)"

clean:
	cargo clean
	rm -f $(BINARY)-universal $(BINARY)-*.tar.gz $(BINARY)-*.sha256
