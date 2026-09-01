# Caching Strategy - Wiki

## 1. Overview

Кеширование ускоряет чтение документов, дерева space, прав и результатов поиска. В MVP обязательным является только frontend query cache; backend cache остаётся опциональной оптимизацией поверх PostgreSQL. Frontend использует TanStack Query для server state.

## 2. Cache Layers

| Layer | Library | Use Case | TTL |
|---|---|---|---|
| L1 process cache | optional Rust cache adapter | permissions, settings, templates | 1-10 min |
| L2 distributed | `redis` | document metadata, search filters, permissions snapshots | 5-60 min |
| Query cache | TanStack Query | Frontend server state | per route |
| CDN/browser | Nginx/object storage | Static assets and immutable attachments | long-term |

## 3. Key Convention

```text
wiki:{entity}:{id}[:{version}]
```

Examples:

- `wiki:space:ENG`
- `wiki:document:018f...`
- `wiki:document-tree:ENG:v12`
- `wiki:revision:018f...:7`
- `wiki:task-key:ENG:SDLC-42`
- `wiki:phase-dossier:018f...`
- `wiki:search:{hash}`

## 4. What to Cache

| Data | Cache | TTL | Invalidation |
|---|---|---|---|
| Space metadata | Redis | 10 min | space update/archive |
| Document metadata | Redis | 5 min | document update/archive |
| Current published revision | Redis | 5 min | publish/restore |
| Space tree | Redis | 2 min | create/move/archive document |
| Permissions matrix | process cache | 5 min | member/role change |
| Templates | process cache + Redis | 15 min | template update |
| Search results | Redis | 1 min | document/evidence index event |

## 5. What Not to Cache

- Passwords, bearer tokens, refresh tokens, API secrets.
- Draft bodies in shared Redis unless encrypted and explicitly configured.
- Large binary files; object storage handles them.
- Audit log writes before durable persistence.

## 6. Cache Aside Pattern

```rust
async fn get_document(&self, id: Uuid) -> Result<Document, Error> {
    let key = format!("wiki:document:{id}");
    if let Some(cached) = self.cache.get(&key).await {
        return Ok(cached);
    }

    let document = self.repo.find_by_id(id).await?;
    self.cache.set(key, document.clone(), TTL_5_MIN).await;
    Ok(document)
}
```

## 7. Invalidation

```rust
async fn publish_document(&self, id: Uuid, draft: PublishDraft) -> Result<DocumentRevision, Error> {
    let revision = self.repo.publish(id, draft).await?;
    self.cache.delete(format!("wiki:document:{id}")).await;
    self.cache.delete_pattern("wiki:document-tree:*").await;
    self.cache.delete_pattern("wiki:search:*").await;
    self.event_bus.publish(DocumentPublished { id, revision_id: revision.id }).await;
    Ok(revision)
}
```

## 8. Frontend Query Caching

| Query | Stale Time |
|---|---|
| Current user | 5 min |
| Space list | 2 min |
| Space tree | 30 sec |
| Document view | 30 sec |
| Search | 15 sec |
| Dossier overview | 30 sec |
| Evidence list | 15 sec |

## 9. Invalidation Strategy

- Backend invalidates local/distributed cache after successful write transactions.
- Frontend invalidates TanStack Query keys after successful mutations.
- Refetch on focus can reconcile stale reads.
- Realtime cache invalidation is deferred and must not be required for MVP.

## 10. Monitoring

- Hit/miss ratio by namespace.
- Redis latency and memory usage.
- Process cache size and eviction count.
- Alert when cache errors exceed threshold but keep reads functional from PostgreSQL.

## 11. References

- `docs/ARCHITECTURE.md`
- `docs/PERFORMANCE.md`
- `docs/EVENTS.md`
