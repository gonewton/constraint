//! Constraint data model and validation logic

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::error::ConstraintError;

/// RFC 2119 constraint types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConstraintType {
    Must,      // Absolute requirement
    Shall,     // Strong requirement
    #[default]
    Should,    // Recommended but not mandatory
    May,       // Optional/permitted
    Forbidden, // Explicitly prohibited
}

/// Current validation state of a constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Valid,   // Constraint is properly formed and verifiable
    Invalid, // Constraint has validation errors
    Warning, // Constraint is valid but has warnings
}

/// Primary constraint entity representing a single requirement or rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Integer version for format evolution (required, >= 1)
    pub version: u32,

    /// Unique hash-based identifier (format: nt-xxxxxx, required)
    pub id: String,

    /// RFC 2119 constraint type (required)
    #[serde(rename = "type")]
    pub r#type: ConstraintType,

    /// Organizational category (lowercase, no spaces, required)
    pub category: String,

    /// Constraint description text (required, non-empty)
    pub text: String,

    /// Optional search tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,

    /// Priority level (P1/P2/P3, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,

    /// Author identifier (required)
    pub author: String,

    /// Free text references
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub references: String,

    /// Verification command/script/description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,

    /// Creation timestamp (required)
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,

    /// Last update timestamp (required)
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,

    /// Current validation state (required)
    pub validation_status: ValidationStatus,
}

impl Constraint {
    /// Create a new constraint with validation
    pub fn new(params: ConstraintParams) -> Result<Self, ConstraintError> {
        let id = params.id.unwrap_or_else(|| Self::generate_id(&params.text, &params.category, &params.r#type));

        let constraint = Self {
            version: 1,
            id,
            r#type: params.r#type,
            category: params.category,
            text: params.text,
            tags: params.tags,
            priority: params.priority,
            author: params.author,
            references: params.references,
            verification: params.verification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            validation_status: ValidationStatus::Valid,
        };

        constraint.validate()?;
        Ok(constraint)
    }

    /// Validate constraint data integrity
    pub fn validate(&self) -> Result<(), ConstraintError> {
        // Validate version
        if self.version < 1 {
            return Err(ConstraintError::Validation(
                "Version must be >= 1".to_string(),
            ));
        }

        // Validate ID format
        let id_regex = Regex::new(r"^nt-[0-9a-z]{6}$")
            .map_err(|_| ConstraintError::Validation("Invalid regex pattern".to_string()))?;

        if !id_regex.is_match(&self.id) {
            return Err(ConstraintError::InvalidIdFormat(self.id.clone()));
        }

        // Validate category format
        let category_regex = Regex::new(r"^[a-z0-9-]+$")
            .map_err(|_| ConstraintError::Validation("Invalid category regex".to_string()))?;

        if !category_regex.is_match(&self.category) {
            return Err(ConstraintError::Validation(
                "Category must be lowercase alphanumeric with hyphens only".to_string(),
            ));
        }

        // Validate text
        if self.text.trim().is_empty() {
            return Err(ConstraintError::Validation(
                "Text cannot be empty".to_string(),
            ));
        }

        if self.text.len() > 10000 {
            return Err(ConstraintError::Validation(
                "Text cannot exceed 10,000 characters".to_string(),
            ));
        }

        // Validate author
        if self.author.trim().is_empty() {
            return Err(ConstraintError::Validation(
                "Author cannot be empty".to_string(),
            ));
        }

        // Validate timestamps
        if self.created_at > self.updated_at {
            return Err(ConstraintError::Validation(
                "Created timestamp cannot be after updated timestamp".to_string(),
            ));
        }

        // Validate priority if present
        if let Some(priority) = &self.priority {
            if !["P1", "P2", "P3"].contains(&priority.as_str()) {
                return Err(ConstraintError::Validation(
                    "Priority must be P1, P2, or P3".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Generate a deterministic ID from constraint content
    pub fn generate_id(text: &str, category: &str, r#type: &ConstraintType) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        category.hash(&mut hasher);
        format!("{:?}", r#type).hash(&mut hasher);

        let hash = hasher.finish();
        let hash_str = format!("{:x}", hash);

        // Take first 6 characters of hash and ensure they're valid base36
        let suffix: String = hash_str
            .chars()
            .take(6)
            .map(|c| match c {
                '0'..='9' => c,
                'a'..='f' => (b'a' + (c as u8 - b'a')) as char,
                _ => '0',
            })
            .collect();

        format!("nt-{}", suffix)
    }

    /// Update the constraint with new data
    pub fn update(&mut self, updates: ConstraintUpdate) -> Result<(), ConstraintError> {
        if let Some(text) = updates.text {
            self.text = text;
        }

        if let Some(tags) = updates.tags {
            self.tags = tags;
        }

        if let Some(priority) = updates.priority {
            self.priority = priority;
        }

        if let Some(references) = updates.references {
            self.references = references;
        }

        if let Some(verification) = updates.verification {
            self.verification = verification;
        }

        self.updated_at = Utc::now();
        self.validate()?;
        Ok(())
    }
}

/// Parameters for creating a new constraint
#[derive(Debug, Default)]
pub struct ConstraintParams {
    pub r#type: ConstraintType,
    pub category: String,
    pub text: String,
    pub author: String,
    pub id: Option<String>,
    pub tags: Vec<String>,
    pub priority: Option<String>,
    pub references: String,
    pub verification: Option<String>,
}

/// Structure for constraint updates
#[derive(Debug, Default)]
pub struct ConstraintUpdate {
    pub text: Option<String>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<Option<String>>,
    pub references: Option<String>,
    pub verification: Option<Option<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_creation() {
        let constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        assert_eq!(constraint.version, 1);
        assert!(constraint.id.starts_with("nt-"));
        assert_eq!(constraint.category, "security");
        assert_eq!(constraint.text, "All passwords must be hashed");
        assert_eq!(constraint.author, "test-author");
        assert!(matches!(
            constraint.validation_status,
            ValidationStatus::Valid
        ));
    }

    #[test]
    fn test_id_generation() {
        let id1 = Constraint::generate_id("test text", "category", &ConstraintType::Must);
        let id2 = Constraint::generate_id("test text", "category", &ConstraintType::Must);

        // Same inputs should generate same ID
        assert_eq!(id1, id2);
        assert!(id1.starts_with("nt-"));
        assert_eq!(id1.len(), 9); // nt- + 6 chars
    }

    #[test]
    fn test_validation_rules() {
        // Valid constraint
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Valid text".to_string(),
            author: "author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        });
        assert!(result.is_ok());

        // Invalid category
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "INVALID-CASE".to_string(),
            text: "Valid text".to_string(),
            author: "author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        });
        assert!(matches!(result, Err(ConstraintError::Validation(_))));

        // Empty text
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "".to_string(),
            author: "author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        });
        assert!(matches!(result, Err(ConstraintError::Validation(_))));

        // Empty author
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Valid text".to_string(),
            author: "".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        });
        assert!(matches!(result, Err(ConstraintError::Validation(_))));

        // Invalid priority
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "Valid text".to_string(),
            author: "author".to_string(),
            id: None,
            tags: vec![],
            priority: Some("P4".to_string()),
            references: "".to_string(),
            verification: None,
        });
        assert!(matches!(result, Err(ConstraintError::Validation(_))));

        // Text too long (>10,000 chars)
        let long_text = "x".repeat(10001);
        let result = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: long_text,
            author: "author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        });
        assert!(matches!(result, Err(ConstraintError::Validation(_))));
    }
}
