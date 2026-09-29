.DEFAULT_GOAL := help
CARGO ?= cargo

# Linker shim for missing -dev packages (libxkbcommon-x11, libX11-xcb).
# Only applied while the shim dir exists — remove after `make deps`.
SHIM_DIR ?= $(HOME)/.local/lib/dev-shim
ifneq ($(wildcard $(SHIM_DIR)),)
export RUSTFLAGS += -L $(SHIM_DIR)
endif

.PHONY: help run build release run-release check test lint fmt fmt-check hooks clean deps shim app open-app

APP_NAME ?= mono-desk
APP_BUNDLE ?= target/macos/$(APP_NAME).app

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

run: ## Run the app (debug)
	$(CARGO) run

build: ## Build debug binary
	$(CARGO) build

release: ## Build optimized binary
	$(CARGO) build --release

run-release: ## Run optimized binary
	$(CARGO) run --release

check: ## Type-check without codegen
	$(CARGO) check

test: ## Run tests
	$(CARGO) test

lint: ## Clippy, warnings denied
	$(CARGO) clippy --all-targets -- -D warnings

fmt: ## Format code
	$(CARGO) fmt

fmt-check: ## Check formatting (CI)
	$(CARGO) fmt --check

hooks: ## Install the pre-commit hook (fmt-check + lint)
	ln -sf ../../scripts/pre-commit .git/hooks/pre-commit

clean: ## Remove build artifacts
	$(CARGO) clean

deps: ## Install system dependencies (Ubuntu/Debian, needs sudo)
	sudo apt update
	sudo apt install -y gcc g++ clang libfontconfig-dev libwayland-dev \
		libwebkit2gtk-4.1-dev libxkbcommon-x11-dev libx11-xcb-dev \
		libssl-dev libzstd-dev vulkan-validationlayers libvulkan1
	@echo "System deps installed — $(SHIM_DIR) is no longer needed."

shim: ## Recreate linker shims for missing -dev packages (no sudo)
	mkdir -p $(SHIM_DIR)
	ln -sf /usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0 $(SHIM_DIR)/libxkbcommon-x11.so
	ln -sf /usr/lib/x86_64-linux-gnu/libX11-xcb.so.1 $(SHIM_DIR)/libX11-xcb.so
	@echo "Shims ready in $(SHIM_DIR)"

app: build ## Bundle mono-desk.app with the brand icon (macOS only)
	@test "$$(uname -s)" = "Darwin" || { echo "make app is macOS-only — use make run"; exit 1; }
	mkdir -p "$(APP_BUNDLE)/Contents/MacOS" "$(APP_BUNDLE)/Contents/Resources"
	cp target/debug/$(APP_NAME) "$(APP_BUNDLE)/Contents/MacOS/$(APP_NAME)"
	cp assets/brand/icon.icns "$(APP_BUNDLE)/Contents/Resources/icon.icns"
	printf '%s\n' \
		'<?xml version="1.0" encoding="UTF-8"?>' \
		'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
		'<plist version="1.0">' \
		'<dict>' \
		'  <key>CFBundleName</key><string>$(APP_NAME)</string>' \
		'  <key>CFBundleDisplayName</key><string>$(APP_NAME)</string>' \
		'  <key>CFBundleIdentifier</key><string>lab.monodesk.settings-demo</string>' \
		'  <key>CFBundleExecutable</key><string>$(APP_NAME)</string>' \
		'  <key>CFBundleIconFile</key><string>icon</string>' \
		'  <key>CFBundlePackageType</key><string>APPL</string>' \
		'  <key>CFBundleShortVersionString</key><string>0.1.0</string>' \
		'  <key>CFBundleVersion</key><string>0.1.0</string>' \
		'  <key>LSMinimumSystemVersion</key><string>11.0</string>' \
		'  <key>NSHighResolutionCapable</key><true/>' \
		'</dict>' \
		'</plist>' > "$(APP_BUNDLE)/Contents/Info.plist"
	plutil -lint "$(APP_BUNDLE)/Contents/Info.plist"
	@echo "Bundle ready: $(APP_BUNDLE)"
	@echo "Open it with: open $(APP_BUNDLE)   (Dock and Cmd-Tab show the kumo icon)"

open-app: app ## Build the bundle and open it (macOS only)
	open "$(APP_BUNDLE)"
