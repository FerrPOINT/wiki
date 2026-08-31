# Auth Implementation Spec - Wiki

## 1. Scope

MVP authentication provides user sessions/JWT for UI and CLI clients. Scoped API tokens are deferred and require a separate approval before implementation.

## 2. User Login

1. User posts email/password.
2. Password is verified with Argon2id.
3. Access token is issued with short TTL.
4. Refresh token/session is persisted hashed.
5. Login event is written to audit log.

## 3. Deferred API Tokens

Future token scopes may include:

- `documents:read`
- `documents:write`
- `evidence:write`
- `admin:read`

Tokens must be limited to space and capability if this scope is approved later.

## 4. Middleware

- Parse bearer token.
- Resolve user principal.
- Attach claims to request extensions.
- Reject inactive users and revoked tokens.

## 5. Error Codes

- `INVALID_CREDENTIALS`
- `TOKEN_EXPIRED`
- `TOKEN_REVOKED`
- `INSUFFICIENT_SCOPE`
- `PERMISSION_DENIED`
