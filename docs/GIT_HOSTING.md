# Git Hosting And Source Links - Wiki

## 1. Purpose

Wiki does not implement Git hosting. Source code remains in GitHub, GitLab, Gitea, Forge CI/CD or another external provider. Wiki stores stable references to repositories, commits, branches, pull requests and code-review evidence so every task page and workflow phase can explain what changed and why.

This document exists for parity with the CI/CD documentation set and defines the Wiki boundary: Git is a source system, not an owned domain.

## 2. Scope

MVP supports:

- links from documents, task pages or phase pages to repository, branch, commit and pull request URLs as ordinary evidence URL materials;
- evidence records for review approval, checks and merge events;
- search by document title/body and evidence metadata already stored in Wiki;
- audit entries for normal evidence/document mutations.

Wiki does not support:

- bare repository storage;
- Git Smart HTTP or SSH transport;
- Git LFS;
- branch protection enforcement;
- merge execution;
- repository-level issue tracking.

## 3. Source Reference Model

| Field | Required | Description |
|---|---:|---|
| `source_system` | yes | `github`, `gitlab`, `gitea`, `forge-cicd`, `manual` |
| `repository_url` | yes | Canonical browser URL |
| `repository_name` | yes | Human-readable repo name |
| `commit_sha` | no | Full SHA when evidence is commit-specific |
| `branch` | no | Source or target branch |
| `pull_request_url` | no | PR/MR browser URL |
| `pull_request_number` | no | Provider-local number |
| `title_snapshot` | no | Captured title at ingest time |
| `author_snapshot` | no | Captured author/login at ingest time |

References are snapshots. Published document revisions must keep the original reference visible for historical traceability.

## 4. User Flows

1. A user opens a task page.
2. Wiki shows linked PRs, commits and CI/CD evidence.
3. The user opens a PR summary document or evidence card.
4. Wiki displays source URL, status snapshot, review state and related documents.
5. Search can find the task by repository name, PR number, commit SHA or linked document title.

## 5. Source References

| Source | Inbound Data |
|---|---|
| task-tracker | task key, source repository hints, PR backlinks |
| CI-CD | pipeline, job, artifact, deployment and commit metadata |
| project-workflow | phase identifiers and required source evidence |
| manual | user-provided repository and PR links |

## 6. Security

- Repository credentials are never stored in source references.
- Private repository URLs are visible only to users who can access the owning space or page.
- Commit/PR metadata is indexed only after permission filtering.
- Imported descriptions are treated as untrusted content and escaped or sanitized.

## 7. API Boundary

There is no dedicated source-links API in MVP. Source references are stored through the regular evidence API as URL evidence with source metadata. A future source-links API requires separate requirements, OpenAPI changes and UI scope.

Write operations should be idempotent by `(owner_type, owner_id, source_system, repository_url, commit_sha, pull_request_url)` when clients submit an `Idempotency-Key`.

## 8. Acceptance Criteria

- A task page can show all linked PRs and commits.
- Source links survive document publication and revision history.
- Permission filtering prevents cross-space source-link leaks.
- Search finds documents by repository, PR number and commit SHA.
- Audit log records every manual source-link mutation.

## References

- `docs/PULL_REQUESTS.md`
- `docs/DATA_MODEL.md`
- `docs/API.md`
- `docs/SECRETS_MGMT.md`
- `docs/contracts/API_CONTRACT.md`
