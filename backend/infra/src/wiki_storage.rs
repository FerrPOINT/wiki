use std::path::{Component, Path, PathBuf};

use shared::AppError;

#[derive(Clone, Debug)]
pub struct LocalWikiAttachmentStorage {
    root: PathBuf,
}

impl LocalWikiAttachmentStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path_for(&self, storage_key: &str) -> Result<PathBuf, AppError> {
        if storage_key
            .chars()
            .any(|ch| ch == '\\' || ch == '\0' || ch.is_control())
        {
            return Err(AppError::invalid_input("attachment storage key is invalid"));
        }

        let relative = Path::new(storage_key);
        let mut parts = Vec::new();

        for component in relative.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_owned()),
                Component::CurDir => {}
                Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                    return Err(AppError::invalid_input("attachment storage key is invalid"));
                }
            }
        }

        if parts.is_empty() {
            return Err(AppError::invalid_input(
                "attachment storage key is required",
            ));
        }

        Ok(parts
            .into_iter()
            .fold(self.root.clone(), |path, part| path.join(part)))
    }
}

#[async_trait::async_trait]
impl domain::wiki::WikiAttachmentStorage for LocalWikiAttachmentStorage {
    async fn put(&self, storage_key: &str, bytes: &[u8]) -> Result<(), AppError> {
        if bytes.is_empty() {
            return Err(AppError::invalid_input("file is required"));
        }
        let path = self.path_for(storage_key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::internal)?;
        }
        tokio::fs::write(&path, bytes)
            .await
            .map_err(AppError::internal)
    }

    async fn get(&self, storage_key: &str) -> Result<Vec<u8>, AppError> {
        let path = self.path_for(storage_key)?;
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(AppError::not_found("attachment file", storage_key))
            }
            Err(err) => Err(AppError::internal(err)),
        }
    }

    async fn delete(&self, storage_key: &str) -> Result<(), AppError> {
        let path = self.path_for(storage_key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AppError::internal(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::wiki::WikiAttachmentStorage;
    use uuid::Uuid;

    #[tokio::test]
    async fn local_wiki_attachment_storage_round_trips_bytes() {
        let root = std::env::temp_dir().join(format!("wiki-storage-{}", Uuid::now_v7()));
        let storage = LocalWikiAttachmentStorage::new(&root);

        storage
            .put("attachments/item/result.txt", b"result")
            .await
            .unwrap();
        assert_eq!(
            storage.get("attachments/item/result.txt").await.unwrap(),
            b"result"
        );
        storage.delete("attachments/item/result.txt").await.unwrap();
        assert!(storage.get("attachments/item/result.txt").await.is_err());

        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn local_wiki_attachment_storage_rejects_unsafe_keys() {
        let root = std::env::temp_dir().join(format!("wiki-storage-{}", Uuid::now_v7()));
        let storage = LocalWikiAttachmentStorage::new(root);

        assert!(storage.put("../secret.txt", b"secret").await.is_err());
        assert!(storage.put("/absolute.txt", b"secret").await.is_err());
        assert!(
            storage
                .put("attachments\\secret.txt", b"secret")
                .await
                .is_err()
        );
        assert!(
            storage
                .put("attachments/\nsecret.txt", b"secret")
                .await
                .is_err()
        );
        assert!(storage.put("", b"secret").await.is_err());
    }
}
