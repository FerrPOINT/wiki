# Общий Header Wiki

## Границы

Wiki использует опубликованный в Base общий `PlatformHeader` выше sidebar-offset:
leading (drawer и platform mark), services (runtime Wiki), actions (создание,
тема, аккаунт). Sidebar начинается под Header 60 px и содержит только навигацию.
Повторные бренд и подпись «База знаний» удалены. PageFrame и detail rail не меняются.

На desktop «Новый документ» остаётся в Header; на телефоне это первая команда
drawer. Brand скрывается только ниже 360 px; доступное имя Wiki сохраняется в
переключателе. Перенос создания в drawer сохраняет действие без переполнения
320 px и без уменьшения touch targets. Drawer закрывается после перехода,
Escape и при переходе на desktop; media listener удаляется при unmount.

На обзоре, пространствах, документе и шаблонах повторная неконтекстная команда создания
видна только на mobile, где Header её не показывает. Команды с space/template
и empty-state переходы сохраняются: они имеют собственный контекст.

Имя и отличный от него email видны только в bounded account menu. Если имени нет,
email показывается один раз; при отсутствии данных - «Пользователь».
Центральный logout использует существующий hook без изменения SSO/permissions.
Legacy admin navigation, backend/API и машинные токены не меняются.

## Проверки

- Offline frozen install, codegen, OpenAPI check/compat, lint/semantic,
  typecheck, 182 frontend tests и production build проходят локально.
- Shell regression: 21 тест; geometry проверяется браузером, не jsdom.
- Новый `frontend/e2e/platform-header-live.spec.ts` использует реальные
  Central Auth/Wiki, 99 route/theme/viewport сочетаний и keyboard/touch.
  Проверяет runtime шести UI, focus return, drawer и обе команды создания,
  central logout/re-entry; не подменяет API и не создаёт документы.
- Повторный backend gate использует отдельную PostgreSQL с
  `WIKI_TEST_DATABASE_URL`; сначала проверены 76 source/manifest/migration
  файлов против checker image с нормализацией LF.

Итоговый candidate production no-mock gate завершён: 3/3, Playwright retries=0,
5,9 минуты, 99 Header + 144 основных страниц + 108 detail сочетаний.
Header/pages: ноль overflow, неожиданных scroller,
console/network errors и serious/critical axe. Четыре full-page PNG открыты
и просмотрены; [evidence](../assets/screens/2026-10-01-platform-header/README.md).
Предыдущий запуск итогового кандидата не засчитан: создание QA-фикстуры
не удалось при старте сразу после пересоздания web. Добавлены явная HTTP
readiness и статус в ошибку setup; полный Wiki gate повторён на готовом сервисе.
После merge требуется отдельный полный прогон 12-тестового SSO-файла вместе
с этими тремя Wiki-тестами на образе из `main`; его результат фиксируется
в PR comment и общем release report, а не подменяет candidate fingerprints.
Backend: 142/142, включая 16 реально выполненных PostgreSQL tests; fmt/clippy
и OpenAPI drift после LF-нормализации. Первый raw diff остановился только на
CRLF и не засчитан: весь backend gate повторён с новой временной БД. Её
контейнер и сеть удалены. Полный format check, docs 5/5 и npm audit зелёные.

Исторические README-изображения не являются доказательством нового Header. Этот компонент
не закрывает оставшиеся Fleet/Workflow Header и весь платформенный релиз.
Платные GitHub builds не запускаются; пользовательские volumes не удаляются.
