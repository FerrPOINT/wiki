# Frontend Boundaries - Wiki

## Target Layers

```text
app -> pages -> widgets -> features -> entities -> shared
```

## Page Ownership

| Page | Owns Composition |
|---|---|
| dashboard | recent docs, gaps, stats |
| spaces | tree and space cards |
| document | viewer, revision panel, related objects |
| document-compose | editor/draft form |
| task-dossier | task docs, phase summary, materials |
| phase-dossier | phase docs and materials |
| evidence | material filters and artifact list |
| templates | template catalog |
| audit-log | immutable events |
| users | access matrix |
| search | query/filter/results |
| admin/settings | configuration |

## Rules

- Shared UI has no Wiki domain knowledge.
- Features call entities API; pages compose features/widgets.
- Current MVP shell uses a thin handwritten API client for auth.
- Target state uses generated OpenAPI types as the API boundary after backend domain migration.
