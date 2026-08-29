# Sequence - Webhook Delivery

Webhook delivery is deferred reference material. It is not part of the Wiki MVP frontend, API or runtime.

```mermaid
sequenceDiagram
    participant O as Outbox
    participant W as Worker
    participant E as External System
    W->>O: claim outgoing webhook
    W->>W: sign payload
    W->>E: POST webhook
    E-->>W: 2xx/5xx
    W->>O: mark delivered or retry
```

## Rules

- Payload is created from committed domain event.
- Delivery includes event id, timestamp and HMAC signature when configured.
- Retry uses exponential backoff and does not reorder events for the same aggregate when ordering is required.
- Delivery response body is truncated and redacted before persistence.

## Failure Modes

| Failure | Handling |
|---|---|
| 2xx | Mark delivered |
| 4xx | Mark failed unless policy says retry |
| 5xx/network | Retry until max attempts |
| Signature config missing | Do not deliver when receiver requires signing |

## Future Acceptance Criteria

- Delivery attempts are visible in a future operator view or audit export.
- Operators can replay safe failed delivery when webhook delivery is approved.
- Duplicate delivery keeps the same event id.
