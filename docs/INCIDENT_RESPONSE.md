# Incident Response - Wiki

## 1. Severity

| Severity | Example |
|---|---|
| SEV1 | Data leak, data loss, auth bypass |
| SEV2 | API unavailable, publish broken, uploads broken |
| SEV3 | Search stale, slow uploads |
| SEV4 | Minor UI/documentation issue |

## 2. First Response

1. Declare severity and owner.
2. Preserve logs and audit events.
3. Disable unsafe tokens or public links if needed.
4. Communicate impact and next update time.
5. Mitigate, then repair root cause.

## 3. Playbooks

| Incident | Playbook |
|---|---|
| Suspected data leak | rotate tokens, disable public links, inspect audit |
| Storage corruption | switch to read-only, restore from backup, verify checksums |
| Search outage | bypass to DB search or show stale warning |
| Write storm | disable source token, deduplicate by idempotency key |

## 4. Post-incident

- Write incident note in Wiki.
- Link affected task/phase dossiers.
- Add regression tests.
- Update threat model/risk register if needed.
