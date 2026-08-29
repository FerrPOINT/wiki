# Auth Implementation Spec - Wiki

## 1. Scope

Authentication provides sessions/JWT and scoped API tokens for users, UI and CLI clients.

## 2. User Login

1. User posts email/password.
2. Password is verified with Argon2id.
3. Access token is issued with short TTL.
4. Refresh token/session is persisted hashed.
5. Login event is written to audit log.

## 3. API Tokens

Token scopes:

- `documents:read`
- `documents:write`
- `evidence:write`
- `admin:read`

Tokens may be limited to space and capability.

## 4. Middleware

- Parse bearer token.
- Resolve user or service principal.
- Attach claims to request extensions.
- Reject inactive users and revoked tokens.

## 5. Error Codes

- `INVALID_CREDENTIALS`
- `TOKEN_EXPIRED`
- `TOKEN_REVOKED`
- `INSUFFICIENT_SCOPE`
- `PERMISSION_DENIED`
