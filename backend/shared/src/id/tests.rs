use crate::{
    AttachmentId, AuditLogId, DocumentId, DocumentRevisionId, DocumentTemplateId, EvidenceId,
    PhaseDossierId, SpaceId, TaskDossierId, UserId,
};
use std::str::FromStr;

type RoundtripCase = (Box<dyn Fn() -> String>, Box<dyn Fn(&str) -> bool>);

#[test]
fn all_uuid_ids_roundtrip_and_nil() {
    let cases: Vec<RoundtripCase> = vec![
        (
            Box::new(|| UserId::new().to_string()),
            Box::new(|s| UserId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| AttachmentId::new().to_string()),
            Box::new(|s| AttachmentId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| AuditLogId::new().to_string()),
            Box::new(|s| AuditLogId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| SpaceId::new().to_string()),
            Box::new(|s| SpaceId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| DocumentId::new().to_string()),
            Box::new(|s| DocumentId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| DocumentRevisionId::new().to_string()),
            Box::new(|s| DocumentRevisionId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| DocumentTemplateId::new().to_string()),
            Box::new(|s| DocumentTemplateId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| TaskDossierId::new().to_string()),
            Box::new(|s| TaskDossierId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| PhaseDossierId::new().to_string()),
            Box::new(|s| PhaseDossierId::from_str(s).is_ok()),
        ),
        (
            Box::new(|| EvidenceId::new().to_string()),
            Box::new(|s| EvidenceId::from_str(s).is_ok()),
        ),
    ];
    for (maker, parse) in cases {
        let s = maker();
        assert!(parse(&s), "roundtrip failed for {}", s);
    }
    assert_eq!(
        UserId::nil().to_string(),
        "00000000-0000-0000-0000-000000000000"
    );
}

#[test]
fn uuid_id_rejects_invalid() {
    assert!(UserId::from_str("not-a-uuid").is_err());
    assert!(DocumentId::from_str("").is_err());
}
