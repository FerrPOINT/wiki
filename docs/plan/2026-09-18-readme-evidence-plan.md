# Wiki README Evidence Plan

> **Status 2026-09-18:** active Base README migration wave. Scope is documentation, reviewed product evidence and a structural CI gate; no API, schema or authentication semantics change.

## Evidence Decision

- Keep the existing `01-login.png` because it contains blank credentials and no environment information.
- Use `12-templates.png` as desktop proof: it has generic template structures and no identifiers, user data, timestamps or URLs.
- Add a redacted mobile login proof from the live `7732/login` surface at `375x812`; the public form remains blank, while the capture wrapper removes browser/runtime chrome.
- Do not use documents, spaces, dashboards, audit or identity surfaces in the root README because their mock fixtures expose SDLC terms, test email-like accounts, IDs or future timestamps.

## Execution

1. Add a test-first `scripts/verify_readme.py` contract for required anchors, local assets, reviewed proof and accidental local paths/placeholders.
2. Add a small `readme` GitHub Actions job, independent of Rust/Node package gates.
3. Replace the root README gallery and duplicated file registry with a short evidence-first product entry point.
4. Add a local ink/violet/sky banner and retain the complete screenshot inventory in `docs/assets/screens/manifest.md`.
5. Validate Rust/Node tests, Compose config, local runtime health/readiness, browser rendering and hosted CI before publishing.
