# Monitoring — Wiki

## 1. Overview

Мониторинг покрывает metrics, logs, traces, alerting. Стек: Prometheus, Grafana, Loki, Alertmanager, OpenTelemetry.

## 2. Metrics

### 2.1 Implemented Backend Metrics (Prometheus)

| Metric | Type | Description |
|--------|------|-------------|
| `axum_http_requests_total` | counter | total requests by method, endpoint and status |
| `axum_http_requests_duration_seconds` | histogram | request latency by method, endpoint and status |
| `axum_http_requests_pending` | gauge | in-flight requests by method and endpoint |
| `wiki_auth_login_attempts_total` | counter | login attempts by result |
| `wiki_users_created_total` | counter | users created by admins |
| `wiki_spaces_created_total` | counter | spaces created by admins |
| `wiki_documents_created_total` | counter | documents created by document type |
| `wiki_document_revisions_published_total` | counter | published document revisions |
| `wiki_documents_archived_total` | counter | archived documents |
| `wiki_task_document_links_total` | counter | document links added to task dossiers |
| `wiki_phase_document_links_total` | counter | document links added to phase dossiers |
| `wiki_evidence_added_total` | counter | evidence records added by source type |
| `wiki_attachments_uploaded_total` | counter | uploaded attachment files |
| `wiki_attachment_upload_bytes_total` | counter | uploaded attachment bytes |
| `wiki_templates_created_total` | counter | templates created by admins |
| `wiki_search_queries_total` | counter | search queries by scope |
| `wiki_permission_denied_total` | counter | 403 responses by API scope |

Operational probes `GET /api/v1/health` and `GET /api/v1/health/ready` bypass the general API rate limiter so Docker and monitoring can distinguish liveness/readiness during client traffic bursts.

### 2.2 Backend Hardening Backlog

The following metrics are planned hardening items and are not required before MVP development starts:

- database pool gauges;
- database query duration histograms;
- storage operation duration/error histograms;
- search index lag gauges;
- rate-limit counters when the selected runtime layer does not expose them.

### 2.3 Frontend Metrics

- Core Web Vitals (LCP, INP, CLS) — `web-vitals` library.
- API error rate.
- Query cache hit/miss (TanStack Query devtools).

Frontend metric export is a hardening item after the MVP UI/API flows stabilize.

### 2.4 Business Metrics

| Metric | Description |
|--------|-------------|
| `wiki_document_revisions_published_total` | published document revisions |
| `wiki_evidence_added_total` | added evidence records |
| `active_spaces` | spaces with activity; derived in analytics later |
| `active_users` | DAU/MAU; derived in analytics later |

## 3. Logging

### 3.1 Format

JSON structured logs:

```json
{
  "timestamp": "2026-07-13T10:00:00Z",
  "level": "INFO",
  "target": "wiki_api::document_service",
  "message": "document published",
  "request_id": "req-uuid",
  "user_id": "user-uuid",
  "space_id": "space-uuid",
  "document_id": "document-uuid",
  "duration_ms": 42
}
```

### 3.2 Levels

| Level | Use |
|-------|-----|
| ERROR | failures requiring attention |
| WARN | recoverable problems |
| INFO | significant operations |
| DEBUG | dev diagnostics |
| TRACE | very verbose |

### 3.3 Frontend Logs

- Console logs only in dev.
- Production: send errors to Sentry-compatible endpoint (future).

## 4. Tracing

- OpenTelemetry для распределённой трассировки.
- Trace ID прокидывается через `x-trace-id`.
- Spans: controller → service → repository → DB/storage.

## 5. Alerting

### 5.1 Critical

- API down > 1 min.
- DB unavailable.
- 5xx rate > 1%.
- Disk > 85%.

### 5.2 Warning

- 4xx rate > 10%.
- P95 latency > 1s.
- Background job failures > 5/hour.

## 6. Dashboards

| Dashboard | Panels |
|-----------|--------|
| API Overview | RPS, latency, errors, rate limits |
| Database | query time, pool, slow queries |
| Infrastructure | CPU, memory, disk, network |
| Product | DAU, published documents, evidence coverage |

## 7. Log Aggregation

- Promtail/Loki for backend logs.
- Grafana Alloy for frontend errors (future).
- Retention: 30 days hot, 1 year cold.

## 8. Health Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /api/v1/health` | current liveness |
| `GET /api/v1/health/ready` | persistent backend readiness |
| `GET /metrics` | Prometheus |

## 9. Uptime Monitoring

- Blackbox exporter или external service (UptimeRobot).
- Check `/api/v1/health` каждые 60s.

## 10. Profiling

- CPU/memory profiling через `pprof` (future).
- Async flamegraphs для Rust.

## 11. Delivery Observability

- Build duration.
- Test pass/fail rate.
- Deployment frequency.
- Mean time to recovery (MTTR).

These metrics describe Wiki delivery health only. Wiki does not run CI/CD pipelines.

## 12. Configuration

```yaml
# backend/config/monitoring.toml
[metrics]
enabled = true
bind = "0.0.0.0:9090"
endpoint = "/metrics"

[tracing]
enabled = true
exporter = "otlp"
otlp_endpoint = "http://tempo:4317"

[logging]
format = "json"
level = "info"
```

## 13. Local Monitoring

```bash
docker compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d
```

Доступ:

- Grafana: http://localhost:3001
- Prometheus: http://localhost:9090
- Loki: http://localhost:3100

## 14. Privacy

- Не логировать персональные данные.
- Не логировать пароли, токены, cookies.
- Маскировать email в логах.
## References

- `docs/ARCHITECTURE.md`
- `docs/DEPLOYMENT.md`
- `docs/PERFORMANCE.md`
