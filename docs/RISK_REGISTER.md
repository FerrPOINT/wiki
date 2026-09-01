# Risk Register - Wiki

| ID | Risk | Impact | Probability | Mitigation |
|---|---|---|---|---|
| R-001 | External tracker terminology or modules get reintroduced into Wiki-owned code | High | Low | Keep API/domain/docs tied to Wiki vocabulary and review new dependencies/routes |
| R-002 | Markdown XSS | High | Medium | Sanitizer tests, CSP, security review |
| R-003 | Evidence duplication from retries | Medium | High | Idempotency keys and source_ref uniqueness |
| R-004 | Search returns unauthorized documents | High | Medium | Permission filters and cross-space tests |
| R-005 | Object storage data loss | High | Medium | Backups, checksums, restore drills |
| R-006 | Scope creep toward full Confluence clone | Medium | Medium | Keep MVP tied to task/phase evidence use case |
| R-007 | Deployment secrets leaked in logs | High | Low | redaction middleware and log tests |
| R-008 | Poor document governance | Medium | Medium | templates, required phase documents, traceability matrix |

## Review Cadence

- Review before each release.
- Add a risk when a P0 requirement changes.
- Close a risk only after mitigation is implemented and verified.
- Security risks require explicit owner and test evidence.

## Current Highest Risks

| Risk | Owner | Required Evidence |
|---|---|---|
| R-001 | Backend owner | Static scan shows no tracker-owned routes/modules in active runtime |
| R-002 | Security owner | Markdown sanitizer tests and CSP check |
| R-004 | Backend + frontend owners | Cross-space search permission tests |
| R-007 | Operator + backend owner | Secret redaction tests for logs/audit/screenshots |

## Acceptance Criteria

- No release ships with untriaged high-probability/high-impact risk.
- `CURRENT_STATE.md` references material unresolved risks.
- Mitigations map to `TEST_PLAN.md` or operational runbooks.
