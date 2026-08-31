# UI/UX - Wiki

## 1. Product Feel

Wiki is an operational knowledge tool. It should feel quiet, readable and efficient: document-first, dense enough for daily work, and free from marketing-style layouts.

## 2. Main Navigation

- Обзор.
- Пространства.
- Документы.
- Задачи.
- Фазы workflow.
- Материалы.
- Шаблоны.
- Поиск.
- Аудит.
- Администрирование и настройки.

## 3. Critical Screens

| Screen | Primary Goal |
|---|---|
| Обзор | Видеть последние документы и незакрытые связи |
| Пространства | Открывать дерево документов |
| Документ | Читать текущую ревизию, обновлять черновик, публиковать и видеть связанный контекст |
| Задача | Видеть все документы и материалы по задаче |
| Фаза | Видеть документы и материалы по фазе |
| Материалы | Проверять файлы и URL-доказательства |
| Шаблоны | Переиспользовать структуры документов |
| Аудит | Смотреть неизменяемую историю изменений |
| Пользователи/настройки | Управлять доступом и параметрами инстанса |
| Поиск | Быстро находить документы и материалы |

## 4. Design Rules

- Keep cards for repeated items and framed tools only.
- Use icon buttons for clear actions.
- Use compact tables/lists for operational data.
- Avoid oversized hero sections.
- Text must fit at mobile and desktop widths.
- Every icon-only action needs accessible label.

## 5. Ready Criteria

- The page route exists in `docs/ROUTING.md`, `frontend/src/app/router.tsx` and `docs/PAGE_DESIGN.md`.
- The page has a desktop screenshot in README and screenshot manifest.
- Mobile smoke covers navigation-heavy, document-reading, task-reading and search flows.
- The page shows the main object context above the fold.
- API-backed pages have clear loading, empty and error states; permission-denied states must stay explicit where access differs by role.
- The page does not expose deferred reports, notifications, webhook delivery or runner controls.

## 6. References

- `docs/PAGE_DESIGN.md`
- `docs/FRONTEND_ARCHITECTURE.md`
- `docs/ROUTING.md`
