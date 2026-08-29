# Contributing - Wiki

## 1. Getting Started

```bash
git clone git@github.com:FerrPOINT/wiki.git
cd wiki
cp .env.example .env
```

## 2. Development Setup

```bash
# Backend
cd backend
cargo build
cargo run --bin server

# Frontend
cd frontend
pnpm install
pnpm dev
```

## 3. Before You Contribute

- Read `docs/PRODUCT_REQUIREMENTS.md` and `docs/ARCHITECTURE.md`.
- Check `docs/CODE_STYLE.md`.
- Add an ADR when the change affects architecture, storage, public API, search, permissions, or integrations.
- Discuss large changes before implementation.

## 4. Making Changes

1. Create a branch: `feat/short-desc` or `fix/short-desc`.
2. Keep changes aligned with the Wiki domain model: spaces, documents, task dossiers, phase dossiers, evidence and attachments.
3. Add or update focused tests.
4. Update docs when behavior or contracts change.
5. Run local checks before opening a PR.

## 5. Local Checks

```bash
# Backend
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# Frontend
pnpm lint
pnpm typecheck
pnpm test
pnpm test:e2e
```

## 6. Commit Messages

Use Conventional Commits:

```text
feat(documents): add revision publishing
feat(evidence): attach workflow artifact to phase dossier
fix(search): exclude archived documents by default
docs(api): describe document diff endpoint
test(cli): cover evidence upload command
```

## 7. Pull Request

- Keep PRs small and reviewable.
- Self-review before requesting review.
- Fill the PR template.
- Link the related task or design note.
- Ensure CI is green.
- Address review feedback with clear follow-up commits.

## 8. Documentation Updates

Every PR must update documentation when it changes:

- architecture, API, workflow integration or storage - update the matching `docs/*.md`;
- new env/configuration - update `docs/DEPLOYMENT.md`, `.env.example`, `README.md`;
- new endpoint - update `docs/API.md` and regenerate OpenAPI;
- new CLI command - update `docs/CLI.md` and `cli/SKILL.md`;
- changed data model - update `docs/DATA_MODEL.md` and migration docs.

## 9. Release

- Maintainers cut releases.
- Follow Semantic Versioning.
- Update `CHANGELOG.md` before tagging.

## 10. Communication

- Tasks: project tracker card or GitHub issue.
- Architecture questions: ADR or design note in Wiki.
- Russian or English accepted.

## 11. References

- `docs/PRODUCT_REQUIREMENTS.md` - product requirements.
- `docs/ARCHITECTURE.md` - architecture and stack.
- `docs/DATA_MODEL.md` - target data model.
- `docs/DOMAIN_MODEL.md` - bounded contexts and aggregates.
- `docs/API.md` - endpoint catalog.
- `docs/CLI.md` - CLI commands.
- `docs/TESTING.md` - testing strategy.
