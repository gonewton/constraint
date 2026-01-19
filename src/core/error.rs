//! Error types for the Newton Constraints CLI tool

use thiserror::Error;

/// Main error type for constraint operations
#[derive(Debug, Error)]
pub enum ConstraintError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Loader error: {0}")]
    Loader(#[from] LoaderError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Constraint not found: {id}")]
    NotFound { id: String },

    #[error("Invalid constraint type: {0}. Must be one of: MUST, SHALL, SHOULD, MAY, FORBIDDEN (RFC 2119)")]
    InvalidConstraintType(String),

    #[error("Invalid constraint ID format: {0}. Expected format: nt-<6-char-base36-suffix> (e.g., nt-a3f2k9)")]
    InvalidIdFormat(String),

    #[error("Constraint ID collision: {id} already exists")]
    #[allow(dead_code)]
    IdCollision { id: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Storage layer specific errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Load error: {0}")]
    #[allow(dead_code)]
    LoadError(String),

    #[error("Invalid constraint ID: {0}")]
    #[allow(dead_code)]
    InvalidId(String),

    #[error("File lock error: {0}")]
    #[allow(dead_code)]
    LockError(String),

    #[error("File corruption detected: {0}")]
    #[allow(dead_code)]
    Corruption(String),
}

/// Version loader specific errors
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    #[error("Unknown version: {0}")]
    UnknownVersion(u32),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    #[allow(dead_code)]
    Migration(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
