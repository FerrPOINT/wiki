# Проверка CLI Wiki

## Принятая поставка sdlc1 — 2026-10-03

Backend принят: `sha256:70231dbb52f3c0a1853f19a553314a462e7490987cbe070c6d65873d8e4ccfe2`. Проверенный code source
`191b449bed1c7dc4e594ca5681b7509b45961415`, опубликованный Base pin
`9408802dfa978cba2f67162a49adca6f65851b01`, Rust 1.88.0, locked dependencies.
Изменения main после этого source, включённые в документационный PR, не меняют
backend/CLI tree. SHA документационного merge проверяется отдельно от build source.
Предыдущий baseline image: `sha256:0c22075b1012a376405772fad6a8f67d91a5bc867d060a8376b843318dc88c1a`.

space/tree/template/document, Markdown stdin, draft/publish, history/revision/archive/move, dossier/summary/evidence, upload/download, search/pages. Новый multipart replay возвращает прежние attachment/evidence IDs; смена boundary не конфликтует, изменение файла/metadata конфликтует. Partial recovery после успешного upload и сбоя create переиспользует attachment с прежним parent key.

Установка и checksums: [CLI_INSTALL.md](CLI_INSTALL.md).
Все более ранние dated sections ниже — исторические проверки и blockers,
снятые этой поставкой, если явно не указано сохранённое ограничение.

### Wiki retention и replay

Legacy idempotency records сохраняются: старый raw multipart hash может дать
ожидаемый conflict при другой boundary. Новые canonical multipart replay и
partial recovery проходят без смены parent key; исторические записи не
переписываются. Собственные live spaces/documents архивированы. API не удаляет
attachment/evidence/revision/idempotency/audit retention rows. Focused fixture:
space `CLAB860E28`, document `01a10237-eeaa-7203-836b-eff0fc81df99`, evidence
`01a10237-f0f3-7a42-a117-01125541b03f`, attachment
`01a10237-f031-7981-b992-9e01467274d0`. Template
`01a10237-dd41-7a61-a439-b09c63d8c13b` остаётся: API не имеет delete/archive
template. Полный список остальных own retained IDs сохранён в локальном
live acceptance отчёте; fixture records не объявляются физически удалёнными.

### Сохранность, откат и границы

По отдельному решению пользователя принят новый чистый sdlc1, подготовленный
другой задачей после потери прежнего Docker/data. Утраченные старые данные и
images не восстановлены этой поставкой. Baseline отката — точные images нового
чистого стенда, сохранённые до обновления; восстановление утраченного старого
стенда не заявляется.

Свежий согласованный backup трёх БД и пяти фактических файловых volumes:
`20261003T142547-890aba`, 11 payloads с проверенными checksums. Перед ним
проверены нулевые active jobs/queue/leases; остановлены только три писателя.
Отдельная fresh-copy QA использовала backup `20261003T122738-8a92db`: restore
45/31/17 таблиц CI/CD/Task/Wiki, rows/sequences/schema/constraints/owners/grants
и hashes/ownership пяти volumes совпали. Эти volumes были пустыми на новом
baseline; сохранность файлов дополнительно подтверждена fixture upload/Git/
artifact/download сценариями, а не восстановлением потерянных старых файлов.

На копии запуск кандидатов, затем точных прежних images не изменил исходные
данные и схему. Старые images проходят 114 проверок и сохраняют четыре известные
ограничения: Task read/restore по ключу, Wiki legacy multipart replay и старое
CI/CD ограничение generic deployment только для Pulse. Поэтому откат возвращает
прежнее поведение, а не гарантирует исправленные CLI-сценарии. Неожиданных
отказов финальной репетиции нет. Первый rollout откатился из-за преждевременной
проверки Docker health `starting`; после исправления локального readiness wait
второй rollout принят. Рабочие данные не восстанавливались и ledger не правился.

Под workspace lock атомарно заменены только три image pins, затем последовательно
Task → Wiki → CI/CD через Compose `up --no-deps --no-build --pull never`.
Readiness: HTTP и Docker healthy, deadline 180 секунд на сервис. Откат: под тем же
lock вернуть три сохранённых pins и пересоздать эти сервисы в том же порядке;
не выполнять автоматический restore рабочих данных. Защищённые backups,
runtime-before и rollout manifest сохранены локально; секреты не публикуются.

Наблюдение: 900 секунд, 31 sample каждые 30 секунд, readiness 200/healthy,
0 restarts, 0 новых ERROR в обоих потоках логов. Остальные контейнеры/mounts
не менялись нашим rollout. Во время наблюдения отдельная задача обновила
admin-api/admin-web/ai-runtime sdlc2; их images сверены с её build/apply receipt,
mounts сохранены. Это отражено отдельно, глобальная неизменность sdlc2 за весь
интервал не заявляется. Наши keys/runtime fingerprints остались неизменными.

QA-кандидаты: 123 основных + 10 дополнительных проверок — PASS. Live: 123
успешные проверки и 6 focused checks — PASS. Отдельная ранняя попытка template
`type=page` получила корректный validation 400: ошибка fixture, исправлена на
поддерживаемый `release_note`; успешный повтор записан отдельно, исходный отказ
не скрыт. JSON/204, confirmations, stdin, access/validation/conflict, redaction,
transport timeout, no-clobber и cleanup проверены регрессиями и CLI-приёмкой.
Execution states задавались собственными fixtures/API; production runner и
внешний deploy не сертифицируются. Pulse — обычный тестовый repository/PR/pipeline
в CI/CD, отдельного постоянного стенда нет. Собственные PAT отозваны, QA Compose
ресурсы и временные keys удалены; принятые и прежние backend images сохранены.

## Исторические проверки

Проверено 2026-10-02 в изолированном task checkout `feat/cli-workflows`.

## Среда и обязательные gates

Ubuntu WSL, rustc 1.88.0 (6b00bc388 2025-06-23), Node 22.20.0 / pnpm 10.28.1, Python 3.12.3. Проверено после объединения с актуальным продуктовым `main`, с чистым опубликованным Services Base `c083783a37791e277db796361203884b87828a7d`, закреплённым в `.base-revision`. Принятая в `main` политика pinned Base, Cargo `--locked` и pnpm `--frozen-lockfile` сохранена. Base и исходные dirty checkout этой задачей не изменены.

Backend: fmt, workspace/all-target Clippy с `-D warnings`, workspace tests, отдельный MSRV check (`cargo check --locked --workspace --all-targets`), OpenAPI drift и release workspace — успешно. Workspace: **155 passed, 0 ignored**, 0 failed. Workspace выполнен последовательно, как предусмотрено workflow этого продукта.

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace -- --test-threads=1
cargo build --locked --release --workspace
WIKI_TEST_DATABASE_URL=postgres://.../wiki_test cargo test --locked --workspace -- --test-threads=1
```

Docs validators и существующие CI-contract tests проходят. Frontend: install с `--frozen-lockfile`, OpenAPI check/compat с `origin/main`, tests, lint и build — успешно. Дополнительно проходят неизменность generated contracts/lockfile, packed Base consumer и effective themes (dark/gray/light) на built preview. Task Tracker дополнительно typecheck; Task Tracker/Wiki — предусмотренный format check. Frontend tests этого продукта: 174.

## Регрессии и проверенные сценарии

- Real API lifecycle расширен первым запуском и повтором всей `evidence add-file` с ключом длиной 128 байт: одинаковые evidence/attachment IDs, evidence count увеличивается только на один, скачанное содержимое совпадает. Изменение файла или metadata с прежним key возвращает JSON conflict 409. До исправления второй этап первой команды возвращал conflict.
- HTTP fixture проверяет восстановление после upload success/evidence failure: отдельные процессы повторяют те же stage keys, upload переиспользуется. Проверены SHA-256/trim/длина child keys и отказ до upload для недопустимого parent key.
- Самостоятельные document create/attachment upload передают явный ключ без изменения и сохраняют replay. Генерация ключей без аргумента не меняется; автоматических write retries нет.
- Workspace запускается последовательно с отдельной PostgreSQL 16 БД и явным WIKI_TEST_DATABASE_URL: все 16 PostgreSQL tests выполняются. CLI real API использует production handlers с memory backend; persistence/replay/access/FTS/pages проверяются отдельным PostgreSQL suite.

Существующие проверки file/stdin, pages, JSON/204, access/validation/conflict, transport timeout, credential redaction и download no-clobber сохраняются и проходят. Новые regressions воспроизвели замечания на исходной ветке, затем прошли после исправлений.

## Границы подтверждения

Проверки используют только fixture данные и собственные временные ресурсы. Постоянные Compose-группы, runtime images, volumes и production deployment не менялись. Windows native linking недоступен (`link.exe`); Rust gates выполнены в WSL. Новых endpoint, миграций или изменений Services Base нет. Эти проверки были выполнены до merge; статус main и живая приёмка приведены ниже. Deploy не выполнялся.

Описание команд, configuration, input/output/errors и ограничения: [CLI.md](CLI.md).

## После merge и приёмка sdlc1 — 2026-10-02

Merge commit: `4a85f6a7c1de82e0869528788710da258c355ffe`. [Post-merge CI](https://github.com/FerrPOINT/wiki/actions/runs/37000178443): docs/backend/frontend/minimum-rust — 4/4 success на этом точном SHA; содержимое merge tree совпало с reviewed head. CLI release binary собран отдельно с закреплённым Base; checksum и установка: [CLI_INSTALL.md](CLI_INSTALL.md).

На собственном space выполнены tree, применение существующего template, document create со stdin и replay, draft/publish, history/revision, stale base conflict, move, task/phase summaries и dossiers, link/file evidence, standalone upload, metadata/download/no-clobber, evidence filters и cursor continuation, document/evidence search. Первый evidence add-file с ключом 128 байт проходит; повтор с тем же файлом/metadata/key получает HTTP 409 на upload. Изменение файла также получает conflict. Это блокирует replay на принятом runtime image, хотя соответствующие main/CI regression tests зелёные. Read-only PAT получает 403. Собственные документы и space архивированы. Attachment/evidence, idempotency и audit rows сохраняются согласно текущему API: permanent deletion в пакет не входит. Новые глобальные templates не создавались.

Проверка выполнялась с временными Central Auth PAT, ограниченными тремя продуктами; read/write и read-only tokens отозваны после прогона. Значения tokens/credentials не сохранялись в логах или артефактах. Исходные dirty checkout, постоянные Compose-группы, runtime images/pins и volumes не изменялись. Fixtures использовали реальные API и PostgreSQL работающего sdlc1, но только собственные project/repository/space и файлы. Ошибки исправленного smoke (имя флага search, when: manual, начальный deployment status и вывод terminal wait) отделены от воспроизведённых отказов runtime.

**Статус:** CLI main/CI проверен; полная совместимость с текущим sdlc1 не принята. Требуется отдельная сверка/обновление backend до согласованных main-кандидатов и повтор блокирующих операций. Публичный release не объявляется готовым.

## Подготовка backend-кандидатов — 2026-10-02

Task checkout обновлён merge актуального опубликованного main без переписывания истории. Текущий продуктовый pin Base: `9408802dfa978cba2f67162a49adca6f65851b01`; он отличается от Base предыдущей CLI-поставки. Чистый Base checkout проверен через verify_base_revision. Предыдущие результаты не подменяют новую проверку этого pin.

Повторно выполнены 5 documentation regression tests и штатный documentation validator — PASS; git diff --check — PASS. Новая проверка backend/frontend, PostgreSQL fixtures, сборка образов, restore rehearsal и live acceptance не завершены: C: заполнен, Docker containers API возвращает HTTP 500, затем WSL стал возвращать E_UNEXPECTED. Попытка локального PostgreSQL старта завершилась с exit 1 без подтверждённого запуска и без выполненных cargo gates.

Runtime pins не записывались, образы не заменялись, миграции и restore не запускались, постоянные сервисы не перезапускались. Новых PAT и API fixtures не создавалось. Исправность текущего runtime и очистку временного WSL build root нужно подтвердить после восстановления окружения. Эта попытка не устранила ранее описанные live-блокеры и не подтверждает готовность новой поставки.

## Возобновлённая проверка кандидатов — 2026-10-02

Проверены опубликованный main `1ae6118fa26141b2572c363de59c88f8b4fa3866` и замороженный task source `35d97823da3cd387bf7d1c304449a92c117af74d`; backend tree task source совпадает с main. Все продукты используют чистый опубликованный Base `9408802dfa978cba2f67162a49adca6f65851b01`. Операционные инструменты backup/restore сохранены отдельным snapshot локального Base с hashes/status: они не представлены как чистый опубликованный Base.

Rust 1.88.0: fmt, minimum Rust (locked workspace/all-targets), Clippy с `-D warnings`, workspace tests, OpenAPI drift и locked release workspace build — PASS. 155 workspace tests, без ignored; последовательный запуск включает PostgreSQL fixture. Node 22.20.0 / pnpm 10.28.1: frozen install, OpenAPI check/compat, frontend tests (174), lint/build, generated-contract/lockfile check, packed Base consumer и effective themes — PASS. Task Tracker typecheck и предусмотренный format check Task Tracker/Wiki — PASS. Первоначальный Wiki format failure локализован в CRLF export; canonical LF export проходит без изменения исходников.

Собран продуктовый Dockerfile с Rust 1.88.0 и locked dependencies из canonical LF source; image ID `sha256:70231dbb52f3c0a1853f19a553314a462e7490987cbe070c6d65873d8e4ccfe2`. Branding builder не применялся. Штатный пользователь, бинарник, runtime libraries, migrations (где поставляются файлами) и доступность собственного uploads/storage каталога проверены. Runtime pins не изменялись.

Template apply/document/draft/publish/revision/archive/move, dossiers/summaries/evidence, attachments/search/pagination проходят на восстановленной PostgreSQL-копии. Повтор add-file с parent key длиной 128 байт возвращает прежние evidence/attachment IDs; новый multipart boundary не вызывает conflict. Изменение файла/metadata даёт 409. После принудительного отказа создания evidence на QA proxy повтор переиспользует attachment и завершает создание; недопустимый key отклоняется до upload.

Общий изолированный CLI-прогон: 123/123 PASS; дополнительный прогон custom-field values и Wiki partial recovery: 10/10 PASS, включая повторные auth checks. Использованы временные QA identity/PAT и отдельный native Auth fixture из чистого pin Base; PAT отозваны. Backends работали в internal Docker network, без Docker socket, runner и внешних deploy targets. Human/JSON confirmations, пустые ответы, stdin, доступ/валидация/conflict, downloads/no-clobber проверены в QA и регрессиях. Оба порядка HTTP/wait timeout, задержки headers/body, redaction и cleanup download temp files проходят subprocess/HTTP tests.

Scoped restore исторического backup от 2026-10-01: три БД (Task Tracker 31, CI/CD 44, Wiki 17 таблиц), rows/sequences/constraints/owners/grants и четыре файловых volumes совпали с evidence; hashes и права 173 файлов совпали. После старта Task Tracker/Wiki данные и схема не изменились; все 8 SQLx checksums Wiki совпали с canonical source. Это не свежая согласованная копия текущего sdlc1.

**Блокеры rollout:** прежние постоянные runtime/images/volumes отсутствовали в Docker после восстановления окружения; их восстановление не входит в этот CLI rollout. Историческая CI/CD БД содержит применённые миграции 36/37, которых нет в опубликованном main (main содержит 1–35). Кандидат отказывается запускаться с VersionMissing(36). Существующий локальный commit `3aa12a4cdf077db720ef1e2e4c715db7fa443620` содержит эти файлы, но не включён в кандидат. Ledger миграций и данные не удалялись и не переписывались. Требуется отдельный план согласования опубликованного кода со схемой, свежий backup и проверенный rollback на прежние image IDs.

Исторические Wiki idempotency records не переписывались. Новый multipart hash подтверждён для записей после обновления; replay старых records с прежним raw multipart hash не сертифицирован, автоматической замены ключа нет. Wiki QA документы/space архивированы через API; до удаления временной QA-копии attachments/evidence/audit/replay retention rows оставались в ней. После проверки удалены только собственные QA ресурсы: 8 контейнеров, 7 volumes и 2 сети. Исходный protected backup и candidate images сохранены; исходные retention records не переписывались. Production pins, signing keys и volumes не изменялись.

**Статус:** исходники и три CLI-кандидата проверены; обновление sdlc1, прежние image rollback, живая приёмка обновлённого стенда и 15-минутное наблюдение не выполнены. QA CI/CD на пустой БД не заменяет приёмку сохранённых данных. Native Windows/macOS/musl, version bumps, tags и публичный release не входят в поставку.
