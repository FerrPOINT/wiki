use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskRef {
    pub tracker_instance_id: Uuid,
    pub task_id: Uuid,
}
#[derive(Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkTaskRevision {
    pub task: TaskRef,
    pub document_id: Uuid,
    pub revision_id: Uuid,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskRevisionLinkResponse {
    pub document_id: Uuid,
    pub revision_id: Uuid,
    pub title: String,
    pub version: i32,
    pub space_key: String,
    pub task_key: String,
    pub namespace: crate::resource_context::NamespaceRef,
}
