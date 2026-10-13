# Единая проектная навигация

Поручение владельца от 2026-10-12: Tracker, Forge и Wiki показывают одинаковые
сквозные проекты. Tracker владеет названием/ключом, Admin — Namespace, Wiki —
содержимым и доступом. Base предоставляет presentation, scope и disclosure state.

## Порядок

1. Сохранить актуальные Namespace PR и исправления доступа/пагинации.
2. Добавить атомарную локальную проекцию machine reader каталога Tracker.
3. Добавить каталог, project routes и подтверждённый контекст документа.
4. Разрешать Wiki Space до монтирования страниц; исключить соседний fallback.
5. Добавить одинаковый desktop/mobile сайдбар и ленивое дерево документов.
6. Проверить совместимый source/image комплект и установить его в pdlc1.

## Приёмка

Одинаковые имена, 101+ проект, неподключённый/архивный проект, all/selected scope,
reload/Back/Forward, прямые ссылки, NS/Space conflict, ленивое дерево и folds.
Backend: атомарность снимка, stale/empty, права, generation и сохранение истории.
Поставка: generated API, frozen deps, frontend/PostgreSQL gates, backup/restore,
совместимый rollback, exact images/served hashes и IAB на 375/1920/2560.

История и common/соседние installations сохраняются. Namespace UI выключается
без изменения legacy routes. PR, CI, установленные образы и live acceptance
подтверждаются отдельно в операторских evidence; план не заменяет эти проверки.
