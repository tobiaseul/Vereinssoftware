pub mod local;

pub use local::LocalFileStorage;

use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum StorageError {
    IoError(String),
    NotFound(String),
    InvalidPath(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(msg) => write!(f, "IO Error: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            StorageError::InvalidPath(msg) => write!(f, "Invalid Path: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn save(
        &self,
        transaction_id: Uuid,
        file_bytes: Vec<u8>,
        filename: &str,
    ) -> Result<String, StorageError>;

    async fn get(&self, reference_path: &str) -> Result<Vec<u8>, StorageError>;

    async fn delete(&self, reference_path: &str) -> Result<(), StorageError>;
}
