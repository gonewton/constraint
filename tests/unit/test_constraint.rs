use constraint::core::constraint::{Constraint, ConstraintType, ValidationStatus, ConstraintParams};
use chrono::Utc;

#[test]
fn test_constraint_creation_valid() {
    let constraint = Constraint::new(ConstraintParams {
        r#type: ConstraintType::Must,
        category: "security".to_string(),
        text: "All passwords must be hashed".to_string(),
        author: "test-author".to_string(),
        id: None,
        tags: vec!["auth".to_string()],
        priority: Some("P1".to_string()),
        references: "RFC 2119".to_string(),
        verification: Some("cargo test --test auth".to_string()),
    }).unwrap();

    assert_eq!(constraint.version, 1);
    assert!(constraint.id.starts_with("nt-"));
    assert_eq!(constraint.r#type, ConstraintType::Must);
    assert_eq!(constraint.category, "security");
    assert_eq!(constraint.text, "All passwords must be hashed");
    assert_eq!(constraint.author, "test-author");
    assert_eq!(constraint.tags, vec!["auth".to_string()]);
    assert_eq!(constraint.priority, Some("P1".to_string()));
    assert_eq!(constraint.references, "RFC 2119".to_string());
    assert_eq!(constraint.verification, Some("cargo test --test auth".to_string()));
    assert!(matches!(constraint.validation_status, ValidationStatus::Valid));
    assert!(constraint.created_at <= constraint.updated_at);
}

#[test]
fn test_constraint_validation_rules() {
    // Valid constraint
    let result = Constraint::new(
        ConstraintType::Should,
        "testing".to_string(),
        "Unit tests are recommended".to_string(),
        "developer".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    );
    assert!(result.is_ok());

    // Invalid category (uppercase)
    let result = Constraint::new(
        ConstraintType::Must,
        "INVALID-CATEGORY".to_string(),
        "Valid text".to_string(),
        "author".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    );
    assert!(result.is_err());

    // Empty text
    let result = Constraint::new(
        ConstraintType::Must,
        "security".to_string(),
        "".to_string(),
        "author".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    );
    assert!(result.is_err());

    // Empty author
    let result = Constraint::new(
        ConstraintType::Must,
        "security".to_string(),
        "Valid text".to_string(),
        "".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    );
    assert!(result.is_err());

    // Invalid priority
    let result = Constraint::new(
        ConstraintType::Must,
        "security".to_string(),
        "Valid text".to_string(),
        "author".to_string(),
        None,
        vec![],
        Some("P4".to_string()),
        "".to_string(),
        None,
    );
    assert!(result.is_err());

    // Text too long (>10,000 chars)
    let long_text = "x".repeat(10001);
    let result = Constraint::new(
        ConstraintType::Must,
        "security".to_string(),
        long_text,
        "author".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_constraint_update() {
    let mut constraint = Constraint::new(
        ConstraintType::Should,
        "testing".to_string(),
        "Initial requirement".to_string(),
        "author".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    ).unwrap();

    let original_updated = constraint.updated_at;

    // Update text and verification
    use constraint::core::constraint::ConstraintUpdate;
    let updates = ConstraintUpdate {
        text: Some("Updated requirement".to_string()),
        verification: Some(Some("cargo test".to_string())),
        ..Default::default()
    };

    constraint.update(updates).unwrap();

    assert_eq!(constraint.text, "Updated requirement");
    assert_eq!(constraint.verification, Some("cargo test".to_string()));
    assert!(constraint.updated_at > original_updated);
}

#[test]
fn test_id_generation_deterministic() {
    let id1 = Constraint::generate_id("test text", "category", &ConstraintType::Must);
    let id2 = Constraint::generate_id("test text", "category", &ConstraintType::Must);

    // Same inputs should generate same ID
    assert_eq!(id1, id2);
    assert!(id1.starts_with("nt-"));
    assert_eq!(id1.len(), 9); // nt- + 6 chars
}

#[test]
fn test_serialization() {
    let constraint = Constraint::new(
        ConstraintType::May,
        "optional".to_string(),
        "Optional feature".to_string(),
        "author".to_string(),
        None,
        vec![],
        None,
        "".to_string(),
        None,
    ).unwrap();

    // Test JSON serialization
    let json = serde_json::to_string(&constraint).unwrap();
    let deserialized: Constraint = serde_json::from_str(&json).unwrap();

    assert_eq!(constraint.id, deserialized.id);
    assert_eq!(constraint.text, deserialized.text);
    assert_eq!(constraint.category, deserialized.category);
    assert_eq!(constraint.r#type, deserialized.r#type);
}

#[test]
fn test_validation_status() {
    let constraint = Constraint::new(
        ConstraintType::Forbidden,
        "security".to_string(),
        "No hardcoded secrets".to_string(),
        "security-team".to_string(),
        None,
        vec!["secrets".to_string()],
        Some("P1".to_string()),
        "Security guidelines".to_string(),
        Some("./scripts/check-secrets.sh".to_string()),
    ).unwrap();

    // Should be valid by default
    assert!(matches!(constraint.validation_status, ValidationStatus::Valid));
}