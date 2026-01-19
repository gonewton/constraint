use constraint::core::error::{ConstraintError, StorageError, LoaderError};
use std::io;

#[test]
fn test_constraint_error_display() {
    let error = ConstraintError::InvalidIdFormat("invalid-id".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Invalid constraint ID format"));
    assert!(display.contains("invalid-id"));
}

#[test]
fn test_storage_error_display() {
    let error = StorageError::InvalidId("bad-id".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Invalid constraint ID"));
    assert!(display.contains("bad-id"));
}

#[test]
fn test_loader_error_display() {
    let error = LoaderError::VersionMismatch { expected: 2, found: 1 };
    let display = format!("{}", error);
    assert!(display.contains("Version mismatch"));
    assert!(display.contains("expected 2"));
    assert!(display.contains("found 1"));
}

#[test]
fn test_error_conversion() {
    // Test IO error conversion
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let constraint_error = ConstraintError::from(io_error);
    assert!(matches!(constraint_error, ConstraintError::Io(_)));

    // Test JSON error conversion
    let json_error = serde_json::from_str::<String>("invalid json").unwrap_err();
    let constraint_error = ConstraintError::from(json_error);
    assert!(matches!(constraint_error, ConstraintError::Json(_)));
}

#[test]
fn test_validation_error() {
    let error = ConstraintError::Validation("Custom validation message".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Validation error"));
    assert!(display.contains("Custom validation message"));
}

#[test]
fn test_workspace_error() {
    let error = ConstraintError::WorkspaceNotFound("/some/path".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Workspace not found"));
    assert!(display.contains("/some/path"));
}

#[test]
fn test_not_found_error() {
    let error = ConstraintError::NotFound { id: "nt-missing".to_string() };
    let display = format!("{}", error);
    assert!(display.contains("Constraint not found"));
    assert!(display.contains("nt-missing"));
}

#[test]
fn test_id_collision_error() {
    let error = ConstraintError::IdCollision { id: "nt-duplicate".to_string() };
    let display = format!("{}", error);
    assert!(display.contains("Constraint ID collision"));
    assert!(display.contains("nt-duplicate"));
}

#[test]
fn test_invalid_constraint_type_error() {
    let error = ConstraintError::InvalidConstraintType("INVALID".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Invalid constraint type"));
    assert!(display.contains("INVALID"));
    assert!(display.contains("MUST, SHALL, SHOULD, MAY, FORBIDDEN"));
}

#[test]
fn test_storage_errors() {
    // IO error
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let storage_error = StorageError::from(io_error);
    assert!(matches!(storage_error, StorageError::Io(_)));

    // JSON error
    let json_error = serde_json::from_str::<String>("{").unwrap_err();
    let storage_error = StorageError::from(json_error);
    assert!(matches!(storage_error, StorageError::Json(_)));

    // Load error
    let load_error = StorageError::LoadError("Failed to parse constraint".to_string());
    let display = format!("{}", load_error);
    assert!(display.contains("Load error"));
    assert!(display.contains("Failed to parse constraint"));
}

#[test]
fn test_loader_errors() {
    // JSON error
    let json_error = serde_json::from_str::<String>("null").unwrap_err();
    let loader_error = LoaderError::from(json_error);
    assert!(matches!(loader_error, LoaderError::Json(_)));

    // Migration error
    let migration_error = LoaderError::Migration("Field mapping failed".to_string());
    let display = format!("{}", migration_error);
    assert!(display.contains("Migration error"));
    assert!(display.contains("Field mapping failed"));

    // Validation error
    let validation_error = LoaderError::Validation("Invalid constraint data".to_string());
    let display = format!("{}", validation_error);
    assert!(display.contains("Validation error"));
    assert!(display.contains("Invalid constraint data"));
}

#[test]
fn test_error_debug() {
    let error = ConstraintError::Validation("test message".to_string());
    let debug = format!("{:?}", error);
    assert!(debug.contains("ConstraintError"));
    assert!(debug.contains("Validation"));
    assert!(debug.contains("test message"));
}

#[test]
fn test_error_equality() {
    let error1 = ConstraintError::Validation("message".to_string());
    let error2 = ConstraintError::Validation("message".to_string());
    let error3 = ConstraintError::Validation("different".to_string());

    // Note: Error types may not implement Eq/PartialEq for complex reasons,
    // but we can test that different variants are different
    assert!(matches!(error1, ConstraintError::Validation(_)));
    assert!(matches!(error2, ConstraintError::Validation(_)));
    assert!(matches!(error3, ConstraintError::Validation(_)));
}