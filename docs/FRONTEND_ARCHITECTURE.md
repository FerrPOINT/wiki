# Frontend Architecture - Wiki

## 1. Overview

Frontend - React SPA on Vite, TypeScript and Tailwind. The UI is an operational knowledge base, so it should prioritize fast navigation, readable documents, dense lists and predictable controls.

## 2. Tech Stack

| Layer        | Library                                          |
| ------------ | ------------------------------------------------ |
| Framework    | React 19                                         |
| Build        | Vite                                             |
| Language     | TypeScript                                       |
| Styling      | Tailwind CSS with CSS variables                  |
| Components   | Radix primitives / local shadcn-style components |
| Icons        | lucide-react                                     |
| Server state | TanStack Query                                   |
| Client state | Zustand                                          |
| Routing      | react-router                                     |
| Forms        | native React forms or local validators           |
| Tests        | Vitest, Testing Library, Playwright              |

## 3. Target Folders

```text
frontend/src/
  app/
    router.tsx
  pages/
    dashboard/
    spaces/
    document/
    document-compose/
    task-dossier/
    phase-dossier/
    evidence/
    templates/
    audit-log/
    users/
    settings/
    wiki-search/
    admin/
    login/
    register/
  widgets/
    app-shell.tsx
    document-tree/
    revision-panel/
    evidence-feed/
  features/
    publish-document/
    edit-document-draft/
    attach-evidence/
    link-task-dossier/
  entities/
    document/
    space/
    task-dossier/
    phase-dossier/
    evidence/
    user/
  shared/
    api/
    auth/
    i18n/
    lib/
    ui/
```

## 4. Dependency Rule

```text
app -> pages -> widgets -> features -> entities -> shared
```

- Upper layers may import lower layers.
- Lower layers do not import upper layers.
- Feature modules should expose a small public API through `index.ts`.
- Shared UI components do not know domain objects.

## 5. Main Routes

- `/` - Wiki dashboard.
- `/spaces` - spaces and document tree.
- `/documents/new` - document draft.
- `/documents/:documentId` - document view.
- `/tasks` and `/tasks/:taskKey` - task-linked document view.
- `/phases` and `/phases/:phaseId` - phase-linked document view.
- `/evidence` - material registry.
- `/templates` - document templates.
- `/audit-log` - audit events.
- `/users` - users and roles.
- `/settings` - instance settings.
- `/search` - document/task/phase/material search.
- `/admin` - administration overview.

## 6. Server State

Current frontend uses generated OpenAPI DTO types in `frontend/src/api/generated.ts`, a thin handwritten HTTP wrapper in `frontend/src/api/wiki.ts` / `frontend/src/api/auth.ts`, and TanStack Query hooks in `frontend/src/shared/api/hooks.ts`. Dashboard, admin overview and all MVP feature pages read their displayed runtime data from the public Wiki API; deterministic mocks remain only in tests and screenshot capture. `npm run generate:api` refreshes DTO schemas from `openapi/openapi.json`; full operation-client generation remains deferred until the app/infra repository boundary stabilizes.

Query key examples:

```ts
export const documentKeys = {
  all: ["documents"] as const,
  detail: (id: string) => [...documentKeys.all, "detail", id] as const,
  revisions: (id: string) => [...documentKeys.all, "revisions", id] as const,
};

export const dossierKeys = {
  task: (space: string, key: string) => ["task-dossier", space, key] as const,
  phase: (space: string, key: string) => ["phase-dossier", space, key] as const,
};
```

## 7. Client State

Use Zustand only for UI state:

- theme;
- sidebar collapsed state;
- unsaved editor panel state;
- recently opened documents;
- search filters before submit.

Do not mirror server entities into client stores.

## 8. Editor

MVP editor:

- Markdown textarea with preview.
- Autosave draft only after explicit backend support.
- Publish creates immutable revision.

## 9. MVP Scope

The frontend contains only the approved Wiki MVP pages. Deferred reporting, notification, webhook and runner ideas are documented as reference material only and must not add routes, menu items or API clients until a new product scope is approved.

## 10. Testing

- Unit/component tests for document tree, editor, evidence feed and permissions states.
- Current E2E smoke: login, navigate through API-backed MVP pages, save a document draft, publish a revision, filter evidence by document and search by document type with deterministic Wiki API mocks.
- Целевое расширение E2E: создать черновик из `/documents/new`, приложить evidence и покрыть состояния запрета доступа.
- Visual checks for document layout on mobile and desktop.

## 11. References

- `docs/ROUTING.md`
- `docs/PAGE_DESIGN.md`
- `docs/API.md`
- `docs/FRONTEND_STANDARDS.md`
