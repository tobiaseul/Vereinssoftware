use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

use super::{FileStorage, StorageError};

pub struct LocalFileStorage {
    base_path: PathBuf,
}

impl LocalFileStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn validate_filename(filename: &str) -> Result<(), StorageError> {
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(StorageError::InvalidPath(
                "Filename contains invalid path characters".to_string(),
            ));
        }
        if filename.is_empty() {
            return Err(StorageError::InvalidPath(
                "Filename cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn check_path_traversal(&self, target: &Path) -> Result<(), StorageError> {
        let canonical_base = self.base_path.canonicalize().map_err(|e| {
            StorageError::IoError(format!("Failed to canonicalize base path: {}", e))
        })?;

        // For paths that don't exist yet, canonicalize the parent directory
        let target_canonical = if target.exists() {
            target.canonicalize().map_err(|e| {
                StorageError::IoError(format!("Failed to canonicalize target path: {}", e))
            })?
        } else {
            // Get parent and canonicalize it, then append the filename
            let parent = target
                .parent()
                .ok_or_else(|| StorageError::InvalidPath("No parent directory".to_string()))?;

            let canonical_parent = if parent.exists() {
                parent.canonicalize().map_err(|e| {
                    StorageError::IoError(format!("Failed to canonicalize parent path: {}", e))
                })?
            } else {
                // If parent doesn't exist, check its ancestors
                return Ok(());
            };

            canonical_parent.join(target.file_name().ok_or_else(|| {
                StorageError::InvalidPath("No filename".to_string())
            })?)
        };

        if !target_canonical.starts_with(&canonical_base) {
            return Err(StorageError::InvalidPath(
                "Path traversal attempt detected".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn save(
        &self,
        transaction_id: Uuid,
        file_bytes: Vec<u8>,
        filename: &str,
    ) -> Result<String, StorageError> {
        Self::validate_filename(filename)?;

        let dir = self.base_path.join("receipts").join(transaction_id.to_string());
        fs::create_dir_all(&dir).await.map_err(|e| {
            StorageError::IoError(format!("Failed to create directory: {}", e))
        })?;

        let file_path = dir.join(filename);

        self.check_path_traversal(&file_path)?;

        fs::write(&file_path, file_bytes)
            .await
            .map_err(|e| StorageError::IoError(format!("Failed to write file: {}", e)))?;

        let relative_path = format!("receipts/{}/{}", transaction_id, filename);
        Ok(relative_path)
    }

    async fn get(&self, reference_path: &str) -> Result<Vec<u8>, StorageError> {
        let file_path = self.base_path.join(reference_path);

        self.check_path_traversal(&file_path)?;

        fs::read(&file_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound(format!("File not found: {}", reference_path))
            } else {
                StorageError::IoError(format!("Failed to read file: {}", e))
            }
        })
    }

    async fn delete(&self, reference_path: &str) -> Result<(), StorageError> {
        let file_path = self.base_path.join(reference_path);

        self.check_path_traversal(&file_path)?;

        fs::remove_file(&file_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound(format!("File not found: {}", reference_path))
            } else {
                StorageError::IoError(format!("Failed to delete file: {}", e))
            }
        })
    }
}
