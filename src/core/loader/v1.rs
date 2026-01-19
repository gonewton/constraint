//! V1 constraint format loader

use crate::core::constraint::Constraint;
use crate::core::error::LoaderError;
use crate::core::loader::ConstraintLoader;

/// Loader for V1 constraint format (current version)
pub struct V1Loader;

impl ConstraintLoader for V1Loader {
    fn version(&self) -> u32 {
        1
    }

    fn can_load(&self, version: u32) -> bool {
        version == 1
    }

    fn load(&self, data: &[u8]) -> Result<Constraint, LoaderError> {
        let constraint: Constraint = serde_json::from_slice(data)?;

        if constraint.version != 1 {
            return Err(LoaderError::VersionMismatch {
                expected: 1,
                found: constraint.version,
            });
        }

        // Validate the loaded constraint
        constraint
            .validate()
            .map_err(|e| LoaderError::Validation(e.to_string()))?;

        Ok(constraint)
    }

    fn upgrade(&self, constraint: Constraint) -> Result<Constraint, LoaderError> {
        // V1 is the current version, no upgrade needed
        Ok(constraint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constraint::ConstraintType;

    #[test]
    fn test_v1_loader_version() {
        let loader = V1Loader;
        assert_eq!(loader.version(), 1);
        assert!(loader.can_load(1));
        assert!(!loader.can_load(2));
    }

    #[test]
    fn test_v1_loader_load_valid() {
        let loader = V1Loader;

        // Create a valid V1 constraint JSON
        let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "test".to_string(),
            text: "Test constraint".to_string(),
            author: "test-author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        let data = serde_json::to_vec(&constraint).unwrap();
        let loaded = loader.load(&data).unwrap();

        assert_eq!(loaded.id, constraint.id);
        assert_eq!(loaded.text, constraint.text);
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn test_v1_loader_load_wrong_version() {
        let loader = V1Loader;

        // Create constraint with wrong version
        let mut constraint = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "test".to_string(),
            text: "Test constraint".to_string(),
            author: "test-author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // Manually set wrong version (this would normally be prevented by validation)
        constraint.version = 2;

        let data = serde_json::to_vec(&constraint).unwrap();
        let result = loader.load(&data);

        assert!(matches!(
            result,
            Err(LoaderError::VersionMismatch {
                expected: 1,
                found: 2
            })
        ));
    }

    #[test]
    fn test_v1_loader_upgrade() {
        let loader = V1Loader;

        let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
            r#type: ConstraintType::Must,
            category: "test".to_string(),
            text: "Test constraint".to_string(),
            author: "test-author".to_string(),
            id: Some("nt-test01".to_string()),
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // V1 upgrade should return the same constraint
        let upgraded = loader.upgrade(constraint.clone()).unwrap();
        assert_eq!(upgraded.id, constraint.id);
        assert_eq!(upgraded.version, constraint.version);
    }
}
