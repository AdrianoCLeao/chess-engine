# Chess Engine — local development helpers (no production deploy targets).
# Requires: Rust (stable), Node.js/npm, and Tauri OS prerequisites.
# https://v2.tauri.app/start/prerequisites/

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
FRONTEND := $(ROOT)/frontend
TAURI := $(ROOT)/src-tauri
CARGO := cargo

.PHONY: help dev frontend check test test-engine icons deps clean typecheck

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "\nChess Engine — commands\n\n" } /^[a-zA-Z_-]+:.*?##/ { printf "  make %-14s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@printf "\n"

deps: ## Install npm deps (root CLI + frontend)
	npm --prefix $(ROOT) install
	npm --prefix $(FRONTEND) install
	@echo "Dependencies ready."

dev: deps ## Launch the desktop app (Tauri + Vite)
	cd $(ROOT) && npx tauri dev

frontend: ## Run only the Vite UI (no Rust / no Tauri window)
	npm --prefix $(FRONTEND) install
	npm --prefix $(FRONTEND) run dev

check: ## Typecheck the Rust engine / Tauri backend
	cd $(TAURI) && $(CARGO) check

test: ## Run Rust unit tests
	cd $(TAURI) && $(CARGO) test --lib -- --nocapture

test-engine: test ## Alias for test

typecheck: ## TypeScript check for the GUI
	npm --prefix $(FRONTEND) install
	npm --prefix $(FRONTEND) run typecheck

icons: ## Regenerate tray/dock icons from src-tauri/icons/icon.png
	@cd $(TAURI)/icons && \
		sips -z 32 32 icon.png --out 32x32.png >/dev/null && \
		sips -z 128 128 icon.png --out 128x128.png >/dev/null && \
		sips -z 256 256 icon.png --out '128x128@2x.png' >/dev/null && \
		sips -s format icns icon.png --out icon.icns >/dev/null && \
		cp icon.png icon.ico && \
		echo "Icons refreshed in src-tauri/icons/"

clean: ## Remove build artifacts
	rm -rf $(FRONTEND)/dist $(FRONTEND)/node_modules $(ROOT)/node_modules $(TAURI)/target $(ROOT)/target
	@echo "Cleaned."
