# Wiki README Evidence Plan

> **Status 2026-09-22:** historical record of the initial README migration. The active Base contract is desktop-only; auth and narrow-viewport evidence remain UI QA, not README or screenshot manifest material.

## Evidence Decision

- Use `12-templates.png` as desktop proof: it has generic template structures and no identifiers, user data, timestamps or URLs.
- Do not use auth surfaces in the root README or manifest. Keep identity and narrow-viewport checks in product UI QA.

## Execution

1. Add a test-first `scripts/verify_readme.py` contract for required anchors, local assets, reviewed proof and accidental local paths/placeholders.
2. Add a small `readme` GitHub Actions job, independent of Rust/Node package gates.
3. Replace the root README gallery and duplicated file registry with a short evidence-first product entry point.
4. Retain only the desktop screenshot inventory in `docs/assets/screens/manifest.md`.
5. Validate Rust/Node tests, Compose config, local runtime health/readiness, browser rendering and hosted CI before publishing.
