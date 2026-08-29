# Transition Map - Wiki

## From Inherited Task-tracker To Wiki

| Inherited Area | Target Replacement |
|---|---|
| projects | spaces |
| issues | task dossiers + external task snapshot |
| board/backlog | document tree + dossier overviews |
| sprints | workflow phase dossiers |
| worklogs | evidence/activity records |
| labels/custom fields | tags + metadata |
| issue attachments | document/evidence attachments |
| JQL search | Wiki full-text/facet search |

## Migration Order

1. Domain entities and value objects.
2. Database migrations.
3. Services/repositories.
4. API/OpenAPI.
5. Frontend thin client and pages.
6. Generated frontend client after Wiki OpenAPI is ready.
7. E2E and traceability.
