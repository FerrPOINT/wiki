# Проверка центрального каталога и ошибок Wiki

Снимки375/1920/2560 px показывают настоящий production UI и собственный
QA-профиль из Central Auth с пустым display_name: используется непустой username.
Это каталог после проекции, а не изображение error state.

Проверено на staged Git tree `d9bdb9cbe613a38eaaf85cddeed6fdf9fd30d330` с consumer Base `9408802dfa978cba2f67162a49adca6f65851b01`:

- Actual Auth/API/PostgreSQL: named, empty и whitespace display_name;
  повторная синхронизация сохраняет UUID/updated_at, disabled — identity.
- Browser SSO, каталог и full-page снимки трёх размеров.
- Настоящий DB fault: безопасный500, русское сообщение и Retry без повторного SSO.
- Остановка собственного Auth:503 и Retry после восстановления.

Браузерные HTTP routes не подменялись. Учётки, ключ и PostgreSQL disposable;
собственный Compose удалён. Старые проверки focus/themes архивирования сохраняют
своё отдельное доказательство. Новые миграции и изменения PAT scopes отсутствуют.

184 frontend tests, frozen dependencies, typecheck/lint/format/build,
OpenAPI drift/compatibility, packed consumer и effective themes прошли.
Окончательные Rust/CI и общая поставка проверяются отдельно; эта запись
не подтверждает Pulse или три полных чистых цикла.
