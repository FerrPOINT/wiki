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

| Screen                 | Primary Goal                                                                        |
| ---------------------- | ----------------------------------------------------------------------------------- |
| Обзор                  | Видеть последние документы и незакрытые связи                                       |
| Пространства           | Открывать дерево документов                                                         |
| Документ               | Читать текущую ревизию, обновлять черновик, публиковать и видеть связанный контекст |
| Задача                 | Видеть все документы и материалы по задаче                                          |
| Фаза                   | Видеть документы и материалы по фазе                                                |
| Материалы              | Проверять файлы и URL-доказательства                                                |
| Шаблоны                | Переиспользовать структуры документов                                               |
| Аудит                  | Смотреть неизменяемую историю изменений                                             |
| Пользователи/настройки | Управлять доступом и просматривать runtime-политики инстанса                        |
| Поиск                  | Быстро находить документы и материалы                                               |

## 4. Design Rules

- Keep cards for repeated items and framed tools only.
- Use icon buttons for clear actions.
- Use compact tables/lists for operational data.
- Avoid oversized hero sections.
- Text must fit at mobile and desktop widths.
- Long Markdown words and links wrap; wide tables and code blocks use keyboard-accessible local scrolling instead of expanding the page.
- Every icon-only action needs accessible label.

## 5. Ready Criteria

- The page route exists in `docs/ROUTING.md`, `frontend/src/app/router.tsx` and `docs/PAGE_DESIGN.md`.
- The page has a desktop screenshot in README and screenshot manifest.
- Mobile smoke covers navigation-heavy, document-reading, task-reading and search flows.
- The page shows the main object context above the fold.
- API-backed pages have clear loading, empty and error states; permission-denied states must stay explicit where access differs by role.
- The page does not expose deferred reports, notifications, webhook delivery or runner controls.

## 6. Геометрия Detail

Документ и задача используют общий `detail-with-aside`/`page-split` из Base:
fluid primary + правый rail 320 px от 1024 px, ниже контекст следует за primary.
Текст документа использует `page-readable` внутри primary; весь документ вместе
с контекстом больше не ограничен reading-режимом. Досье фазы использует `wide`:
документы и материалы являются равноправными рабочими областями, а не sidebar.
Mapping и реальные проверки: [план](plans/2026-10-01-detail-layout-modes.md).

## 7. References

- `docs/PAGE_DESIGN.md`
- `docs/FRONTEND_ARCHITECTURE.md`
- `docs/ROUTING.md`

## 8. Общие состояния Base — source 2026-10-22

46 композиций в 15 файлах используют LoadingState, ErrorState, EmptyState и
Notice вместо локального оформления. Загрузка и недоступность страницы используют
состояния ресурса Base; inline feedback и результаты операций — Notice. Действия
повтора, открытия результата и закрытия передаются через actions. Локальные
декоративные иконки подтверждения удалены: семантикой иконки управляет Notice.
Права, запросы, namespace/space resolution, version/conflict и callbacks сохранены.
При неверной или недоступной привязке другое пространство не выбирается.

Исходный AST-реестр содержал 47 live-region мест: 43 перенесены, четыре счётчика
результатов оставлены у продукта. Дополнительно перенесены три подтверждённые
пустоты без role. Два нативных контейнера role=status вокруг EmptyState сохраняют
объявление результата; собственного оформления у них нет. Заголовки, связанные
карточки и прочие предметные формы остаются отдельной классификацией.

Scoped unit по 13 файлам: первичные прогоны 61/62 и 95/96. Гонка проверки
существующего URL воспроизведена на исходниках до переноса; теперь проверка ждёт
подтверждённого URL. Прежние min-h классы в проверке уведомления заменены проверкой
публичного size-контракта и действия закрытия. Повтор двух файлов 24/24 и финальный
navigation 13/13 прошли. Совокупность последних результатов — 158 cases,
не новый полный suite. TypeScript, scoped lint, primitive/choice и AppShell guards
прошли. Полный route gate открыт: текущий docs/ui-routes.json отсутствует;
историческая recovery-копия не является актуальной матрицей приёмки.

IAB: 72 сочетания шести сцен, light/dark/gray и 320/375/1920/2560, без overflow;
mobile buttons не ниже 44 px. Проверены длинное имя, сохранение ссылки на шаблон
и project_scope, клавиатурное закрытие уведомления. Full-page снимки сохранены
для 375/1920/2560. Один POST создал только синтетический шаблон в памяти собственного
QA-процесса. Product backend, credentials, dev/production и установленный runtime
не использовались. Архив пакета, точные pins и итоговый gate остаются открытыми.
