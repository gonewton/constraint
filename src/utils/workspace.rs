//! Workspace discovery and management utilities

use crate::core::error::ConstraintError;
use std::path::{Path, PathBuf};

/// Workspace management for constraint storage
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Create a new workspace with the given root path
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Discover workspace starting from current directory
    pub fn discover() -> Result<Self, ConstraintError> {
        let current = std::env::current_dir()
            .map_err(|e| ConstraintError::WorkspaceNotFound(e.to_string()))?;

        Self::find_workspace_root(&current)
    }

    /// Find workspace root by looking for .newton directory
    fn find_workspace_root(start_path: &Path) -> Result<Self, ConstraintError> {
        let mut current = start_path.to_path_buf();

        loop {
            let newton_dir = current.join(".newton");

            if newton_dir.exists() && newton_dir.is_dir() {
                return Ok(Self { root: newton_dir });
            }

            // Try to go up one directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                // Reached filesystem root without finding .newton
                return Err(ConstraintError::WorkspaceNotFound(
                    "No .newton directory found in current directory or parents".to_string(),
                ));
            }
        }
    }

    /// Get the constraints storage directory
    pub fn constraints_dir(&self) -> PathBuf {
        self.root.join("constraints")
    }

    /// Get the path for a specific category directory
    pub fn category_dir(&self, category: &str) -> PathBuf {
        self.constraints_dir().join(category)
    }

    /// Get the path for a specific constraint file
    #[allow(unused)]
    pub fn constraint_file(&self, category: &str, id: &str) -> Result<PathBuf, ConstraintError> {
        // Validate ID format
        if !crate::utils::id::IdGenerator::validate(id) {
            return Err(ConstraintError::InvalidIdFormat(id.to_string()));
        }

        Ok(self.category_dir(category).join(format!("{}.jsonl", id)))
    }

    /// Ensure the workspace structure exists
    pub fn ensure_structure(&self) -> Result<(), ConstraintError> {
        std::fs::create_dir_all(&self.root).map_err(ConstraintError::Io)?;

        std::fs::create_dir_all(self.constraints_dir()).map_err(ConstraintError::Io)?;

        Ok(())
    }

    /// List all available categories
    #[allow(unused)]
    pub fn list_categories(&self) -> Result<Vec<String>, ConstraintError> {
        let constraints_dir = self.constraints_dir();

        if !constraints_dir.exists() {
            return Ok(vec![]);
        }

        let mut categories = vec![];

        for entry in std::fs::read_dir(&constraints_dir).map_err(ConstraintError::Io)? {
            let entry = entry.map_err(ConstraintError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    categories.push(name.to_string());
                }
            }
        }

        categories.sort();
        Ok(categories)
    }

    /// Check if workspace is properly initialized
    #[allow(unused)]
    pub fn is_initialized(&self) -> bool {
        self.root.exists()
            && self.root.is_dir()
            && self.constraints_dir().exists()
            && self.constraints_dir().is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let newton_dir = temp_dir.path().join(".newton");
        std::fs::create_dir(&newton_dir).unwrap();
        std::fs::create_dir(newton_dir.join("constraints")).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Should find workspace
        let workspace = Workspace::discover().unwrap();
        assert!(workspace.is_initialized());
        assert_eq!(workspace.constraints_dir(), newton_dir.join("constraints"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_workspace_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Change to temp directory without .newton
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Should fail to find workspace
        let result = Workspace::discover();
        assert!(matches!(result, Err(ConstraintError::WorkspaceNotFound(_))));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_constraint_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace {
            root: temp_dir.path().join(".newton"),
        };

        let file_path = workspace.constraint_file("security", "nt-a1b2c3").unwrap();
        let expected = workspace.category_dir("security").join("nt-a1b2c3.jsonl");
        assert_eq!(file_path, expected);
    }

    #[test]
    fn test_invalid_id() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace {
            root: temp_dir.path().join(".newton"),
        };

        let result = workspace.constraint_file("security", "invalid-id");
        assert!(matches!(result, Err(ConstraintError::InvalidIdFormat(_))));
    }
}
