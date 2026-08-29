# Architecture Index - Wiki

## Purpose

This index is the entry point for Wiki architecture. It separates current scaffolding inherited from `task-tracker` from the target base Wiki architecture.

## Start Here

1. `docs/PRODUCT_REQUIREMENTS.md` - requirements and REQ-ID catalog.
2. `docs/CURRENT_STATE.md` - current vs target capability snapshot.
3. `docs/FUNCTIONAL_ARCHITECTURE.md` - capability map and bounded contexts.
4. `docs/ARCHITECTURE.md` - runtime and workspace layout.
5. `docs/DOMAIN_MODEL.md` - aggregates and invariants.
6. `docs/DATA_MODEL.md` - target database model.
7. `docs/IMPLEMENTATION_CONTRACTS.md` - implementation rules and contract index.

## Bounded Contexts

| Context | Architecture | Details |
|---|---|---|
| Identity & Access | `docs/AUTHORIZATION.md` | `docs/SECURITY.md`, `docs/SYSTEM_ADMIN.md` |
| Knowledge Base | `docs/FUNCTIONAL_ARCHITECTURE.md` | `docs/DATA_MODEL.md`, `docs/API.md` |
| SDLC Links and Evidence | `docs/FUNCTIONAL_ARCHITECTURE.md` | `docs/EVENTS.md`, `docs/WEBHOOKS.md` as deferred references |
| Storage & lifecycle | `docs/STORAGE_ARCHITECTURE.md` | `docs/STORAGE.md`, `docs/BACKUP_RESTORE.md` |
| API, UI & delivery | `docs/DELIVERY_ARCHITECTURE.md` | `docs/API*.md`, `docs/FRONTEND_ARCHITECTURE.md`, `docs/CLI.md` |
| Operations | `docs/OPERATIONS.md` | `docs/SLO.md`, `docs/METRICS.md`, `docs/TROUBLESHOOTING.md` |

## SDLC Quality Set

- `docs/TRACEABILITY.md` - requirements traceability matrix.
- `docs/TEST_PLAN.md` - verification strategy.
- `docs/THREAT_MODEL.md` - trust boundaries and abuse cases.
- `docs/RISK_REGISTER.md` - product and delivery risks.
- `docs/ACCESSIBILITY.md` - accessibility acceptance criteria.
- `docs/THIRD_PARTY.md` - dependency and license tracking.
- `docs/DISASTER_RECOVERY.md` - restore objectives.
- `docs/INCIDENT_RESPONSE.md` - operational incidents.

## Mandatory Change Impact

| Change | Must Update |
|---|---|
| REST contract | `docs/API.md`, `docs/contracts/API_CONTRACT.md`, OpenAPI, target generated client |
| Database schema | migrations, `docs/DATA_MODEL.md`, `docs/contracts/MIGRATION_CONTRACT.md` |
| Authorization boundary | `docs/AUTHORIZATION.md`, `docs/THREAT_MODEL.md`, policy tests |
| Document format | `docs/contracts/DOCUMENT_FORMAT.md`, renderer/sanitizer tests |
| Domain event | `docs/EVENTS.md`, `docs/contracts/EVENT_CONTRACT.md`, audit/search tests |
| Storage lifecycle | `docs/STORAGE_ARCHITECTURE.md`, `docs/contracts/DATA_LIFECYCLE.md` |
| User-visible behavior | `docs/USER_GUIDE.md`, Playwright smoke, screenshots |

## Current vs Target Notation

- **Current**: verified in this repository.
- **Target**: approved architecture; implementation pending.
- **Inherited**: copied from `task-tracker` and waiting for replacement.
