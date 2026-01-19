use constraint::core::loader::{ConstraintLoader, LoaderRegistry};
use constraint::core::constraint::{Constraint, ConstraintType};

#[test]
fn test_loader_registry_creation() {
    let registry = LoaderRegistry::new();
    assert_eq!(registry.current_version(), 1);
    assert!(registry.supports_version(1));
    assert!(!registry.supports_version(2));
}

#[test]
fn test_v1_loader_functionality() {
    let registry = LoaderRegistry::new();

    // Create a V1 constraint
    let constraint = Constraint::new(
        ConstraintType::Must,
        "test".to_string(),
        "Test constraint".to_string(),
        "test-author".to_string(),
        Some("nt-test01".to_string()),
        vec![],
        None,
        "".to_string(),
        None,
    ).unwrap();

    // Serialize to test loading
    let data = serde_json::to_vec(&constraint).unwrap();

    // Test version detection
    let version = registry.detect_version(&data).unwrap();
    assert_eq!(version, 1);

    // Test loading
    let loaded = registry.load_constraint(&data).unwrap();
    assert_eq!(loaded.id, constraint.id);
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.text, constraint.text);
}

#[test]
fn test_constraint_serialization_roundtrip() {
    let registry = LoaderRegistry::new();

    // Create constraint with all fields
    let original = Constraint::new(crate::core::constraint::ConstraintParams {
        r#type: ConstraintType::Should,
        category: "complex".to_string(),
        text: "Complex constraint with all fields".to_string(),
        author: "complex-author".to_string(),
        id: Some("nt-complex".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        priority: Some("P2".to_string()),
        references: "RFC 2119 reference".to_string(),
        verification: Some("./validate.sh".to_string()),
    }).unwrap();

    // Serialize and load back
    let data = serde_json::to_vec(&original).unwrap();
    let loaded = registry.load_constraint(&data).unwrap();

    // Verify all fields match
    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.r#type, original.r#type);
    assert_eq!(loaded.category, original.category);
    assert_eq!(loaded.text, original.text);
    assert_eq!(loaded.author, original.author);
    assert_eq!(loaded.tags, original.tags);
    assert_eq!(loaded.priority, original.priority);
    assert_eq!(loaded.references, original.references);
    assert_eq!(loaded.verification, original.verification);
    assert_eq!(loaded.validation_status, original.validation_status);
}

#[test]
fn test_unknown_version_error() {
    let registry = LoaderRegistry::new();

    // Create data with unknown version
    let data = r#"{
        "version": 999,
        "id": "nt-test01",
        "type": "MUST",
        "category": "test",
        "text": "Test constraint",
        "author": "test-author",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "validation_status": "Valid"
    }"#;

    let result = registry.load_constraint(data.as_bytes());
    assert!(result.is_err());
    assert!(matches!(result, Err(constraint::core::error::ConstraintError::Loader(
        constraint::core::error::LoaderError::UnknownVersion(999)
    ))));
}

#[test]
fn test_invalid_json_handling() {
    let registry = LoaderRegistry::new();

    // Test with invalid JSON
    let invalid_data = b"not json at all";
    let result = registry.load_constraint(invalid_data);
    assert!(result.is_err());

    // Test with incomplete JSON
    let incomplete_data = b"{\"version\": 1, \"incomplete\": ";
    let result = registry.load_constraint(incomplete_data);
    assert!(result.is_err());
}

#[test]
fn test_constraint_validation_on_load() {
    let registry = LoaderRegistry::new();

    // Create constraint with invalid data
    let invalid_data = r#"{
        "version": 1,
        "id": "invalid-id",
        "type": "INVALID_TYPE",
        "category": "test",
        "text": "",
        "author": "",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "validation_status": "Valid"
    }"#;

    let result = registry.load_constraint(invalid_data.as_bytes());
    assert!(result.is_err());
    // Should fail due to validation errors (empty text, empty author, invalid ID format, invalid type)
}

#[test]
fn test_loader_trait_behavior() {
    let registry = LoaderRegistry::new();

    // Get the V1 loader
    let loader = registry.get_loader(1).unwrap();
    assert_eq!(loader.version(), 1);
    assert!(loader.can_load(1));
    assert!(!loader.can_load(2));

    // Test upgrade (V1 should not change)
    let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
        r#type: ConstraintType::Must,
        category: "test".to_string(),
        text: "Test".to_string(),
        author: "author".to_string(),
        id: Some("nt-test01".to_string()),
        tags: vec![],
        priority: None,
        references: "".to_string(),
        verification: None,
    }).unwrap();

    let upgraded = loader.upgrade(constraint.clone()).unwrap();
    assert_eq!(upgraded.id, constraint.id);
    assert_eq!(upgraded.version, constraint.version);
}

#[test]
fn test_registry_loader_list() {
    let registry = LoaderRegistry::new();

    let loaders = registry.loaders();
    assert_eq!(loaders.len(), 1);
    assert_eq!(loaders[0].version(), 1);
}

#[test]
fn test_auto_upgrade_not_needed() {
    let registry = LoaderRegistry::new();

    // Create V1 constraint (current version)
    let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
        r#type: ConstraintType::Must,
        category: "test".to_string(),
        text: "Test constraint".to_string(),
        author: "author".to_string(),
        id: Some("nt-test01".to_string()),
        tags: vec![],
        priority: None,
        references: "".to_string(),
        verification: None,
    }).unwrap();

    let data = serde_json::to_vec(&constraint).unwrap();
    let loaded = registry.load_constraint(&data).unwrap();

    // Should be the same since no upgrade needed
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.id, constraint.id);
}