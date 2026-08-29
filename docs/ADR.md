# Architecture Decision Records — Wiki

## 1. Overview

ADR фиксируют ключевые архитектурные решения: контекст, альтернативы, выбор, последствия. Хранятся в `docs/adr/`.

## 2. Format

Каждый ADR — файл `NNNN-title.md`:

```markdown
# ADR-0001: Title

## Status

Proposed / Accepted / Deprecated / Superseded by ADR-NNNN

## Context

What problem are we solving?

## Decision

What did we decide?

## Consequences

Positive and negative.
```

## 3. Active ADRs

| ID | Title | Status | File |
|----|-------|--------|---|
| ADR-0001 | Rust + Axum + SQLx for backend | Accepted | `adr/0001-rust-axum-sqlx.md` |
| ADR-0002 | React + Vite + Tailwind for frontend | Accepted | `adr/0002-react-vite-tailwind.md` |
| ADR-0003 | Manual review and phase transitions | Deferred | `adr/0003-manual-job-transitions.md` |
| ADR-0004 | PostgreSQL as primary data store | Accepted | `adr/0004-postgresql-only.md` |
| ADR-0005 | Workspace layered architecture | Accepted | `adr/0005-workspace-layered-architecture.md` |
| ADR-0006 | PostgreSQL outbox for domain events | Deferred | `adr/0006-postgresql-outbox.md` |
| ADR-0007 | Runner security boundary | Deferred | `adr/0007-runner-security-boundary.md` |
| ADR-0008 | Versioned SQLx migrations | Accepted | `adr/0008-versioned-sqlx-migrations.md` |
| ADR-0009 | Canonical registry and source priority | Accepted | `adr/0009-canonical-registry.md` |
| ADR-0010 | Background jobs are deferred | Deferred | `adr/0010-apalis.md` |

## 4. Creating New ADRs

1. Взять следующий номер.
2. Создать `docs/adr/NNNN-title.md`.
3. Обновить index в этом файле.
4. Открыть PR.

## 5. Superseding

Если решение меняется:

1. Новый ADR со статусом `Accepted`.
2. Старый ADR меняет статус на `Superseded by ADR-NNNN`.

## 6. Principles

- One significant decision — one ADR.
- Keep ADRs concise (1-2 pages).
- Link to related docs.
- Russian or English? English for consistency with code/docs codebase.
## References

- `docs/ARCHITECTURE.md`
- `CONTRIBUTING.md` (корень репозитория)
