# Стабильные frontend types/unit при смене agent ID

Исходная версия98ac6fa уже отделяет browser setup от reviewed Node suites.
Повторно менять setup, assertions, maxWorkers4 и production graph не требуется.
Неизменившиеся types/unit не должны исполняться заново из-за другого непустого
Codex correlation ID. Общий контракт принадлежит Base:
`docs/plans/frontend-agent-presence-20261025.md` в принятом quality tooling.

1. Проверить собственных readers ID и реальные dependency semantics на Node22.20.0.
   Сохранить product Vite env, CA/proxy/options и остальные unknown inputs.
2. Добавить actual loader/projection regression в существующий lightweight
   tooling gate; получить RED до включения контракта только у types/unit.
3. Принять product CI selector exact0d8d59bd штатным packager. Проверить optional
   Rust cache/frontend helper/runtime recipe bytes; root quality и SDK/UI/
   skills/operator pins не менять. Production artifact contract не включать.
4. После source checks и commit выполнить один missing current Fast вместе
   с Fleet на согласованной frontend-вехе, только после fresh host census и
   штатного общего slot. Использовать пригодную существующую среду и store.
5. Три ready-повтора с другими IDs должны сохранить receipt/log/mtime без
   admission/install/export/tests/typecheck. Реальный VITE input отменяет reuse;
   восстановление возвращает прежние receipts. Dist не пересобирать.

Измерять wall и job CPU/private commit, не выдавая их за RSS/heap или service RAM.
Исторические proofs сохранять. Integration/slow/release/live — отдельная ручная
веха; source/probe PASS сам по себе не доказывает native compatibility.

Source qualification: новый actual loader/projection test дал RED при отсутствии
контракта, затем полный выбранный stdlib набор29PASS. Public std-env consumer
Vitest на Node22.20.0 дал одинаковые original/projected primitive exports для
absent, empty, разных ID и строки0. Собственных readers сырых ID не найдено.
Canonical package --check PASS; Rust cache/frontend helper/runtime recipe
байт-в-байт сохранены. Native Fast/reuse требует отдельного terminal evidence.
