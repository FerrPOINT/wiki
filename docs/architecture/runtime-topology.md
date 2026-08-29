# Runtime Topology - Wiki

```text
          Browser
             |
      Frontend static
             |
        Reverse proxy
             |
        Wiki Backend
       /     |      \
PostgreSQL Redis Object storage
       |
 Search projection
```

## Notes

- PostgreSQL is source of truth.
- Object storage stores binary artifacts.
- Redis is optional for cache or session support.
- Search projection is PostgreSQL full-text search in MVP.
- Background workers are not required for the base product.

## Deployment Modes

| Mode | Topology | Use |
|---|---|---|
| Local MVP | API and frontend from developer machine; PostgreSQL/Redis in Docker | development |
| Single-node self-hosted | reverse proxy, API, PostgreSQL, optional Redis and local storage | small team |
| S3-compatible storage | PostgreSQL local or managed; MinIO/S3 for attachments | larger files/backup |

## Network Boundaries

- Browser talks only to frontend/reverse proxy.
- Frontend calls `/api/v1`.
- PostgreSQL and Redis are not exposed to the public network.
- Object storage URLs are signed or proxied according to access policy.

## Operational Signals

Runtime health requires API liveness, PostgreSQL connectivity, migration version, search index lag and storage write/read probe. These signals feed `OPERATIONS.md`, `SLO.md` and `MONITORING.md`.
