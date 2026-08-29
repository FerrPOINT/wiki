# Pull Requests And Code Review Evidence - Wiki

## 1. Purpose

Wiki records pull request context as documentation evidence. It helps a reader answer:

- which PR implemented the task;
- what decision or requirement the PR belongs to;
- which checks passed;
- who reviewed it;
- what was merged, rejected or rolled back.

Wiki is not the merge authority. Pull request state is imported as a snapshot from the external source.

## 2. Pull Request Card

Each PR card in a document, task dossier or phase dossier contains:

| Field | Description |
|---|---|
| Repository | Provider and repository name |
| PR number | Provider-local number |
| Title | Snapshot title |
| Status | `open`, `merged`, `closed`, `draft`, `unknown` |
| Source/target branch | Optional branch names |
| Author | Snapshot author |
| Review state | `pending`, `approved`, `changes_requested`, `unknown` |
| Checks | Link to CI/CD evidence summary |
| URL | External browser link |

## 3. Document Links

Pull request cards can be attached to:

- requirements documents;
- architecture decision documents;
- implementation notes;
- test notes;
- release notes;
- task dossiers;
- workflow phase dossiers.

Published revisions preserve the attached PR snapshot. Later refreshes create a new metadata snapshot and do not mutate historical revision meaning.

## 4. Review Evidence

Review evidence is represented as `Evidence` with:

- `kind = review`;
- source reference;
- external review URL;
- reviewer snapshot;
- status snapshot;
- optional checklist result;
- optional files or comments summary.

Sensitive comments are not copied unless the source explicitly marks them safe for Wiki storage.

## 5. UI Requirements

- Task page shows PR links near implementation materials.
- Phase page shows whether required review materials exist.
- Document view shows PR links in metadata, not inside body-only text.
- Search supports repository, PR number, title, author and status facets.
- Dashboard can surface missing PR/review material gaps.

## 6. API Requirements

Dedicated PR endpoints are deferred. In MVP, PR links are stored through the regular evidence API as external URL materials.

Future target endpoints:

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/pull-requests` | Filter imported PR snapshots |
| `POST` | `/api/v1/tasks/{source}/{key}/pull-requests` | Link PR to task dossier |
| `POST` | `/api/v1/documents/{id}/pull-requests` | Link PR to document |

Future ingestion is idempotent by provider event id or by `(source_reference, repository_url, pull_request_number, updated_at)`.

## 7. Acceptance Criteria

- Users can see PR context without leaving a task dossier.
- A published requirements or release document keeps its PR references.
- CI/CD checks can be linked to the PR card as evidence.
- A missing review evidence gap appears on the dashboard or task page.
- PR data respects space permissions.

## References

- `docs/GIT_HOSTING.md`
- `docs/REVIEW.md`
- `docs/REPORTS.md`
- `docs/contracts/EVENT_CONTRACT.md`
