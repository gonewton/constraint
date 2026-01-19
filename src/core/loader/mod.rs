//! Versioning system for constraint format evolution

use crate::core::constraint::Constraint;
use crate::core::error::LoaderError;

/// Trait for loading and upgrading constraint formats
pub trait ConstraintLoader: Send + Sync {
    /// Get the version this loader handles
    #[allow(unused)]
    fn version(&self) -> u32;

    /// Check if this loader can handle the given version
    fn can_load(&self, version: u32) -> bool;

    /// Load a constraint from raw data
    fn load(&self, data: &[u8]) -> Result<Constraint, LoaderError>;

    /// Upgrade a constraint to a newer format
    fn upgrade(&self, constraint: Constraint) -> Result<Constraint, LoaderError>;
}

/// Registry for managing multiple constraint loaders
pub struct LoaderRegistry {
    loaders: Vec<Box<dyn ConstraintLoader>>,
    current_version: u32,
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderRegistry {
    /// Create a new loader registry with all available loaders
    pub fn new() -> Self {
        let loaders: Vec<Box<dyn ConstraintLoader>> = vec![Box::new(v1::V1Loader)];

        Self {
            current_version: 1,
            loaders,
        }
    }

    /// Detect the version of a constraint from its data
    pub fn detect_version(&self, data: &[u8]) -> Result<u32, LoaderError> {
        #[derive(serde::Deserialize)]
        struct VersionDetector {
            version: u32,
        }

        let detector: VersionDetector = serde_json::from_slice(data)?;
        Ok(detector.version)
    }

    /// Load a constraint using the appropriate loader for its version
    pub fn load_constraint(&self, data: &[u8]) -> Result<Constraint, LoaderError> {
        let version = self.detect_version(data)?;

        // Find loader for this version
        let loader = self
            .loaders
            .iter()
            .find(|l| l.can_load(version))
            .ok_or(LoaderError::UnknownVersion(version))?;

        // Load the constraint
        let mut constraint = loader.load(data)?;

        // Auto-upgrade to current version if needed
        if constraint.version < self.current_version {
            constraint = self.upgrade_to_current(constraint)?;
        }

        Ok(constraint)
    }

    /// Upgrade a constraint to the current version
    pub fn upgrade_to_current(
        &self,
        mut constraint: Constraint,
    ) -> Result<Constraint, LoaderError> {
        while constraint.version < self.current_version {
            let loader = self
                .loaders
                .iter()
                .find(|l| l.can_load(constraint.version))
                .ok_or(LoaderError::UnknownVersion(constraint.version))?;

            constraint = loader.upgrade(constraint)?;
        }
        Ok(constraint)
    }

    /// Get the current version
    pub fn current_version(&self) -> u32 {
        self.current_version
    }
}

pub mod v1;
