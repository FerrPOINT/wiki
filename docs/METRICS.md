# Metrics - Wiki

## 1. HTTP

- `http_requests_total{method,route,status}`
- `http_request_duration_seconds{method,route}`
- `http_request_body_bytes`
- `http_response_body_bytes`

## 2. Product

- `spaces_total`
- `documents_total{status}`
- `document_revisions_total`
- `task_dossiers_total{source_system}`
- `phase_dossiers_total{state}`
- `evidence_items_total{source_type}`
- `attachments_total`

## 3. Indexing And Maintenance

- `search_index_lag_seconds`
- `file_preview_jobs_total{status}`

## 4. Storage

- `storage_operation_duration_seconds{operation,backend}`
- `storage_errors_total{operation,backend}`
- `attachment_bytes_total`

## 5. Security

- `auth_login_attempts_total{result}`
- `permission_denied_total{entity_type}`
- `api_tokens_total{scope}`
