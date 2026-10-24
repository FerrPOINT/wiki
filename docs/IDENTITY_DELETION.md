# Wiki как владелец ссылок identity

Статус: для точного исходного кандидата `96260109f03bc6babfda92bd3c6772208c9f770f`
пройдены pinned Rust 1.88 fmt/clippy, unfiltered library tests (98 PASS, 29 ignored),
явные owner PostgreSQL tests (10 PASS, 0 ignored) и named server release.
Счётчики library и явного PostgreSQL run могут пересекаться и не складываются;
ignored behavior не квалифицирован. Отдельный genuine C9 Auth/retained Wiki image
curl-run подтвердил семь private HTTP cases: no-bearer 401, wrong-scope 403,
check, freeze, exact same-operation readback, release и late-freeze 409.
Receipt SHA-256: `117a7c6aab49bbfe9737f108c317d09bd81984cc08b9890bfb904ba875d73a92`;
source/process cleanup и guards четырёх постоянных групп пройдены. Это не
полный cohort восьми owners, restart/restore или Kubernetes: они остаются открыты.
Установленный runtime не изменён; эти результаты не означают завершённый rollout.
Согласованный Base protocol: `pdlc_shared::identity_lifecycle`, SDK candidate
`ed61ec63b58fe9b60d8669c85de5ac6b5bec872b`; SDK pins обновлены отдельно от UI.

## Private API и допуск

`POST /api/internal/v1/user-deletions/{operation_id}/check|freeze|release` принимает
`DeletionRequest` schema v1. Path/body UUID должны совпадать. Отказ и успех имеют
`Cache-Control: no-store`; тело ограничено 16 KiB. Routes подключены вне обычного
Wiki bearer/JIT/admission middleware и обращаются через `WikiBackendPort`.
Memory backend не подменяет реальный storage и не поддерживает этот контракт.

Private machine factory требует fresh central verification, `service_account`
из identity owner, PAT без browser SID, exact `wiki:identity-lifecycle`, точный
issuer и зарегистрированный UUID из `WIKI_IDENTITY_DELETION_COORDINATOR_SUBJECTS`.
Human admin, обычный PAT и `wiki:write` не являются таким правом.
`WIKI_AUTH__CENTRAL_ISSUER` должен совпадать с request.identity.issuer.
Исходный verified deadline сохраняется через storage I/O, окно не продлевается
при ожидании lock или повторе. Coordinator bearer не хранится в operation JSON.
Типизированное исходное окно передаётся до storage без преобразования в boolean.
До owner locks транзакция получает PostgreSQL clock anchor и после ответа измеряет
остаток исходного monotonic deadline. Минимум этого фиксированного предела и
исходного wall expiry привязывается к существующему deferred COMMIT clock.
Истечение во время самого COMMIT откатывает freeze/release целиком; фиктивный
human context для private machine запроса не создаётся.

## Привязка прежней истории

Оператор явно задаёт `WIKI_IDENTITY_DELETION_LEGACY_ISSUER` после проверки,
какому Auth issuer принадлежит существующее хранилище. Первый startup assertion
сохраняется неизменно; смена issuer запрещена. Без привязки owner возвращает
unknown/unavailable, а не ноль ссылок. Docker/Compose и Kubernetes передают ту же
защищённую deployment-конфигурацию; переход между ними не переопределяет binding.

Ledger `wiki_identity_aliases` различает `kind=local` без subject/issuer и
`kind=central` с фактическим `users.central_sub`. Локальный bootstrap UUID не
считается центральным subject даже при случайном совпадении строк. Email,
username, display name и Markdown не служат основаниями связывания.

Ledger заполняется по существующим профилям и сохраняется после их удаления.
Поддерживаемый явный переход `central_sub: NULL → доказанный subject` сохраняет
локальный UUID и все его прежние ссылки. В той же guarded транзакции сначала
проверяется fence нового центрального subject, затем атомарно переводится alias
из local в central. Изменять или очищать уже центральное соответствие нельзя;
UUID профиля неизменяем. После удаления native-профиля этот UUID нельзя
повторно вставить и приписать его историю другой identity. Существующий local
профиль может быть явно связан с доказанным subject; это UPDATE прежнего профиля.
Переименование не меняет identity.

Отсутствующий alias для реально существующей user-ссылки, например старого audit
entity после удалённого профиля, означает неизвестное происхождение и требует
доказанного operator backfill. Неполный central alias без issuer также остаётся
unknown. Такие строки не превращаются в ноль; на чистых/полностью сопоставленных
данных check работает, а известные local-only профили не блокируют удаление
постороннего центрального пользователя.

## Полный inventory

- Локальные профили `users` и прежние `auth_sessions`.
- Владение spaces/documents и `space_members`.
- Авторы `document_revisions`, `document_drafts`.
- Создатели document task/phase links, `managed_task_document_links` и evidence.
- Загрузившие attachments, включая staged uploads.
- `audit_log.actor_id` и target `entity_id`, когда `entity_type=user`.
- `idempotency_records.actor_id`, включая незавершённые команды.
- `wiki_space_projects.created_by`.
- `wiki_confirmed_global_roles.user_id` и `granted_by`.
- Typed commands/receipts в namespace bindings, project projection sets,
  space-owner operations и project-projection operations; в том числе точные
  owner issuer/subject, responsible subject и поддерживаемые local UUID keys.

Document content, произвольный evidence JSON, настройки и email-текст не
сканируются как identity. Считаются строки со ссылкой, а не количество полей
одной строки. Duplicate references не уменьшаются до «нет участия».

## Fences и восстановление

Аддитивная migration `202610220003_identity_deletion_fences.up.sql` вводит общий
statement guard на всех перечисленных таблицах. Legacy SQL/repository/worker
writers получают shared transaction lock и epoch-row lock; freeze/release —
соответствующий exclusive lock. Счётчик epoch делает устаревший repeatable-read
snapshot serialization failure, а не stale чтением fence. Lock удерживается до
commit; предварительный SELECT, admission cache или JIT не заменяют guard.

Freeze drains прежних writers, сохраняет durable operation и фактический count.
Повтор возвращает прежний receipt; другой payload с operation UUID — конфликт.
Ненулевой count сохраняет freeze до явной отмены. Release сохраняет отмену даже
при потерянном freeze ACK; поздний freeze той же operation запрещён. Другая
operation не снимает существующий fence. Автоматического thaw по времени нет.
Записи истории и alias ledger не каскадируются cleanup этой операции.

## Проверки кандидата

```text
cargo test --locked -p infra identity_deletion_tests -- --ignored --test-threads=1
cargo test --locked -p api identity_deletion -- --test-threads=1
```

Требуется явная disposable `WIKI_TEST_DATABASE_URL` с именем БД `*_test`.
Тесты создают собственную schema и удаляют только её после завершения/ошибки.
Compose checks выполняются Base ComposeHelper v2 в отдельном временном проекте
с проверкой свободных 30 GiB и finally cleanup. Обычный ignored test listing,
локальный rustfmt или authored tests не доказывают PostgreSQL PASS.
Добавленный сценарий задерживает deferred COMMIT отдельным advisory barrier для
freeze и release: доказывает достижение COMMIT, истечение исходного окна, откат
journal transition и отсутствие временных clock rows. Его выполнение требует
отдельного реального PostgreSQL gate; написанный тест не означает PASS.

Проверяются populated upgrade, immutable binding/aliases, отсутствие email
linking, каждый reference table через настоящий raw SQL writer, local→central linking до/после fence, writer drain,
repeatable-read epoch, zero-reference/replay/conflict, release-before-late-freeze,
фактическая orphan history и machine/HTTP rejection. Genuine coordinator credential выдавался только в fresh disposable QA Auth DB
для этого семисценарного curl-run. Совместное завершение удаления с восемью
owners остаётся отдельным сквозным gate; эта Wiki проверка его не заменяет.

Owner transactions set local lock and statement timeouts to 8 seconds before acquiring the reference barrier. The private HTTP boundary bounds the entire call to 9 seconds and returns a static, non-cacheable 503 when it expires. A missing response remains an unknown outcome; callers read back or replay the same operation rather than infer that no fence was saved.

The retained proven central alias is itself counted as a reference, so removing a projection cannot erase its historical binding. Manual alias backfills take the writer barrier and enforce the same frozen-subject check. Native aliases remain noncentral and do not count toward a central identity.

Explicit typed issuer fields must be nonblank JSON strings. Numeric, null, blank, object and boolean issuer fields are unknown and cannot fall back to the deployment issuer. Typed central subject values must also be strings; malformed non-null values become unknown. Recursion is bounded to32levels and overflow is unknown. These raw storage guard cases are covered by explicit PostgreSQL regressions.
