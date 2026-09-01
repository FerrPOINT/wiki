# Sequence: CI/CD Evidence Reference

## Purpose

Attach CI/CD outcomes to Wiki task and phase pages as evidence. In MVP this is a normal evidence write through UI, CLI or public API. Signed CI/CD webhooks are deferred.

## Flow

```text
CI-CD pipeline finishes
  -> User or script captures pipeline URL, artifact URL or file
  -> UI/CLI/API creates evidence in Wiki
  -> Evidence is linked to a document, task key or phase key
  -> Wiki validates permissions and idempotency
  -> Wiki stores evidence metadata and audit entry
  -> Task/phase page shows the linked material
```

## Evidence Mapping

| CI/CD Object | Wiki Evidence       |
| ------------ | ------------------- |
| Pipeline run | `kind=ci_pipeline`  |
| Job log      | `kind=ci_job_log`   |
| Artifact     | `kind=ci_artifact`  |
| Deployment   | `kind=deployment`   |
| Failed check | `kind=quality_gate` |

## Idempotency

Use an explicit idempotency key for repeated CLI/API writes. URL evidence sends the provider URL only; file evidence is a staged attachment whose checksum is computed by Wiki during upload.

## Acceptance Criteria

- A successful pipeline can satisfy testing evidence.
- A deployment can satisfy release evidence.
- Repeated writes with the same idempotency key do not duplicate evidence rows.
- Evidence links back to external CI/CD URLs.
- Task and phase pages show the linked evidence.

## References

- `docs/ARTIFACTS.md`
- `docs/contracts/INTEGRATION_PROTOCOL.md`
