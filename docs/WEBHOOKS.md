# Webhooks - Wiki

## 1. Status

Webhooks are deferred. MVP does not include inbound or outbound webhook endpoints.

The base product uses only public REST API and two clients: UI and CLI.

## 2. MVP Boundary

Do not implement these in MVP:

- inbound task events;
- inbound workflow phase events;
- inbound pipeline events;
- outgoing document publication hooks;
- outgoing evidence hooks;
- webhook retry/backoff tables.

Documents, task links, phase links and evidence are created through normal API/CLI operations.

## 3. Future Inbound Webhooks

If approved later, inbound webhooks can create or update:

- task snapshot metadata;
- phase snapshot metadata;
- URL evidence;
- file evidence metadata.

They must not own external task state, workflow state or CI/CD state.

## 4. Future Outbound Webhooks

Outbound webhooks may notify external systems about:

- document published;
- document archived;
- evidence added;
- space member changed.

They require delivery retries, dead-letter handling and audit before implementation.

## 5. Security Rules for Future Work

- Require HMAC or signed token.
- Enforce replay window.
- Require idempotency key or source event id.
- Redact secrets in logs and audit.
- Return success for duplicate events without duplicate writes.

## 6. References

- `docs/API.md`
- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/contracts/EVENT_CONTRACT.md`
