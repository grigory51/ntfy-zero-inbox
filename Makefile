# ntfy-zero-inbox (Tauri v2 + Svelte 5)
#
#   make setup   — поставить JS-зависимости (Rust соберётся при первом dev/build)
#   make dev     — запустить приложение в дев-режиме (hot reload фронта)
#   make build   — собрать релизный бандл
#   make icons   — перегенерировать иконки из src-tauri/icons/icon.png
#   make check   — типы Svelte/TS

.PHONY: setup dev build icons check

setup:
	npm install

dev: setup
	npm run tauri dev

build: setup
	npm run tauri build

icons:
	npm run tauri icon src-tauri/icons/icon.png

check:
	npm run check
