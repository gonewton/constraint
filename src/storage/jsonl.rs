//! JSONL-based storage implementation for constraints

use fs2::FileExt;
use std::path::{Path, PathBuf};

use crate::core::constraint::Constraint;
use crate::core::error::ConstraintError;
use crate::core::loader::LoaderRegistry;
use crate::utils::id::IdGenerator;

/// JSONL-based storage for constraints
pub struct JsonlStorage {
    constraints_dir: PathBuf,
    loader_registry: LoaderRegistry,
}

impl JsonlStorage {
    /// Create a new JSONL storage instance
    pub fn new(constraints_dir: PathBuf) -> Self {
        Self {
            constraints_dir,
            loader_registry: LoaderRegistry::new(),
        }
    }

    /// Write a constraint to storage
    pub fn write_constraint(&self, constraint: &Constraint) -> Result<(), ConstraintError> {
        let file_path = self.constraint_file_path(&constraint.category, &constraint.id)?;

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create a copy with current version for writing
        let mut constraint_to_write = constraint.clone();
        constraint_to_write.version = self.loader_registry.current_version();

        // Prepare data for writing
        let data = serde_json::to_vec(&constraint_to_write)?;

        // Atomic write with file locking
        self.atomic_write(&file_path, &data)?;

        Ok(())
    }

    /// Read a constraint from storage
    pub fn read_constraint(&self, category: &str, id: &str) -> Result<Constraint, ConstraintError> {
        let file_path = self.constraint_file_path(category, id)?;

        if !file_path.exists() {
            return Err(ConstraintError::NotFound { id: id.to_string() });
        }

        let data = std::fs::read(&file_path)?;
        let constraint: Constraint = serde_json::from_slice(&data)?;

        // Validate ID format
        if !IdGenerator::validate(&constraint.id) {
            return Err(ConstraintError::InvalidIdFormat(constraint.id));
        }

        Ok(constraint)
    }

    /// Read a constraint by ID (searches all categories)
    pub fn read_constraint_by_id(&self, id: &str) -> Result<Constraint, ConstraintError> {
        // Validate ID format first
        if !IdGenerator::validate(id) {
            return Err(ConstraintError::InvalidIdFormat(id.to_string()));
        }

        if !self.constraints_dir.exists() {
            return Err(ConstraintError::NotFound { id: id.to_string() });
        }

        // Search through all categories
        for category_entry in std::fs::read_dir(&self.constraints_dir)? {
            let category_entry = category_entry?;
            let category_path = category_entry.path();

            if category_path.is_dir() {
                if let Some(category_name) = category_path.file_name().and_then(|n| n.to_str()) {
                    // Try to read from this category
                    match self.read_constraint(category_name, id) {
                        Ok(constraint) => return Ok(constraint),
                        Err(ConstraintError::NotFound { .. }) => continue, // Not in this category
                        Err(e) => return Err(e),                           // Other error
                    }
                }
            }
        }

        Err(ConstraintError::NotFound { id: id.to_string() })
    }

    /// Read all constraints from a category
    pub fn read_category_constraints(
        &self,
        category: &str,
    ) -> Result<Vec<Constraint>, ConstraintError> {
        let category_dir = self.constraints_dir.join(category);

        if !category_dir.exists() {
            return Ok(vec![]);
        }

        let mut constraints = vec![];

        for entry in std::fs::read_dir(&category_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                match self.read_constraint_file(&path) {
                    Ok(constraint) => constraints.push(constraint),
                    Err(e) => {
                        // Log error but continue with other files
                        eprintln!(
                            "Warning: Failed to load constraint from {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(constraints)
    }

    /// Read all constraints across all categories
    pub fn read_all_constraints(&self) -> Result<Vec<Constraint>, ConstraintError> {
        let mut all_constraints = vec![];

        if !self.constraints_dir.exists() {
            return Ok(all_constraints);
        }

        for category_entry in std::fs::read_dir(&self.constraints_dir)? {
            let category_entry = category_entry?;
            let category_path = category_entry.path();

            if category_path.is_dir() {
                if let Some(category_name) = category_path.file_name().and_then(|n| n.to_str()) {
                    let mut category_constraints = self.read_category_constraints(category_name)?;
                    all_constraints.append(&mut category_constraints);
                }
            }
        }

        Ok(all_constraints)
    }

    /// Search constraints by text content
    pub fn search_constraints(
        &self,
        query: &str,
        category_filter: Option<&str>,
    ) -> Result<Vec<Constraint>, ConstraintError> {
        let constraints = if let Some(category) = category_filter {
            self.read_category_constraints(category)?
        } else {
            self.read_all_constraints()?
        };

        let query_lower = query.to_lowercase();
        let results: Vec<Constraint> = constraints
            .into_iter()
            .filter(|c| {
                c.text.to_lowercase().contains(&query_lower)
                    || c.tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
                    || c.category.to_lowercase().contains(&query_lower)
                    || c.references.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(results)
    }

    /// Delete a constraint
    pub fn delete_constraint(&self, category: &str, id: &str) -> Result<(), ConstraintError> {
        let file_path = self.constraint_file_path(category, id)?;

        if !file_path.exists() {
            return Err(ConstraintError::NotFound { id: id.to_string() });
        }

        std::fs::remove_file(&file_path)?;
        Ok(())
    }

    /// Get the file path for a constraint
    fn constraint_file_path(&self, category: &str, id: &str) -> Result<PathBuf, ConstraintError> {
        // Validate ID format
        if !IdGenerator::validate(id) {
            return Err(ConstraintError::InvalidIdFormat(id.to_string()));
        }

        Ok(self
            .constraints_dir
            .join(category)
            .join(format!("{}.jsonl", id)))
    }

    /// Read a constraint from a specific file path
    fn read_constraint_file(&self, file_path: &Path) -> Result<Constraint, ConstraintError> {
        let data = std::fs::read(file_path)?;
        let constraint = self.loader_registry.load_constraint(&data)?;
        Ok(constraint)
    }

    /// Perform atomic write with file locking
    fn atomic_write(&self, file_path: &Path, data: &[u8]) -> Result<(), ConstraintError> {
        // Create temporary file
        let temp_path = file_path.with_extension("tmp");

        // Write to temporary file
        std::fs::write(&temp_path, data)?;

        // Lock and move atomically
        let file = std::fs::File::open(&temp_path)?;
        file.lock_exclusive()?;

        // Atomic move
        std::fs::rename(&temp_path, file_path)?;

        // Unlock (automatically released when file is dropped)
        drop(file);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constraint::ConstraintType;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_constraint() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonlStorage::new(temp_dir.path().to_path_buf());

        // Create test constraint
        let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Test constraint".to_string(),
            author: "test-author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // Write constraint
        storage.write_constraint(&constraint).unwrap();

        // Read constraint back
        let read_constraint = storage.read_constraint("security", "nt-test01").unwrap();

        assert_eq!(constraint.id, read_constraint.id);
        assert_eq!(constraint.text, read_constraint.text);
        assert_eq!(constraint.category, read_constraint.category);
    }

    #[test]
    fn test_read_nonexistent_constraint() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonlStorage::new(temp_dir.path().to_path_buf());

        let result = storage.read_constraint("security", "nt-123456");
        assert!(matches!(result, Err(ConstraintError::NotFound { .. })));
    }

    #[test]
    fn test_search_constraints() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonlStorage::new(temp_dir.path().to_path_buf());

        // Create test constraints
        let constraint1 = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Password must be hashed".to_string(),
            author: "author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec!["auth".to_string()],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        let constraint2 = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Should,
            category: "testing".to_string(),
            text: "Unit tests recommended".to_string(),
            author: "author".to_string(),
            id: Some("nt-test02".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        storage.write_constraint(&constraint1).unwrap();
        storage.write_constraint(&constraint2).unwrap();

        // Search for "password"
        let results = storage.search_constraints("password", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "nt-test01");

        // Search for "test" (should find constraint2 due to "testing" category and "tests" in text)
        let results = storage.search_constraints("test", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "nt-test02");
    }

    #[test]
    fn test_delete_constraint() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonlStorage::new(temp_dir.path().to_path_buf());

        // Create and write constraint
        let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Test constraint".to_string(),
            author: "author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        storage.write_constraint(&constraint).unwrap();

        // Verify it exists
        let read_constraint = storage.read_constraint("security", "nt-test01").unwrap();
        assert_eq!(read_constraint.id, "nt-test01");

        // Delete constraint
        storage.delete_constraint("security", "nt-test01").unwrap();

        // Verify it's gone
        let result = storage.read_constraint("security", "nt-test01");
        assert!(matches!(result, Err(ConstraintError::NotFound { .. })));
    }
}
