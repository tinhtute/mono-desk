.DEFAULT_GOAL := help
CARGO ?= cargo

# Linker shim for missing -dev packages (libxkbcommon-x11, libX11-xcb).
# Only applied while the shim dir exists — remove after `make deps`.
SHIM_DIR ?= $(HOME)/.local/lib/dev-shim
ifneq ($(wildcard $(SHIM_DIR)),)
export RUSTFLAGS += -L $(SHIM_DIR)
endif

.PHONY: help run build release run-release check test lint fmt fmt-check clean deps shim

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
