# Metrics - Wiki

## 1. Implemented MVP Metrics

`GET /metrics` exposes Prometheus text outside the versioned `/api/v1` OpenAPI contract.

### 1.1 HTTP

The API uses `axum-prometheus` for route-level HTTP metrics:

- `axum_http_requests_total{method,endpoint,status}`
- `axum_http_requests_duration_seconds{method,endpoint,status}`
- `axum_http_requests_pending{method,endpoint}`

### 1.2 Product And Security Counters

The API records successful MVP write/search operations and 403 responses:

- `wiki_auth_login_attempts_total{result}` where `result` is `success`, `failure` or `error`.
- `wiki_users_created_total`.
- `wiki_spaces_created_total`.
- `wiki_documents_created_total{document_type}`.
- `wiki_document_revisions_published_total`.
- `wiki_documents_archived_total`.
- `wiki_task_document_links_total`.
- `wiki_phase_document_links_total`.
- `wiki_evidence_added_total{source_type}`.
- `wiki_attachments_uploaded_total`.
- `wiki_attachment_upload_bytes_total`.
- `wiki_templates_created_total`.
- `wiki_search_queries_total{scope}` where `scope` is `global`, `space`, `task` or `phase`.
- `wiki_permission_denied_total{scope}` for API requests returned as 403.

Counters are process-local and are persisted by the Prometheus scrape/storage layer, not by Wiki itself.

## 2. Hardening Backlog

These metrics are planned after MVP runtime patterns settle and must not be treated as shipped MVP contracts:

- database pool gauges;
- per-query database duration histograms;
- per-storage-backend duration/error histograms;
- search index lag gauges;
- frontend Core Web Vitals export;
- rate-limit counters if the selected gateway/runtime does not already expose them.
