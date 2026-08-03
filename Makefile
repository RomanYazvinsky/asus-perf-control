.PHONY: all build-rust build-flutter build-artifacts source-tarball check-rpmbuild check-makepkg package-rpm package-arch package-all run clean help

PACKAGE_NAME := asus-linux-control
PACKAGE_VERSION := 0.0.1
DIST_DIR := dist
SOURCE_TARBALL := $(DIST_DIR)/$(PACKAGE_NAME)-$(PACKAGE_VERSION).tar.gz
RPMBUILD_DIR := $(DIST_DIR)/rpmbuild
ARCHBUILD_DIR := $(DIST_DIR)/arch

# Default target
all: run

# Build Rust backend
build-rust:
	@echo "Building Rust backend..."
	@cd rust_core && cargo build --release
	@echo "✓ Rust backend built"

# Build Flutter app (depends on Rust)
build-flutter: build-rust
	@echo "Building Flutter app..."
	@flutter build linux --release

# Build all local release artifacts required by native packages
build-artifacts: build-daemon build-flutter
	@echo "✓ Release artifacts ready"

# Create source archive containing the release artifacts consumed by spec/PKGBUILD
source-tarball: build-artifacts
	@echo "Creating source tarball..."
	@rm -rf "$(DIST_DIR)/source"
	@mkdir -p "$(DIST_DIR)/source/$(PACKAGE_NAME)-$(PACKAGE_VERSION)"
	@tar --exclude='./.git' --exclude='./dist' --exclude='./.dart_tool' -cf - . | tar -C "$(DIST_DIR)/source/$(PACKAGE_NAME)-$(PACKAGE_VERSION)" --strip-components=1 -xf -
	@tar -C "$(DIST_DIR)/source" -czf "$(SOURCE_TARBALL)" "$(PACKAGE_NAME)-$(PACKAGE_VERSION)"
	@echo "✓ Source tarball: $(SOURCE_TARBALL)"

check-rpmbuild:
	@command -v rpmbuild >/dev/null || { echo "rpmbuild is required to build RPM packages"; exit 127; }

check-makepkg:
	@command -v makepkg >/dev/null || { echo "makepkg is required to build pacman packages"; exit 127; }

# Build Fedora/Bazzite RPM packages
package-rpm: check-rpmbuild source-tarball
	@echo "Building RPM packages..."
	@mkdir -p "$(RPMBUILD_DIR)/BUILD" "$(RPMBUILD_DIR)/RPMS" "$(RPMBUILD_DIR)/SOURCES" "$(RPMBUILD_DIR)/SPECS" "$(RPMBUILD_DIR)/SRPMS"
	@cp "$(SOURCE_TARBALL)" "$(RPMBUILD_DIR)/SOURCES/"
	@cp "packaging/rpm/$(PACKAGE_NAME).spec" "$(RPMBUILD_DIR)/SPECS/"
	@rpmbuild -ba --define "_topdir $(abspath $(RPMBUILD_DIR))" "$(RPMBUILD_DIR)/SPECS/$(PACKAGE_NAME).spec"
	@echo "✓ RPM packages:"
	@find "$(RPMBUILD_DIR)/RPMS" "$(RPMBUILD_DIR)/SRPMS" -type f

# Build Arch/SteamOS pacman packages
package-arch: check-makepkg source-tarball
	@echo "Building pacman packages..."
	@rm -rf "$(ARCHBUILD_DIR)"
	@mkdir -p "$(ARCHBUILD_DIR)"
	@cp "$(SOURCE_TARBALL)" "$(ARCHBUILD_DIR)/"
	@cp packaging/arch/PKGBUILD packaging/arch/asus-linux-control-daemon.install packaging/arch/asus-linux-control-gui.install "$(ARCHBUILD_DIR)/"
	@cd "$(ARCHBUILD_DIR)" && makepkg -f
	@echo "✓ pacman packages:"
	@find "$(ARCHBUILD_DIR)" -maxdepth 1 -type f \( -name '*.pkg.tar.*' -o -name '*.src.tar.*' \)

# Build all native packages for supported package ecosystems
package-all: package-rpm package-arch

# Run Flutter app (depends on Rust)
run: build-rust
	@echo "Running Flutter app..."
	@flutter run -d linux

# Run in debug mode
debug: build-rust
	@flutter run -d linux --debug

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cd rust_core && cargo clean
	@flutter clean
	@rm -rf "$(DIST_DIR)"
	@echo "✓ Clean complete"

# Build Rust daemon
build-daemon:
	@echo "Building Rust daemon..."
	@cd rust_daemon && cargo build --release
	@echo "✓ Daemon built at rust_daemon/target/release/asus-control-daemon"

# Install daemon and rules
install: build-daemon
	@echo "Installing daemon and rules..."
	@cd install && sudo ./install.sh

# Help
help:
	@echo "ASUS Linux Control - Build Targets"
	@echo ""
	@echo "  make run          - Build Rust backend and run Flutter app"
	@echo "  make build-flutter- Build release version"
	@echo "  make build-artifacts - Build daemon + Flutter release bundle"
	@echo "  make package-rpm    - Build Fedora/Bazzite RPM packages"
	@echo "  make package-arch   - Build Arch/SteamOS pacman packages"
	@echo "  make package-all    - Build RPM and pacman packages"
	@echo "  make build-rust   - Build only Rust backend"
	@echo "  make build-daemon - Build Rust daemon"
	@echo "  make install      - Build and install daemon + rules"
	@echo "  make debug        - Run in debug mode"
	@echo "  make clean        - Clean all build artifacts"
	@echo "  make help         - Show this help"
