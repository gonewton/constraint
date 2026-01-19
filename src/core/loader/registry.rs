//! Loader registry implementation

use crate::core::constraint::Constraint;
use crate::core::error::LoaderError;
use crate::core::loader::{ConstraintLoader, LoaderRegistry};

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderRegistry {
    /// Add a new loader to the registry
    pub fn add_loader(&mut self, loader: Box<dyn ConstraintLoader>) {
        self.loaders.push(loader);
        // Update current version if this loader handles a newer version
        let loader_version = self.loaders.last().unwrap().version();
        if loader_version > self.current_version {
            self.current_version = loader_version;
        }
    }

    /// Get all registered loaders
    pub fn loaders(&self) -> &[Box<dyn ConstraintLoader>] {
        &self.loaders
    }

    /// Check if a version is supported
    pub fn supports_version(&self, version: u32) -> bool {
        self.loaders.iter().any(|l| l.can_load(version))
    }

    /// Get loader for a specific version
    pub fn get_loader(&self, version: u32) -> Option<&dyn ConstraintLoader> {
        self.loaders
            .iter()
            .find(|l| l.can_load(version))
            .map(|l| l.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::loader::v1::V1Loader;

    #[test]
    fn test_registry_creation() {
        let registry = LoaderRegistry::new();
        assert_eq!(registry.current_version(), 1);
        assert!(registry.supports_version(1));
        assert!(!registry.supports_version(2));
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = LoaderRegistry::new();

        // Should have V1 loader
        assert_eq!(registry.loaders().len(), 1);
        assert!(registry.get_loader(1).is_some());

        // Test constraint loading
        let constraint = crate::core::constraint::Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: crate::core::constraint::ConstraintType::Must,
            category: "test".to_string(),
            text: "Test constraint".to_string(),
            author: "test-author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        }).unwrap();

        let data = serde_json::to_vec(&constraint).unwrap();
        let loaded = registry.load_constraint(&data).unwrap();

        assert_eq!(loaded.id, constraint.id);
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn test_unknown_version() {
        let registry = LoaderRegistry::new();

        // Create data with unknown version
        let data = r#"{"version": 999, "id": "nt-test01", "type": "MUST", "category": "test", "text": "test", "author": "test", "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "validation_status": "Valid"}"#;

        let result = registry.load_constraint(data.as_bytes());
        assert!(matches!(result, Err(LoaderError::UnknownVersion(999))));
    }

    #[test]
    fn test_version_detection() {
        let registry = LoaderRegistry::new();

        let data = r#"{"version": 1, "id": "nt-test01", "type": "MUST", "category": "test", "text": "test", "author": "test", "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "validation_status": "Valid"}"#;

        let version = registry.detect_version(data.as_bytes()).unwrap();
        assert_eq!(version, 1);
    }
}