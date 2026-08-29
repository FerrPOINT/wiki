# Automation Architecture - Wiki

## 1. Status

Automation is deferred. It is kept as an architectural reference because the Wiki repository mirrors the CI/CD documentation set, but it is not part of the base Wiki MVP.

## 2. MVP Boundary

The base product has no inbound webhook processing, no outgoing webhook delivery and no background automation ownership.

MVP writes happen through:

- UI calling public API;
- CLI calling public API;
- direct user/admin actions authenticated by Wiki.

Any external system that needs to add documents or evidence must use the same public API or CLI as every other client. There is no separate automation domain model.

## 3. Deferred Scope

Automation may be reconsidered only after the base app is working:

- documents and revisions;
- task and phase links;
- evidence links/files;
- search;
- audit;
- UI and CLI.

Future automation could include:

- importing task snapshots;
- importing pipeline/evidence links;
- scheduled cleanup;
- search reindex jobs.

These features require separate requirements before implementation.

## 4. Rules for Future Work

- Do not add webhooks to MVP endpoints.
- Do not create source-specific commands in CLI.
- Do not let automation own task state, phase state or pipeline state.
- Use idempotency keys for every future automated write.
- Keep audit on every automated write.

## 5. References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/CLI.md`
- `docs/EVENTS.md`
