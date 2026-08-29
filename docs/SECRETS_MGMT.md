# Secrets Management - Wiki

## 1. Purpose

Wiki MVP stores only runtime secrets: auth/session signing keys, initial admin password during bootstrap and optional storage credentials. Integration and worker secrets are deferred.

## 2. Secret Types

| Type | Examples | Owner |
|---|---|---|
| Auth secret | JWT/session signing secret | Operator |
| Bootstrap password | Initial admin password from env | Operator |
| Storage credential | Future external storage credential | Operator |

## 3. Storage Requirements

- Secret values are encrypted at rest with AES-256-GCM or equivalent envelope encryption.
- Database rows store key id, version, ciphertext, nonce, metadata and status.
- API responses never return plaintext secret values.
- Secret names and last rotation timestamp may be displayed.
- Secret values are redacted from logs and audit metadata.

## 4. Access Model

| Actor | Capability |
|---|---|
| System admin | view safe secret metadata |
| Operator | configure runtime/storage secrets |
| Editor/reader | no secret access |

Every secret metadata change creates an audit event with secret name/version, not value.

## 5. Rotation

Rotation flow:

1. Admin creates a new version for the secret name.
2. New runtime reads use the latest active version.
3. Old version remains valid during grace period if configured.
4. Admin revokes old version.
5. Audit log records create, activate and revoke events.

Emergency revoke immediately prevents future resolution.

## 6. Redaction

Redaction applies to:

- API errors;
- search documents;
- screenshots and seed evidence.

Redaction must handle direct values, common encodings and accidental token fragments. It is a defense-in-depth layer, not a replacement for keeping secrets out of user content.

## 7. Configuration

| Env | Description |
|---|---|
| `WIKI_SECRETS_KEY` | Base64 master key or local development key source |
| `WIKI_SECRET_ROTATION_GRACE_SECONDS` | Optional old-version grace period |
| `WIKI_SECRET_AUDIT_RETENTION_DAYS` | Retention for secret audit entries |

## 8. Acceptance Criteria

- Creating a secret never returns plaintext in response.
- Rotating a secret preserves name and increments version.
- Secret metadata changes are audited.
- Search and screenshots do not contain configured secret values.
- Revoked secrets cannot be resolved.

## References

- `docs/SECURITY.md`
- `docs/AUTHORIZATION.md`
- `docs/contracts/DATA_LIFECYCLE.md`
