# Проверка UI-кандидата Wiki — 2026-10-21

UI закреплён на merged Base #297:
`94272dc5512be58ec6c88fb7c1d1d33c7599a3e4`. SDK и skills pins, backend,
API, lockfiles, права и продуктовые обработчики не меняются.

Проверены настоящий frozen install, 252/252 frontend unit tests, ESLint,
semantic/shell checks, OpenAPI drift, TypeScript/Vite production build,
actual packed UI consumer и effective theme contrast — PASS.
После нормального объединения с актуальными main и веткой PR дополнительно
прошли 22/22 теста AppShell, полная frontend format check, scoped ESLint и
OpenAPI compatibility.

Три Chromium fixture сценария прошли без retries/skips: semantic work-area
geometry на 375/1440/1920/2560 px, active route и клавиатурное мобильное меню,
preview нового документа и защита несохранённого черновика. Full-page
screenshots сохранены в qualification evidence. Мобильный диалог выбирается
по фактическому имени `Wiki`; assertions focus trap, Escape, возврата focus и
`aria-current` не ослаблены.

Первый запуск browser harness с несовпадающими host и старым именем диалога
не был успешным. Итоговый повтор использует единый host и текущий DialogTitle.
Результаты первого запуска сохраняются отдельно и не объявляются зелёными.

Данные API/Auth синтетические. Это проверка source-кандидата и собранного UI,
а не live приёмка backend/SSO либо установка. Временный preview остановлен;
runtime, credentials и volumes не менялись.
