# Event Contract - Wiki

## 1. Envelope

```json
{
  "event_id": "018f...",
  "event_type": "document.published",
  "version": 1,
  "occurred_at": "2026-08-27T12:34:56Z",
  "actor_id": "018f...",
  "entity_type": "document",
  "entity_id": "018f...",
  "payload": {}
}
```

## 2. Rules

- `event_id` is globally unique.
- `event_type` is lowercase dot notation.
- Events are append-only.
- Consumers are idempotent by `event_id`.
- Breaking payload changes increment `version`.

## 3. Required Event Families

- `space.*`
- `document.*`
- `task_dossier.*`
- `phase_dossier.*`
- `evidence.*`
- `attachment.*`

## 4. Deferred Event Families

The following event families are not part of MVP and must not be required by API, UI or CLI until separately approved:

- `comment.*`
- `notification.*`
- `webhook.*`
- `external_sync.*`
