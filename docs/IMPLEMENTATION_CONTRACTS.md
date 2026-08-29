# Implementation Contracts - Wiki

## 1. Purpose

Implementation contracts are normative rules for API, authorization, events, migrations, UI/API coupling and document format.

## 2. Contract Index

| Contract | File |
|---|---|
| API | `docs/contracts/API_CONTRACT.md` |
| Authorization | `docs/contracts/AUTHZ_CONTRACT.md` |
| Events | `docs/contracts/EVENT_CONTRACT.md` |
| Data lifecycle | `docs/contracts/DATA_LIFECYCLE.md` |
| Migrations | `docs/contracts/MIGRATION_CONTRACT.md` |
| UI/API | `docs/contracts/UI_API_CONTRACT.md` |
| Document format | `docs/contracts/DOCUMENT_FORMAT.md` |
| Integration protocol | `docs/contracts/INTEGRATION_PROTOCOL.md` |
| Workflow documentation DSL | `docs/contracts/PIPELINE_DSL.md` |
| Runner protocol | `docs/contracts/RUNNER_PROTOCOL.md` |

## 3. Rules

- A feature is not done until its contract, tests and docs are updated.
- Contracts override narrative docs when they conflict.
- Current implementation gaps must be recorded in `docs/CURRENT_STATE.md`.

## 4. Required Contract Fields

Every contract update must define:

- scope and non-scope;
- canonical entity names;
- request/response or event shape where applicable;
- idempotency rule for mutations;
- authorization expectations;
- failure modes;
- acceptance criteria.

## 5. Change Control

Contract-breaking changes require:

1. ADR or explicit contract version bump.
2. Migration or compatibility plan.
3. Updated OpenAPI or schema artifact.
4. Focused tests.
5. Update to `TRACEABILITY.md`.

## 6. Acceptance Criteria

- No API, event or worker behavior is implemented without a contract entry.
- UI routes reference the matching API group.
- Current/target status is not ambiguous.
