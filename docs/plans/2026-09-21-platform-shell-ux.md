# Wiki: привести AppShell к платформенному контракту

**Статус 2026-09-21:** завершено.

## Контекст

Документированный shell contract уже требует header 60 px, sidebar 264 px и
tablet rail 72 px, но текущий `AppShell` использует 48/240 px, скрывает
навигацию на tablet, ограничивает всю рабочую область `max-w-7xl` и открывает
самодельный mobile overlay без focus trap.

## Изменение

- Реализовать 60/72/264 px shell без локального ограничения рабочей области.
- Перевести активные ссылки на `NavLink`, сохранив role-based admin routes.
- Заменить mobile overlay на Radix Dialog с focus trap, Escape и возвратом фокуса.
- Сохранить в header создание документа, service switcher, тему и аккаунт;
  довести основные controls до 40 px, mobile trigger и nav links до 44 px.
- Не менять API, auth model и содержимое страниц.

## Проверки

- Unit: role-based routes, direct nested active state, drawer focus/Escape,
  account/logout.
- Frontend: OpenAPI check/compat, format, lint, typecheck, Vitest, build.
- Playwright: постоянный keyboard scenario.
- Browser QA: 375/768/1440/2560 px, light/dark, overflow и геометрия shell.

## Результат

- Shell приведён к контракту 60/72/264 px, а ограничение ширины перенесено на
  страницу настроек, где оно действительно нужно для читаемости на ultrawide.
- Mobile navigation использует Radix Dialog: focus trap, Escape и возврат
  фокуса проверены постоянным Playwright-сценарием и ручной QA-матрицей.
- Исправлен Tailwind `@source` для `@sdlc/ui`, поэтому arbitrary-классы shared
  Radix-компонентов снова попадают в production CSS.
- Unit: 24 файла, 136 тестов; Playwright: 9 сценариев в Chromium, Firefox и
  WebKit; lint, semantic HTML, OpenAPI check/compat и production build пройдены.
- Скриншоты и машинные метрики: `docs/screenshots/2026-09-21-shell/`.
