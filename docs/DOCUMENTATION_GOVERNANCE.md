# Documentation Governance - Wiki

## 1. Purpose

Documentation is part of the delivery contract. Wiki changes must update requirements, architecture, API, test and operations documents when behavior changes.

## 2. Document Types

| Type | File |
|---|---|
| Product requirements | `docs/PRODUCT_REQUIREMENTS.md` |
| Current state | `docs/CURRENT_STATE.md` |
| Architecture entry point | `docs/ARCHITECTURE_INDEX.md` |
| API contract | `docs/API.md`, `docs/contracts/API_CONTRACT.md` |
| Data lifecycle | `docs/contracts/DATA_LIFECYCLE.md` |
| Test evidence | `docs/TEST_PLAN.md`, `docs/TRACEABILITY.md` |
| Operations | `docs/OPERATIONS.md`, `docs/SLO.md` |

## 3. Rules

- Every capability has a REQ-ID.
- Every public endpoint has OpenAPI and API docs.
- Every schema change updates data model and migration contract.
- Every trust boundary change updates threat model.
- Every user-visible change updates user guide and route docs.
- Current-state claims must be backed by code, tests or manual evidence.

## 4. Review Checklist

- Links resolve.
- Current vs target status is explicit.
- No stale task-tracker-only terminology except migration notes.
- Screens/pages in docs match frontend routes.
- Test plan covers changed REQ-IDs.
