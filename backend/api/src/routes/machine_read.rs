//! Closed classification of existing document-reader routes. No owner/admin surface.
pub(super) fn allowed(method: &axum::http::Method, path: Option<&str>) -> bool {
    if method != axum::http::Method::GET {
        return false;
    }
    matches!(
        path,
        Some(
            "/api/v1/spaces"
                | "/api/v1/spaces/{space_key}"
                | "/api/v1/spaces/{space_key}/tree"
                | "/api/v1/spaces/{space_key}/projects"
                | "/api/v2/spaces/{space_key}/projects"
                | "/api/v1/documents/{document_id}"
                | "/api/v1/documents/{document_id}/revisions"
                | "/api/v1/documents/{document_id}/revisions/{revision_id}"
                | "/api/v1/spaces/{space_key}/tasks/{task_key}/documents"
                | "/api/v1/spaces/{space_key}/tasks/{task_key}/evidence"
                | "/api/v1/evidence"
                | "/api/v1/evidence/{evidence_id}"
                | "/api/v1/attachments/{attachment_id}"
                | "/api/v1/attachments/{attachment_id}/download"
                | "/api/v1/search"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::allowed;
    use axum::http::Method;

    #[test]
    fn only_classified_read_routes_accept_machine_readers() {
        for route in [
            "/api/v1/search",
            "/api/v1/documents/{document_id}",
            "/api/v1/attachments/{attachment_id}/download",
            "/api/v2/spaces/{space_key}/projects",
        ] {
            assert!(allowed(&Method::GET, Some(route)));
            for method in [
                Method::PUT,
                Method::POST,
                Method::DELETE,
                Method::PATCH,
                Method::HEAD,
            ] {
                assert!(!allowed(&method, Some(route)));
            }
        }
        for route in [
            "/api/v1/users",
            "/api/v1/settings",
            "/api/v1/audit-log",
            "/api/v1/spaces/{space_key}/members",
            "/api/v1/namespace-resources/wiki_space/{id}",
            "/api/v1/documents/{document_id}/draft",
            "/api/v1/space-admission",
            "/api/v1/search/anything",
        ] {
            assert!(!allowed(&Method::GET, Some(route)));
        }
        assert!(!allowed(&Method::GET, None));
    }
}
