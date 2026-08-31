# ADR-0002: React + Vite + Tailwind For Frontend

## Status

Accepted

## Context

Wiki frontend is an operational knowledge interface: dashboard, spaces, document tree, editor, task-linked pages, phase-linked pages, evidence, templates, search and admin pages. It must be fast to develop, easy to host as static assets and consistent with the sibling SDLC tools.

## Alternatives Considered

| Option                  | Pros                                             | Cons                                                |
| ----------------------- | ------------------------------------------------ | --------------------------------------------------- |
| Next.js                 | SSR and file routing                             | Extra runtime complexity for a private SPA          |
| Vue + Vite              | Simple and productive                            | Less local reuse from existing React SDLC projects  |
| SvelteKit               | Small bundles, good DX                           | Smaller local ecosystem and fewer reusable patterns |
| React + Vite + Tailwind | Existing stack, broad ecosystem, fast dev server | Requires explicit routing/data conventions          |

## Decision

Use React 19, Vite, TypeScript and Tailwind CSS. The app remains a SPA served behind a reverse proxy. Pages are organized under `frontend/src/pages`, shared shell under `widgets`, local UI primitives under `shared/ui`.

## Consequences

- Frontend can reuse existing build/deploy knowledge from CI/CD and task-tracker projects.
- Static hosting is simple.
- UI is optimized for dense operational screens, not marketing pages.
- Generated OpenAPI DTO types are used for the MVP API boundary; full operation-client generation remains target state after backend Wiki repository migration.
- SSR and collaborative editing are postponed until a concrete requirement appears.

## References

- `docs/FRONTEND_ARCHITECTURE.md`
- `docs/UI_UX.md`
- `docs/ROUTING.md`
