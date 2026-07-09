# ntfy-zero-inbox — сборка одной командой.
#
#   make run      — сгенерировать проект, собрать и запустить (самое частое)
#   make          — то же самое (алиас на run)
#   make deps     — поставить XcodeGen, если его нет
#   make generate — только сгенерировать .xcodeproj из project.yml
#   make build    — только собрать
#   make open     — открыть проект в Xcode
#   make clean    — удалить сборку и сгенерированный проект
#
# Team ID можно не вписывать в project.yml, а передать разово:
#   make run TEAM=ABCDE12345

PROJECT   := NtfyZeroInbox.xcodeproj
SCHEME    := NtfyZeroInbox
TARGET    := NtfyZeroInbox
CONFIG    := Debug
DERIVED   := build
APP       := $(DERIVED)/Build/Products/$(CONFIG)/$(TARGET).app

# Пусто → берётся DEVELOPMENT_TEAM из project.yml. Иначе переопределяем.
TEAM ?=
ifneq ($(strip $(TEAM)),)
  TEAM_FLAG := DEVELOPMENT_TEAM=$(TEAM)
else
  TEAM_FLAG :=
endif

.PHONY: all run deps generate build open clean teams

all: run

# Показать доступные Team ID (10 символов в скобках).
teams:
	@echo "→ Твои signing identities:"
	@security find-identity -v -p codesigning | grep -Eo '\([A-Z0-9]{10}\)' | tr -d '()' | sort -u || true
	@echo "Если пусто — Xcode → Settings → Accounts → твой Apple ID → Team ID."

deps:
	@command -v xcodegen >/dev/null 2>&1 || { \
		echo "→ Ставлю XcodeGen через Homebrew…"; \
		brew install xcodegen; }

generate: deps
	@echo "→ Генерирую $(PROJECT) из project.yml…"
	@xcodegen generate

build: generate
	@echo "→ Собираю $(TARGET) ($(CONFIG))…"
	@xcodebuild \
		-project $(PROJECT) \
		-scheme $(SCHEME) \
		-configuration $(CONFIG) \
		-derivedDataPath $(DERIVED) \
		-allowProvisioningUpdates \
		$(TEAM_FLAG) \
		build

run: build
	@echo "→ Запускаю $(APP)…"
	@open "$(APP)"
	@echo "Иконка должна появиться в menu bar. Настройки — ⌘, из меню приложения."

open: generate
	@open $(PROJECT)

clean:
	@echo "→ Чищу сборку и сгенерированный проект…"
	@rm -rf $(DERIVED) $(PROJECT)
	@echo "Готово."
