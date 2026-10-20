//! Native Wiki data plus an actual HTTP reader. The responder is a protocol fixture, not a live Tracker server.
use super::PostgresWikiBackend;
use pdlc_shared::{project_spaces::*, space_access::*};
use shared::{AppError, wiki_contract::WikiClaims};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
pub(super) async fn verify(
    backend: &mut PostgresWikiBackend,
    ordinary: &WikiClaims,
    independent: &WikiClaims,
    a: &SpaceRef,
    b: &SpaceRef,
    standalone: &SpaceRef,
) {
    let tracker: Uuid = std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
        .unwrap()
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}/", listener.local_addr().unwrap());
    let file = std::env::temp_dir().join(format!("wiki-canonical-reader-{}", Uuid::new_v4()));
    std::fs::write(&file, "synthetic-wiki-context-reader").unwrap();
    let names = [
        "WIKI_NAMESPACE__TRACKER_URL",
        "WIKI_NAMESPACE__TRACKER_TOKEN_FILE",
    ];
    let previous: Vec<_> = names.iter().map(|key| std::env::var_os(key)).collect();
    // This fresh isolated native test owns its process; no live configuration is changed.
    unsafe {
        std::env::set_var(names[0], origin);
        std::env::set_var(names[1], &file);
    }
    backend.task_reader = Some(Arc::new(
        super::managed_links::TaskReader::from_deployment()
            .unwrap()
            .unwrap(),
    ));
    let a = a.clone();
    let b = b.clone();
    let standalone = standalone.clone();
    let source_a = a.clone();
    let source_standalone = standalone.clone();
    let expected_actor = ordinary
        .admission
        .as_ref()
        .unwrap()
        .read_delegation()
        .unwrap()
        .identity;
    let server = tokio::spawn(async move {
        for index in 0..4 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let split;
            loop {
                if let Some(at) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                    split = at + 4;
                    break;
                }
                let mut chunk = [0; 4096];
                let n = stream.read(&mut chunk).await.unwrap();
                assert!(n > 0 && bytes.len() + n <= 1024 * 1024);
                bytes.extend_from_slice(&chunk[..n]);
            }
            let head = String::from_utf8(bytes[..split].to_vec()).unwrap();
            assert!(head.starts_with("POST /api/v2/project-contexts/query HTTP/1.1"));
            let fields: Vec<_> = head.lines().filter_map(|s| s.split_once(':')).collect();
            assert_eq!(
                fields
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
                    .unwrap()
                    .1
                    .trim(),
                "Bearer synthetic-wiki-context-reader"
            );
            let len: usize = fields
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                .unwrap()
                .1
                .trim()
                .parse()
                .unwrap();
            assert!(len <= 1024 * 1024);
            while bytes.len() - split < len {
                let mut chunk = [0; 4096];
                let n = stream.read(&mut chunk).await.unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&chunk[..n]);
            }
            let request: ProjectSpaceContextQuery =
                serde_json::from_slice(&bytes[split..split + len]).unwrap();
            assert!(request.valid_for(&expected_actor.issuer, tracker));
            assert_eq!(request.actor.identity, expected_actor);
            let ProjectContextSelector::Space {
                space,
                limit,
                offset,
            } = &request.selector
            else {
                panic!(
                    "Space collection must query the actual Space owner, not stale projection IDs"
                )
            };
            assert_eq!(*limit, 50);
            assert_eq!(*offset, 0);
            let mut actor = request.actor.clone();
            let empty = *space == standalone;
            if empty {
                assert_eq!(index, 1);
            } else {
                assert_eq!(*space, a);
            }
            if index == 2 {
                actor.validity = AdmissionValidity::confirmed(
                    chrono::Utc::now().timestamp() - 70,
                    chrono::Utc::now().timestamp() - 10,
                    i64::MAX,
                )
                .unwrap();
            }
            if index == 3 {
                actor.identity.subject = Uuid::new_v4().to_string();
            }
            let project = ProjectRef {
                tracker_instance_id: tracker,
                project_id: Uuid::new_v4(),
            };
            let projects = if empty {
                vec![]
            } else {
                vec![ProjectSpaceReadContext {
                    lifecycle: pdlc_shared::project_spaces::ProjectLifecycle {
                        schema_version: 1,
                        project: project.clone(),
                        revision: 1,
                        archived: false,
                    },
                    key: "FRESH".into(),
                    name: "Новая каноническая связь".into(),
                    links: ConfirmedProjectSpaces {
                        project,
                        revision: 10,
                        spaces: vec![a.clone(), b.clone()],
                        validity: actor.validity,
                    },
                }]
            };
            let response = ProjectSpaceContextBatch {
                schema_version: 1,
                policy_revision: request.policy_revision,
                total: projects.len() as u64,
                has_more: false,
                projects,
                actor,
            };
            let body = serde_json::to_vec(&response).unwrap();
            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
        }
    });
    let response = backend
        .canonical_space_projects(ordinary, "QA01", 50, 0)
        .await
        .unwrap();
    assert_eq!(response.space, source_a);
    assert_eq!(response.total, 1);
    assert_eq!(
        response.projects[0].key, "FRESH",
        "fresh Tracker association is visible even when Wiki's A projection is empty"
    );
    assert!(
        matches!(
            backend
                .canonical_space_projects(ordinary, "QA02", 50, 0)
                .await,
            Err(AppError::Forbidden)
        ),
        "A admission does not open B's collection or trigger a machine request for it"
    );
    let key = source_standalone
        .space_id
        .simple()
        .to_string()
        .to_ascii_uppercase();
    let empty = backend
        .canonical_space_projects(independent, &key, 50, 0)
        .await
        .unwrap();
    assert_eq!(empty.total, 0);
    assert!(empty.projects.is_empty());
    assert!(
        backend
            .canonical_space_projects(ordinary, "QA01", 50, 0)
            .await
            .is_err(),
        "expired canonical decision is never a successful stale result"
    );
    assert!(
        backend
            .canonical_space_projects(ordinary, "QA01", 50, 0)
            .await
            .is_err(),
        "foreign original actor is rejected"
    );
    server.await.unwrap();
    backend.task_reader = None;
    std::fs::remove_file(file).unwrap();
    for (key, old) in names.iter().zip(previous) {
        unsafe {
            if let Some(old) = old {
                std::env::set_var(key, old);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}
