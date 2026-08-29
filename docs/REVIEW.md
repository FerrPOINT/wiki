# Review - Wiki

## 1. Purpose

Review in Wiki MVP means a lightweight human quality check of important pages before or after publication. It is a documentation practice, not a separate approval workflow, not code review in Git and not release sign-off.

Wiki stores the resulting document revision, task link, phase link, evidence and audit entries. It does not own external review decisions.

## 2. MVP Scope

Review support in MVP is intentionally small:

- document pages show status, owner, current revision and related task/phase links;
- task and phase pages show linked documents and evidence gaps;
- templates include sections for acceptance criteria, risks, checks and release notes;
- audit records document publish/archive and evidence changes;
- search helps users find current and historical knowledge.

There are no review-specific MVP routes, API groups, reports or notification flows.

## 3. Quality Checklist

For every high-impact page:

- title and owner are clear;
- document type is correct;
- linked task key or phase key is present when applicable;
- acceptance criteria are testable;
- evidence links or files are attached when the page claims completed work;
- security, privacy and operational impact are mentioned when relevant;
- release-impacting pages include rollback or support notes;
- stale or conflicting pages are linked or archived.

## 4. Document States

MVP uses the document lifecycle states from `docs/PRODUCT_REQUIREMENTS.md`:

| State | Meaning |
|---|---|
| `draft` | Editable working copy exists |
| `published` | Current immutable revision is visible to readers |
| `archived` | Page is hidden from the default tree but preserved |

Changing a published page creates a new draft and then a new revision. Published revision content is not changed in place.

## 5. UI Expectations

- Document view shows current status, owner and revision metadata.
- Task page shows related documents and evidence for one task key.
- Phase page shows related documents and evidence for one phase key.
- Templates make expected sections visible before writing begins.
- Audit log is the place to inspect write actions.

## 6. Deferred

The following are outside MVP and require separate requirements before implementation:

- approval chains;
- review assignments;
- review-specific API endpoints;
- pending-review reports;
- review notifications.

## 7. Acceptance Criteria

- A user can understand whether a page is draft, published or archived.
- A task or phase page makes missing documents/evidence visible without a report module.
- Templates guide authors toward complete SDLC documentation.
- Audit entries preserve who changed documents, evidence and permissions.
- No review route or review API is required for the base application.

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/UI_UX.md`
- `docs/AUTHORIZATION.md`
- `docs/AUDIT.md`
