RUST_DIR := src/rust
SCRIPTS_DIR := Scripts
GENERATED_DIR := src/swift/Ferrite/Generated

# CI overrides (e.g. make release CI=1)
CI ?= 0
VERSION ?= $(shell git describe --tags --always 2>/dev/null || echo "dev")

# Code signing flags — disabled in CI
ifeq ($(CI),1)
CODESIGN_FLAGS := CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO CODE_SIGNING_ALLOWED=NO
RUST_BUILD_FLAGS := --skip-checks
else
CODESIGN_FLAGS :=
RUST_BUILD_FLAGS :=
endif

.PHONY: all build-rust generate-bindings xcode release archive dmg clean open

all: generate-bindings

build-rust:
	@echo "Building Rust crates..."
	$(SCRIPTS_DIR)/build-rust.sh --skip-checks

generate-bindings:
	@echo "Building Rust + generating Swift bindings..."
	$(SCRIPTS_DIR)/build-rust.sh $(RUST_BUILD_FLAGS)

xcode:
	@echo "Regenerating Xcode project..."
	xcodegen generate

release: all xcode
	@echo "Building release app..."
	set -o pipefail && xcodebuild \
		-project Ferrite.xcodeproj \
		-scheme Ferrite \
		-configuration Release \
		-archivePath build/Ferrite.xcarchive \
		archive \
		ARCHS=arm64 \
		$(CODESIGN_FLAGS)

archive: release

dmg: release
	@echo "Creating DMG..."
	@rm -rf build/dmg build/Ferrite-*.dmg build/Ferrite.dmg
	@mkdir -p build/dmg
	@cp -R build/Ferrite.xcarchive/Products/Applications/Ferrite.app build/dmg/
	@strip -x build/dmg/Ferrite.app/Contents/MacOS/Ferrite
	@codesign --force --deep --sign - build/dmg/Ferrite.app
	@xattr -cr build/dmg/Ferrite.app
	@ln -s /Applications build/dmg/Applications
	@hdiutil create \
		-volname "Ferrite $(VERSION)" \
		-srcfolder build/dmg \
		-ov \
		-format UDZO \
		"build/Ferrite-$(VERSION).dmg"
	@rm -rf build/dmg
	@echo "DMG created at build/Ferrite-$(VERSION).dmg"

clean:
	cd $(RUST_DIR) && cargo clean
	rm -rf $(GENERATED_DIR)
	rm -rf build .build DerivedData

open: all
	open Ferrite.xcodeproj