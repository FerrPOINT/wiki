# Frontend Standards — Wiki

> Стартовый документ. До конца разработки часть соглашений может измениться — актуализировать при стабилизации frontend-реализации.

## 1. Scope

Соглашения для frontend-подсистемы Wiki: архитектурные слои, именование, работа с API, формы, состояние, стилизация, тестирование и производительность.

## 2. Архитектурные слои (Feature-Sliced Design)

Проект использует FSD-методологию. Каждый слой имеет чёткие права импорта:

| Слой | Ответственность | Может импортировать из |
|---|---|---|
| `app` | Инициализация, роутер, providers, глобальные стили | `pages`, `widgets`, `features`, `entities`, `shared` |
| `pages` | Страницы, композиция виджетов | `widgets`, `features`, `entities`, `shared` |
| `widgets` | Самостоятельные блоки интерфейса | `features`, `entities`, `shared` |
| `features` | Пользовательские сценарии | `entities`, `shared` |
| `entities` | Бизнес-сущности | `shared` |
| `shared` | Переиспользуемые примитивы, API-клиент, i18n, utils | — |

## 3. Именование и структура сегментов

Каждый слайс состоит из сегментов:

- `ui/` — компоненты.
- `model/` — бизнес-логика, stores, selectors.
- `api/` — запросы к backend.
- `lib/` — вспомогательные функции.
- `config/` — конфигурация слайса.

Пример:

```text
features/
└── publish-document/
    ├── api/
    │   └── publish-document.ts
    ├── model/
    │   ├── store.ts
    │   └── types.ts
    ├── ui/
    │   └── PublishDocumentForm.tsx
    └── index.ts
```

Публичный API слайса экспортируется только через `index.ts`.

## 4. Компоненты

- Использовать функциональные компоненты + hooks.
- Презентационные компоненты не должны знать о состоянии приложения.
- Container-компоненты подключают store и передают данные вниз.
- Props-интерфейсы именуются `{ComponentName}Props`.
- Составные компоненты (compound) оформляются как единый объект с подкомпонентами.

## 5. Работа с API

- Все API-запросы проходят через `@tanstack/react-query`.
- Ключи запросов типизированы и централизованы в `shared/api/query-keys.ts`.
- Мутации сопровождаются инвалидацией связанных query-ключей.
- Ошибки API обрабатываются глобальным обработчиком и отображаются через `sonner`.
- Retry-политика: 3 попытки при 5xx/NetworkError, 0 попыток при 4xx.

## 6. Формы

- Простые формы MVP делают validation locally in React; form/schema libraries добавляются только если сложность формы реально выросла.
- Правила валидации хранятся рядом с формой в `features/<name>/lib/validation.ts`.
- Каждая форма должна поддерживать состояния: pristine, submitting, submit error, success.
- Disabled состояние всех полей при `isSubmitting`.

## 7. Состояние

- Серверное состояние — `@tanstack/react-query`.
- Глобальное клиентское состояние — `zustand` (максимум 3-4 stores).
- Локальное состояние компонента — `useState`/`useReducer`.
- Side-effects внутри features — через кастомные hooks; новые state/effects библиотеки добавляются только отдельным решением.

## 8. Стилизация

- Tailwind CSS 4.1.0 + `@tailwindcss/vite`.
- Кастомные классы и design tokens — через `frontend/src/styles/index.css`.
- Использовать `cn()` из `shared/lib/cn.ts` для условных классов.
- Цвета, отступы, типографика — только из design tokens.
- Адаптивность: mobile-first, breakpoints `sm`, `md`, `lg`, `xl`, `2xl`.

## 9. Маршрутизация

- `react-router` 8.1.0.
- Страницы регистрируются в `app/router.tsx`.
- Lazy loading для всех страниц кроме login и dashboard.
- Layout-компоненты в `app/layouts/`.

## 10. Тестирование

- Unit/интеграционные — Vitest + React Testing Library.
- E2E — Playwright.
- Каждый feature покрывается через user-centric сценарии.
- После UI-изменений — full-page скриншоты 375 / 1920 / 2560.

## 11. Производительность

- React Compiler и `memo` только после профилирования.
- Виртуализация длинных списков — `@tanstack/react-virtual`.
- Code-splitting по страницам и крупным виджетам.
- Ассеты кэшируются через Vite PWA/service worker (при реализации).

## 12. i18n

- Локали: `ru` (default), `en`.
- Ключи именуются в kebab-case: `documents.create.title`.
- Тексты не хардкодятся в компонентах.

## 14. UI Shell Contract

Wiki follows the Base [UI Shell Standard](https://github.com/FerrPOINT/services-base/blob/main/docs/platform/UI_SHELL_STANDARD.md).
`AppLayout` owns the shared 264 px/72 px left sidebar, 60 px global header and
full-width right work area. Routes use only the common modes `wide`,
`reading/form` and `detail-with-aside`; they do not define a Wiki-local shell or
content-width scale.

- Dashboard, spaces, document catalog, search, audit and administration use
  `wide`. Dense lists stay lists/tables and retain local overflow only where
  technical data needs it.
- Document reading and editing use `reading/form`; an optional metadata/outline
  rail uses `detail-with-aside` and moves below content on narrow screens.
- Settings and focused forms use `reading/form`, with only the inner 760 px
  column bounded rather than catalog or search routes.
- The same route order and active state appear in the 264 px desktop sidebar,
  72 px compact tablet rail and mobile drawer below 768 px.
- Global controls stay in the one-row header. Page title, breadcrumbs, document
  actions and filters are page-owned rows below it.
- Shell/layout verification covers 375, 1440 and 2560 px, direct-route active
  navigation, keyboard drawer flow, header alignment and no body overflow.

Product page details remain in `docs/PAGE_DESIGN.md`; this contract preserves
Wiki's document-first visual language.

**Статус 2026-09-21:** контракт реализован в `frontend/src/widgets/app-shell.tsx`;
unit/Playwright regression и responsive evidence хранятся рядом с frontend
тестами и в `docs/screenshots/2026-09-21-shell/`.

## 15. References

- `docs/FRONTEND_ARCHITECTURE.md` — технический стек и структура.
- `docs/DESIGN_TOKENS.md` — токены дизайна.
- `docs/REACT_STYLING.md` — руководство по стилизации.
- `docs/UI_LIBRARIES.md` — выбор библиотек.
- `docs/TESTING.md` — стратегия тестирования.
- `docs/CODE_STYLE.md` — общий code style.
- `docs/API.md` — REST API.
- `docs/ERROR_HANDLING.md` — обработка ошибок.
