# ntfy-zero-inbox

Небольшое нативное macOS-приложение (menu bar + виджет), которое подписывается на
[ntfy](https://ntfy.sh) и даёт то, чего нет у готовых ntfy-клиентов: **persistent
inbox с отметкой «обработано» → zero-inbox workflow**. Уведомления от твоих
скриптов/автоматизаций складываются по полочкам (топик = категория), переживают
разрывы связи и твоё длительное отсутствие, и ничего не теряется.

## Что внутри

- **Menu bar app** (SwiftUI `MenuBarExtra`, agent-app без иконки в Dock) —
  список уведомлений, сгруппированный по топикам, с бейджем «сколько
  необработанных» прямо в статус-баре. Кнопка ✓ убирает уведомление из очереди.
- **Виджет** (WidgetKit, small + medium) — счётчик непрочитанного и последние
  сообщения на рабочем столе / в Центре уведомлений.
- **SwiftData** в общем контейнере App Group — приложение пишет, виджет читает.
- **Стриминг с реконнектом** — держит `GET /<topics>/json`, при обрыве
  переподключается с `since=<время>`, догружая пропущенное.
- **Self-hosted** — URL сервера и Bearer-токен настраиваются в UI.

## Требования

- macOS 14.0+ (собрано на текущей 26.x, минимум опущен до 14 ради запаса).
- Xcode 15+.
- [XcodeGen](https://github.com/yonyz/XcodeGen) — `brew install xcodegen`.

## Сборка

Проще всего через `make` (он сам поставит XcodeGen, сгенерит проект, соберёт и запустит):

```bash
make run TEAM=ABCDE12345   # свой Team ID: Xcode → Settings → Accounts → Team
```

Team ID можно и вписать один раз в `project.yml` (`DEVELOPMENT_TEAM`) — тогда просто `make run`.

Другие цели: `make generate` (только .xcodeproj), `make build`, `make open`
(открыть в Xcode), `make clean`. Ручной путь без make:

```bash
xcodegen generate && open NtfyZeroInbox.xcodeproj   # дальше Cmd+R
```

При первом запуске App Group `group.com.grigory.ntfy-zero-inbox` создастся
автоматически под твоим Team. Если хочешь другой bundle-id/группу — поменяй в
`project.yml` (три места: `PRODUCT_BUNDLE_IDENTIFIER`, `application-groups`) и в
`Shared/AppConfig.swift` (`appGroupID`), они должны совпадать.

## Настройка

Открой **Настройки** (⌘, из меню приложения):

- **URL сервера** — например `https://ntfy.твойдомен`.
- **Токен** — если сервер закрытый (ntfy access tokens).
- **Топики** — через запятую: `backups, ci, alerts, home`.

Нажми «Применить и переподключиться».

## Проверка

Отправь тестовое уведомление в любой из своих топиков:

```bash
curl -H "Title: Проверка" -H "Tags: white_check_mark" \
     -d "Работает" https://ntfy.твойдомен/alerts
```

Оно появится в menu bar с бейджем и в виджете.

## Известные ограничения (осознанные)

- **Виджет не держит живое соединение** — это ограничение WidgetKit. Обновляется
  по таймлайну; приложение форсит обновление через `reloadAllTimelines()` при
  каждом новом сообщении, плюс 15-мин страховочный refresh.
- **Токен лежит в UserDefaults**, а не в Keychain. Для личного использования ок;
  для раздачи другим — вынеси в Keychain (единственное место, которое стоит
  доработать по безопасности).
- Перехватить **чужие** системные уведомления macOS нельзя (запрет API). Это
  инбокс для **твоих** источников, шлющих в ntfy, — а не зеркало всего Notification Center.

## Структура

```
project.yml            — описание таргетов для XcodeGen
Shared/                — общий код app ↔ widget
  AppConfig.swift      — App Group id + фабрика SwiftData-контейнера
  InboxItem.swift      — @Model уведомления (с флагом isRead)
NtfyZeroInbox/         — menu bar приложение
  NtfyZeroInboxApp.swift
  NtfyClient.swift     — стриминг + реконнект + мутации инбокса
  NtfyMessage.swift    — декодер JSON ntfy
  SettingsStore.swift  — настройки (App Group UserDefaults)
  MenuContentView.swift
  SettingsView.swift
NtfyWidget/            — WidgetKit-расширение
  NtfyWidget.swift
```
