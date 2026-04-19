use uuid::Uuid;
use vereinssoftware_backend::storage::{FileStorage, LocalFileStorage};

#[tokio::test]
async fn test_save_and_get_file() {
    let temp_dir = std::env::temp_dir().join("test_storage");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    let storage = LocalFileStorage::new(temp_dir.clone());
    let transaction_id = Uuid::new_v4();
    let test_content = b"test file content".to_vec();
    let filename = "test.txt";

    // Save file
    let result = storage.save(transaction_id, test_content.clone(), filename).await;
    assert!(result.is_ok(), "Failed to save file");

    let reference_path = result.unwrap();
    assert_eq!(reference_path, format!("receipts/{}/{}", transaction_id, filename));

    // Get file
    let retrieved = storage.get(&reference_path).await;
    assert!(retrieved.is_ok(), "Failed to get file");
    assert_eq!(retrieved.unwrap(), test_content);

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_delete_file() {
    let temp_dir = std::env::temp_dir().join("test_storage_delete");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    let storage = LocalFileStorage::new(temp_dir.clone());
    let transaction_id = Uuid::new_v4();
    let test_content = b"test content".to_vec();
    let filename = "test.txt";

    // Save file
    let reference_path = storage
        .save(transaction_id, test_content, filename)
        .await
        .unwrap();

    // Verify file exists
    let exists = storage.get(&reference_path).await;
    assert!(exists.is_ok(), "File should exist after save");

    // Delete file
    let delete_result = storage.delete(&reference_path).await;
    assert!(delete_result.is_ok(), "Failed to delete file");

    // Verify file is deleted
    let not_found = storage.get(&reference_path).await;
    assert!(not_found.is_err(), "File should not exist after delete");

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_path_traversal_protection() {
    let temp_dir = std::env::temp_dir().join("test_storage_traversal");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    let storage = LocalFileStorage::new(temp_dir.clone());

    // Try to access a file outside the base path
    let malicious_path = "../../../etc/passwd";
    let result = storage.get(malicious_path).await;
    assert!(result.is_err(), "Path traversal should be blocked");

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_invalid_filename() {
    let temp_dir = std::env::temp_dir().join("test_storage_invalid");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    let storage = LocalFileStorage::new(temp_dir.clone());
    let transaction_id = Uuid::new_v4();
    let test_content = b"test content".to_vec();

    // Test with ".." in filename
    let result = storage
        .save(transaction_id, test_content.clone(), "../malicious.txt")
        .await;
    assert!(result.is_err(), "Should reject filename with ..");

    // Test with "/" in filename
    let result = storage
        .save(transaction_id, test_content.clone(), "dir/file.txt")
        .await;
    assert!(result.is_err(), "Should reject filename with /");

    // Test with "\\" in filename
    let result = storage
        .save(transaction_id, test_content, "dir\\file.txt")
        .await;
    assert!(result.is_err(), "Should reject filename with \\");

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_multiple_files_same_transaction() {
    let temp_dir = std::env::temp_dir().join("test_storage_multi");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    let storage = LocalFileStorage::new(temp_dir.clone());
    let transaction_id = Uuid::new_v4();

    let content1 = b"content 1".to_vec();
    let content2 = b"content 2".to_vec();

    // Save multiple files
    let path1 = storage
        .save(transaction_id, content1.clone(), "file1.txt")
        .await
        .unwrap();
    let path2 = storage
        .save(transaction_id, content2.clone(), "file2.txt")
        .await
        .unwrap();

    // Retrieve both files
    let retrieved1 = storage.get(&path1).await.unwrap();
    let retrieved2 = storage.get(&path2).await.unwrap();

    assert_eq!(retrieved1, content1);
    assert_eq!(retrieved2, content2);

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
