use std::str::FromStr;

use shared::{AttachmentId, DocumentId, EvidenceId, SpaceId, UserId};

use crate::wiki::{
    AttachmentMetadata, AttachmentOwnerType, Document, DocumentRevision, DocumentSlug,
    DocumentStatus, DocumentType, EvidenceItem, EvidenceTarget, EvidenceType, PhaseKey, Space,
    SpaceKey, SpaceRole, TaskKey,
};

#[test]
fn space_key_normalizes_and_rejects_invalid_values() {
    assert_eq!(SpaceKey::parse("eng").unwrap().as_str(), "ENG");
    assert!(SpaceKey::parse("E").is_err());
    assert!(SpaceKey::parse("-ENG").is_err());
    assert!(SpaceKey::parse("ENG_1").is_err());
}

#[test]
fn document_slug_is_lowercase_route_segment() {
    assert_eq!(
        DocumentSlug::parse("Product-Requirements")
            .unwrap()
            .as_str(),
        "product-requirements"
    );
    assert!(DocumentSlug::parse("bad slug").is_err());
    assert!(DocumentSlug::parse("bad--slug").is_err());
    assert!(DocumentSlug::parse("-bad").is_err());
}

#[test]
fn task_and_phase_keys_validate_route_safe_values() {
    assert_eq!(TaskKey::parse("SDLC-42").unwrap().as_str(), "SDLC-42");
    assert!(TaskKey::parse("SDLC 42").is_err());
    assert_eq!(
        PhaseKey::parse("Implementation").unwrap().as_str(),
        "implementation"
    );
    assert!(PhaseKey::parse("_implementation").is_err());
    assert!(PhaseKey::parse("release phase").is_err());
}

#[test]
fn role_and_type_enums_match_api_values() {
    assert_eq!(SpaceRole::from_str("editor").unwrap(), SpaceRole::Editor);
    assert!(SpaceRole::Editor.can_write());
    assert!(!SpaceRole::Viewer.can_write());
    assert_eq!(
        DocumentType::from_str("implementation_note").unwrap(),
        DocumentType::ImplementationNote
    );
    assert_eq!(EvidenceType::ExternalUrl.as_str(), "external_url");
    assert_eq!(EvidenceType::UploadedFile.as_str(), "uploaded_file");
}

#[test]
fn document_lifecycle_enforces_base_invariants() {
    let owner_id = UserId::new();
    let space = Space::create(
        SpaceKey::parse("ENG").unwrap(),
        "Engineering",
        "Internal docs",
        owner_id,
    )
    .unwrap();
    let mut document = Document::create(
        space.id,
        None,
        DocumentSlug::parse("requirements").unwrap(),
        "Requirements",
        DocumentType::Requirements,
        owner_id,
    )
    .unwrap();

    assert_eq!(document.status, DocumentStatus::Draft);
    assert!(document.move_to(Some(document.id)).is_err());
    let revision = DocumentRevision::publish(
        &document,
        1,
        "# Requirements",
        "<h1>Requirements</h1>",
        "Requirements",
        "abc123",
        Some("Initial publish".into()),
        owner_id,
    )
    .unwrap();

    document.mark_published(revision.id);
    assert_eq!(document.status, DocumentStatus::Published);
    assert_eq!(document.current_revision_id, Some(revision.id));

    document.archive();
    assert_eq!(document.status, DocumentStatus::Archived);
    assert!(document.archived_at.is_some());
}

#[test]
fn publish_rejects_empty_body_and_non_positive_version() {
    let document = Document::create(
        SpaceId::new(),
        None,
        DocumentSlug::parse("requirements").unwrap(),
        "Requirements",
        DocumentType::Requirements,
        UserId::new(),
    )
    .unwrap();
    assert!(
        DocumentRevision::publish(
            &document,
            0,
            "# Body",
            "<h1>Body</h1>",
            "Body",
            "hash",
            None,
            UserId::new()
        )
        .is_err()
    );
    assert!(
        DocumentRevision::publish(&document, 1, "   ", "", "", "hash", None, UserId::new())
            .is_err()
    );
}

#[test]
fn evidence_requires_target_and_matching_payload() {
    let space_id = SpaceId::new();
    let user_id = UserId::new();
    let document_id = DocumentId::new();
    let target = EvidenceTarget::document(document_id);

    let external = EvidenceItem::external_url(
        space_id,
        target,
        "CI run",
        "https://ci.example/jobs/1",
        user_id,
    )
    .unwrap();
    assert_eq!(external.evidence_type, EvidenceType::ExternalUrl);
    assert!(external.url.is_some());
    assert!(external.attachment_id.is_none());

    let uploaded = EvidenceItem::uploaded_file(
        space_id,
        target,
        "Test artifact",
        AttachmentId::new(),
        "sha256",
        user_id,
    )
    .unwrap();
    assert_eq!(uploaded.evidence_type, EvidenceType::UploadedFile);
    assert!(uploaded.url.is_none());
    assert!(uploaded.attachment_id.is_some());

    assert!(
        EvidenceItem::external_url(
            space_id,
            EvidenceTarget::default(),
            "Bad",
            "https://x",
            user_id
        )
        .is_err()
    );
}

#[test]
fn staged_attachment_can_be_claimed_for_file_evidence() {
    let space_id = SpaceId::new();
    let mut attachment = AttachmentMetadata::staged(
        "result.txt",
        "text/plain",
        12,
        "attachments/staged/result.txt",
        "sha256",
        UserId::new(),
    )
    .unwrap();

    assert!(attachment.owner_entity_type.is_none());
    let evidence_id = EvidenceId::new();
    attachment.claim_for_evidence(space_id, evidence_id);

    assert_eq!(attachment.space_id, Some(space_id));
    assert_eq!(
        attachment.owner_entity_type,
        Some(AttachmentOwnerType::Evidence)
    );
    assert_eq!(attachment.owner_entity_id, Some(evidence_id.as_uuid()));
}

#[test]
fn staged_attachment_rejects_empty_or_unsafe_metadata() {
    assert!(
        AttachmentMetadata::staged(
            "",
            "text/plain",
            12,
            "attachments/staged/a.txt",
            "sha256",
            UserId::new()
        )
        .is_err()
    );
    assert!(
        AttachmentMetadata::staged(
            "../secret.txt",
            "text/plain",
            12,
            "attachments/staged/a.txt",
            "sha256",
            UserId::new()
        )
        .is_err()
    );
    assert!(
        AttachmentMetadata::staged(
            "a.txt",
            "",
            12,
            "attachments/staged/a.txt",
            "sha256",
            UserId::new()
        )
        .is_err()
    );
    assert!(
        AttachmentMetadata::staged(
            "a.txt",
            "text/plain",
            0,
            "attachments/staged/a.txt",
            "sha256",
            UserId::new()
        )
        .is_err()
    );
}
