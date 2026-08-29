# Notifications - Wiki

## 1. Status

Notifications are deferred. Wiki MVP has no notifications route, notification bell or notification API client.

## 2. MVP Boundary

MVP does not include:

- notification tables;
- user notification preferences;
- email digest;
- in-app unread counters;
- external delivery channels.

Users discover changes through document pages, task/phase pages, search and audit log.

## 3. Future Events

If notifications are approved later, candidate events are:

| Event | Description |
|---|---|
| `document_published` | Published revision |
| `document_archived` | Archived page |
| `evidence_added` | Added URL/file evidence |
| `space_member_changed` | Access changed |

## 4. Future Channels

| Channel | Description |
|---|---|
| `in_app` | Bell icon and notification center |
| `email` | Optional digest |

External webhook delivery is a separate future feature and not part of notification MVP.

## 5. Future Requirements

- Permission-filter notifications by space.
- Do not notify a user about content they cannot read.
- Redact secrets in notification payloads.
- Let users mute non-critical notifications.
- Keep security/access events auditable.

## 6. References

- `docs/EVENTS.md`
- `docs/API.md`
- `docs/SYSTEM_ADMIN.md`
