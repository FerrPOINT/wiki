use super::namespace::namespace_postgres_tests::{backend, create_command};
use super::{PostgresWikiBackend, workspace_catalog::validate_project};
use shared::{AppError, resource_context::*, wiki_contract::*};
use uuid::Uuid;

fn project(command: &OwnerCommand, name: &str, key: &str) -> ResourceContextSummary {
    let mut binding = command.readback(true);
    binding.resource = ResourceRef {
        kind: ResourceKind::TrackerProject,
        instance_id: std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
            .unwrap()
            .parse()
            .unwrap(),
        resource_id: Uuid::now_v7(),
    };
    ResourceContextSummary {
        binding,
        label: name.into(),
        resource_key: key.into(),
    }
}
impl PostgresWikiBackend {
    async fn test_replace_projects(
        &self,
        items: Result<Vec<ResourceContextSummary>, AppError>,
    ) -> Result<(), AppError> {
        self.replace_projects(items).await
    }
}

#[tokio::test]
#[ignore = "requires disposable PostgreSQL and QA namespace identities"]
async fn workspace_snapshot_preserves_identity_permissions_and_neighbor_projects() {
    let (backend, owner) = backend().await;
    sqlx::query("UPDATE users SET central_sub=$1 WHERE id=$2")
        .bind(format!("qa-central-{owner}"))
        .bind(owner)
        .execute(&backend.pool)
        .await
        .unwrap();
    let command = create_command(owner);
    backend.apply_namespace(&command).await.unwrap();
    let connected = project(&command, "Canonical project", "TRACKER");
    let mut unbound = project(&command, "Canonical project", "NEIGHBOR");
    unbound.binding.namespace.namespace_id = Uuid::now_v7();
    backend
        .test_replace_projects(Ok(vec![connected.clone(), unbound]))
        .await
        .unwrap();
    let claims = WikiClaims {
        user_id: owner.to_string(),
        session_id: None,
        request_id: None,
        admission: None,
    };
    let page = backend
        .list_projects(&claims, None, 100, 0, "")
        .await
        .unwrap();
    assert_eq!(page.total, 2);
    let bound = page
        .items
        .iter()
        .find(|p| p.namespace == command.namespace)
        .unwrap();
    assert_eq!(bound.name, "Canonical project");
    assert_eq!(bound.project_key, "TRACKER");
    assert_eq!(bound.binding.as_ref().unwrap().resource, command.resource);
    assert!(matches!(
        bound.resource_status,
        ProjectResourceStatus::Connected
    ));
    assert!(
        page.items
            .iter()
            .any(|p| matches!(p.resource_status, ProjectResourceStatus::NotConnected))
    );
    let foreign = Uuid::now_v7();
    sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role) VALUES($1,$2,$3,'Viewer','unused','user')").bind(foreign).bind(format!("{foreign}@example.test")).bind(foreign.to_string()).execute(&backend.pool).await.unwrap();
    assert!(matches!(
        backend
            .list_projects(
                &WikiClaims {
                    user_id: foreign.to_string(),
                    session_id: None,
                    request_id: None,
                    admission: None,
                },
                None,
                100,
                0,
                ""
            )
            .await,
        Err(AppError::Forbidden)
    ));
    sqlx::query("UPDATE users SET central_sub=$1 WHERE id=$2")
        .bind(format!("qa-central-{foreign}"))
        .bind(foreign)
        .execute(&backend.pool)
        .await
        .unwrap();
    let denied = backend
        .list_projects(
            &WikiClaims {
                user_id: foreign.to_string(),
                session_id: None,
                request_id: None,
                admission: None,
            },
            Some(&command.namespace),
            1,
            0,
            "",
        )
        .await
        .unwrap();
    assert!(matches!(
        denied.items[0].resource_status,
        ProjectResourceStatus::Forbidden
    ));
    assert!(denied.items[0].space_key.is_none() && denied.items[0].binding.is_none());
    let document = Uuid::now_v7();
    sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'context','Context document',$3)").bind(document).bind(command.resource.resource_id).bind(owner).execute(&backend.pool).await.unwrap();
    assert_eq!(
        backend
            .document_context(&claims, document)
            .await
            .unwrap()
            .unwrap()
            .binding
            .namespace,
        command.namespace
    );
    assert!(matches!(
        backend
            .document_context(
                &WikiClaims {
                    user_id: foreign.to_string(),
                    session_id: None,
                    request_id: None,
                    admission: None,
                },
                document
            )
            .await,
        Err(AppError::Forbidden)
    ));
    backend.pool.close().await;
}

#[tokio::test]
#[ignore = "requires disposable PostgreSQL and QA namespace identities"]
async fn snapshot_replacement_is_atomic_and_empty_snapshot_is_real() {
    let (backend, owner) = backend().await;
    sqlx::query("UPDATE users SET central_sub=$1 WHERE id=$2")
        .bind(format!("qa-central-{owner}"))
        .bind(owner)
        .execute(&backend.pool)
        .await
        .unwrap();
    let command = create_command(owner);
    backend.apply_namespace(&command).await.unwrap();
    let mut item = project(&command, "Retained", "KEEP");
    item.binding.generation = 2;
    backend
        .test_replace_projects(Ok(vec![item.clone()]))
        .await
        .unwrap();
    let claims = WikiClaims {
        user_id: owner.to_string(),
        session_id: None,
        request_id: None,
        admission: None,
    };
    item.binding.generation = 1;
    assert!(backend.test_replace_projects(Ok(vec![item])).await.is_err());
    assert_eq!(
        backend
            .list_projects(&claims, None, 100, 0, "")
            .await
            .unwrap()
            .total,
        1
    );
    assert!(
        backend
            .test_replace_projects(Err(AppError::Unavailable("page_two_failed".into())))
            .await
            .is_err()
    );
    assert_eq!(
        backend
            .list_projects(&claims, None, 100, 0, "")
            .await
            .unwrap()
            .total,
        1
    );
    backend.test_replace_projects(Ok(vec![])).await.unwrap();
    assert_eq!(
        backend
            .list_projects(&claims, None, 100, 0, "")
            .await
            .unwrap()
            .total,
        0
    );
    assert!(
        backend
            .namespace_binding(command.resource.resource_id)
            .await
            .unwrap()
            .is_some()
    );
    backend.pool.close().await;
}

#[test]
fn presentation_reader_rejects_foreign_and_invalid_bindings() {
    let registry = Uuid::new_v4();
    let instance = Uuid::new_v4();
    let command = OwnerCommand {
        schema_version: 1,
        namespace: NamespaceRef {
            registry_instance_id: registry,
            namespace_id: Uuid::new_v4(),
        },
        resource: ResourceRef {
            kind: ResourceKind::TrackerProject,
            instance_id: instance,
            resource_id: Uuid::new_v4(),
        },
        operation_id: Uuid::new_v4(),
        generation: 1,
        state: "active".into(),
        create_spec: None,
    };
    let mut item = ResourceContextSummary {
        binding: command.readback(true),
        label: "Project".into(),
        resource_key: "KEY".into(),
    };
    assert!(validate_project(&item, registry, instance).is_ok());
    assert!(validate_project(&item, Uuid::new_v4(), instance).is_err());
    item.binding.resource.kind = ResourceKind::WikiSpace;
    assert!(validate_project(&item, registry, instance).is_err());
}

#[tokio::test]
#[ignore = "requires disposable PostgreSQL and QA namespace identities"]
async fn space_projects_are_many_to_many_without_transferring_acl_or_content() {
    let (backend, owner) = backend().await;
    sqlx::query("UPDATE users SET central_sub=$1 WHERE id=$2")
        .bind(format!("qa-central-{owner}"))
        .bind(owner)
        .execute(&backend.pool)
        .await
        .unwrap();
    let command = create_command(owner);
    backend.apply_namespace(&command).await.unwrap();
    let space = command.resource.resource_id;
    let key: String = sqlx::query_scalar("SELECT key FROM spaces WHERE id=$1")
        .bind(space)
        .fetch_one(&backend.pool)
        .await
        .unwrap();
    let first = project(&command, "First project", "ONE");
    let mut second = project(&command, "Second project", "TWO");
    second.binding.namespace.namespace_id = Uuid::now_v7();
    backend
        .replace_projects(Ok(vec![first.clone(), second.clone()]))
        .await
        .unwrap();
    let claims = WikiClaims {
        user_id: owner.to_string(),
        session_id: None,
        request_id: Some("qa-space-projects".into()),
        admission: None,
    };
    let independent = Uuid::now_v7();
    let independent_key =
        format!("INDEPENDENT{}", &independent.simple().to_string()[..16]).to_uppercase();
    sqlx::query("INSERT INTO spaces(id,key,name,owner_id) VALUES($1,$2,'Independent', $3)")
        .bind(independent)
        .bind(&independent_key)
        .bind(owner)
        .execute(&backend.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'admin')")
        .bind(independent)
        .bind(owner)
        .execute(&backend.pool)
        .await
        .unwrap();
    assert!(
        backend
            .linked_projects(&claims, &independent_key)
            .await
            .unwrap()
            .projects
            .is_empty()
    );
    backend
        .associate_project(&claims, &key, &second.binding.namespace, true)
        .await
        .unwrap();
    backend
        .associate_project(&claims, &key, &second.binding.namespace, true)
        .await
        .unwrap();
    backend
        .associate_project(&claims, &independent_key, &first.binding.namespace, true)
        .await
        .unwrap();
    assert_eq!(
        backend
            .linked_projects(&claims, &key)
            .await
            .unwrap()
            .projects
            .len(),
        2
    );
    let page = backend
        .list_projects(&claims, Some(&first.binding.namespace), 1, 0, "")
        .await
        .unwrap();
    assert_eq!(page.items[0].spaces.len(), 2);
    let audits: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_log WHERE entity_id=$1 AND action='space.project_link'",
    )
    .bind(space)
    .fetch_one(&backend.pool)
    .await
    .unwrap();
    assert_eq!(audits, 1, "repeated link must not duplicate audit");
    let document = Uuid::now_v7();
    sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'independent','Independent content',$3)").bind(document).bind(space).bind(owner).execute(&backend.pool).await.unwrap();
    assert_eq!(
        backend
            .document_projects(&claims, document)
            .await
            .unwrap()
            .projects
            .len(),
        2
    );
    let foreign = Uuid::now_v7();
    sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,central_sub) VALUES($1,$2,$3,'Other viewer','unused',$4)").bind(foreign).bind(format!("{foreign}@example.test")).bind(foreign.to_string()).bind(format!("central-{foreign}")).execute(&backend.pool).await.unwrap();
    let outsider = WikiClaims {
        user_id: foreign.to_string(),
        session_id: None,
        request_id: None,
        admission: None,
    };
    assert!(matches!(
        backend.linked_projects(&outsider, &key).await,
        Err(AppError::Forbidden)
    ));
    let denied = backend
        .list_projects(&outsider, Some(&first.binding.namespace), 1, 0, "")
        .await
        .unwrap();
    assert!(
        denied.items[0].spaces.is_empty(),
        "project membership must not expose Wiki spaces"
    );
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'viewer')")
        .bind(space)
        .bind(foreign)
        .execute(&backend.pool)
        .await
        .unwrap();
    assert_eq!(
        backend
            .linked_projects(&outsider, &key)
            .await
            .unwrap()
            .projects
            .len(),
        2
    );
    assert!(matches!(
        backend
            .associate_project(&outsider, &key, &second.binding.namespace, false)
            .await,
        Err(AppError::Forbidden)
    ));
    let mut archived = command.clone();
    archived.operation_id = Uuid::now_v7();
    archived.generation = 2;
    archived.state = "archived".into();
    archived.create_spec = None;
    backend.apply_namespace(&archived).await.unwrap();
    sqlx::query("UPDATE documents SET title='Still editable' WHERE id=$1")
        .bind(document)
        .execute(&backend.pool)
        .await
        .unwrap();
    // Existing links remain idempotent even while the external catalog is absent.
    backend.replace_projects(Ok(vec![])).await.unwrap();
    backend
        .associate_project(&claims, &key, &second.binding.namespace, true)
        .await
        .unwrap();
    backend
        .associate_project(&claims, &key, &first.binding.namespace, false)
        .await
        .unwrap();
    backend
        .associate_project(&claims, &key, &second.binding.namespace, false)
        .await
        .unwrap();
    assert!(
        backend
            .linked_projects(&claims, &key)
            .await
            .unwrap()
            .projects
            .is_empty()
    );
    backend.apply_namespace(&archived).await.unwrap();
    assert!(
        backend
            .linked_projects(&claims, &key)
            .await
            .unwrap()
            .projects
            .is_empty(),
        "provisioning replay must not restore removed associations"
    );
    let title: String = sqlx::query_scalar("SELECT title FROM documents WHERE id=$1")
        .bind(document)
        .fetch_one(&backend.pool)
        .await
        .unwrap();
    assert_eq!(title, "Still editable");
    sqlx::query("UPDATE spaces SET archived_at=now() WHERE id=$1")
        .bind(space)
        .execute(&backend.pool)
        .await
        .unwrap();
    assert!(
        backend
            .associate_project(&claims, &key, &first.binding.namespace, true)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE documents SET title='Blocked' WHERE id=$1")
            .bind(document)
            .execute(&backend.pool)
            .await
            .is_err()
    );
    backend.pool.close().await;
}
