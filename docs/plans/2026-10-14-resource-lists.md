# Общие списки ресурсов Base

Утверждённый объём: одинаковые отступы, разделители, hover/focus обычных списков; продукт сохраняет данные, раскладку, маршруты и действия. Backend/skills pins не меняются.

## Контракт

Боковой отступ — 16 px. Normal: вертикальный 8 px, минимум 48 px; comfortable: 16 px, минимум 56 px. Высота не фиксируется. Семантика нативная; контейнер с несколькими действиями не становится кнопкой. CSS Base используется React и штатным SSR-экспортом.

## Покрытие

| Участок | Результат | Основание |
|---|---|---|
| `frontend/src/pages/audit-log/index.tsx:480` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/dashboard/index.tsx:252` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/dashboard/index.tsx:300` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/evidence/index.tsx:707` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/phase-dossier/index.tsx:169` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/phase-dossier/index.tsx:392` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/phase-dossier/index.tsx:423` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/settings/index.tsx:105` | Исключено | Описание свойств, не коллекция ресурсов |
| `frontend/src/pages/spaces/index.tsx:473` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/task-dossier/index.tsx:179` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/task-dossier/index.tsx:413` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/task-dossier/index.tsx:444` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/task-dossier/index.tsx:468` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/templates/index.tsx:430` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/users/index.tsx:174` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/wiki-search/index.tsx:438` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/workspaces/index.tsx:36` | Перенесено | Обычный список ресурсов или активности |
| `frontend/src/pages/managed-task/index.tsx:59` | Перенесено | Связанные ресурсы или список сессий |

Сайдбары, document trees, маркированные пояснения/ошибки, формы, редакторы, таблицы и схемы исключены по назначению, а не по совпадению CSS-класса.

## Приёмка

Компонентные тесты и Chromium contract: семантика/действия, 16 px, grid, темы, длинный текст, 375/1281/1920 px, native disclosure. Затем продуктовые тесты, lint, typecheck/build, packed SDK и SSR drift. Установленная PDLC1 проверяется через IAB; происхождение images фиксируется отдельно от source/CI. Изменённые сервисы обновляются через private bundle с rollback images.
