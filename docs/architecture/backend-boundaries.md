# Backend Boundaries - Wiki

## Target Crates

```text
api      -> HTTP routes and OpenAPI
app      -> use cases/services
domain   -> entities, value objects, invariants
infra    -> PostgreSQL, local storage, search
shared   -> config, errors, IDs
cli      -> HTTP client CLI
```

## Rules

- `domain` does not depend on `api` or `infra`.
- `app` depends on domain traits, not concrete repositories.
- `infra` implements repository/storage/search ports.
- `api` maps HTTP DTOs to app commands.
- `cli` talks only to public API.

## Ownership

| Layer | Owns | Must Not Own |
|---|---|---|
| `domain` | entities, value objects, invariants, state transitions | SQL, HTTP, filesystem, env parsing |
| `app` | use cases, transaction intent, authorization decisions | route parsing, database driver calls |
| `infra` | PostgreSQL repositories, local storage, search | business state transitions |
| `api` | Axum routes, DTO mapping, OpenAPI annotations | domain persistence details |
| `server` | composition root and graceful shutdown | business rules |
| `cli` | user commands and API client | direct DB access |

## Review Checklist

- A new capability has a domain type or an explicit reason why it is read-only projection.
- Application service owns the transaction boundary.
- Repository implementation returns domain errors, not transport errors.
- API layer converts validation and domain conflicts to the shared error envelope.
- Tests cover at least one success path and one boundary violation.

## Vocabulary Rule

Wiki-owned code uses Wiki vocabulary. External task/workflow systems are represented by keys, URLs and snapshots; they do not bring their internal modules, routes or persistence model into Wiki.
