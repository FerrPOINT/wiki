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
                    request_id: None
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
                    request_id: None
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
