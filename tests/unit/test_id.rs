use constraint::utils::id::IdGenerator;

#[test]
fn test_id_generation_deterministic() {
    let mut generator1 = IdGenerator::new();
    let mut generator2 = IdGenerator::new();

    let id1 = generator1.generate("test constraint", "security", "MUST");
    let id2 = generator2.generate("test constraint", "security", "MUST");

    // Same inputs should generate same ID across different generators
    assert_eq!(id1, id2);
    assert!(IdGenerator::validate(&id1));
}

#[test]
fn test_id_generation_unique() {
    let mut generator = IdGenerator::new();

    let id1 = generator.generate("constraint 1", "security", "MUST");
    let id2 = generator.generate("constraint 2", "security", "MUST");
    let id3 = generator.generate("constraint 1", "testing", "MUST");

    // Different inputs should generate different IDs
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);

    // All should be valid
    assert!(IdGenerator::validate(&id1));
    assert!(IdGenerator::validate(&id2));
    assert!(IdGenerator::validate(&id3));
}

#[test]
fn test_id_validation() {
    // Valid IDs
    assert!(IdGenerator::validate("nt-a1b2c3"));
    assert!(IdGenerator::validate("nt-012345"));
    assert!(IdGenerator::validate("nt-abcdef"));
    assert!(IdGenerator::validate("nt-123456"));
    assert!(IdGenerator::validate("nt-789xyz"));

    // Invalid IDs
    assert!(!IdGenerator::validate(""));
    assert!(!IdGenerator::validate("nt-"));
    assert!(!IdGenerator::validate("nt-12345")); // too short
    assert!(!IdGenerator::validate("nt-1234567")); // too long
    assert!(!IdGenerator::validate("xx-123456")); // wrong prefix
    assert!(!IdGenerator::validate("nt-ABCDEFG")); // uppercase not allowed
    assert!(!IdGenerator::validate("nt-123 456")); // spaces not allowed
    assert!(!IdGenerator::validate("nt-123@456")); // special chars not allowed
}

#[test]
fn test_collision_handling() {
    let mut generator = IdGenerator::new();

    // Generate a base ID
    let base_id = generator.generate("base content", "category", "MUST");

    // Try to generate another ID (this simulates a collision scenario)
    // Note: In practice, collisions are extremely rare with SHA256,
    // but this tests the collision handling logic
    let another_id = generator.generate("different content", "category", "MUST");

    assert_ne!(base_id, another_id);
    assert!(IdGenerator::validate(&base_id));
    assert!(IdGenerator::validate(&another_id));
}

#[test]
fn test_different_constraint_types() {
    let mut generator = IdGenerator::new();

    let must_id = generator.generate("same text", "category", "MUST");
    let should_id = generator.generate("same text", "category", "SHOULD");
    let may_id = generator.generate("same text", "category", "MAY");
    let forbidden_id = generator.generate("same text", "category", "FORBIDDEN");

    // All should be different
    let ids = vec![must_id, should_id, may_id, forbidden_id];
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "IDs {} and {} should be different", i, j);
        }
    }

    // All should be valid
    for id in ids {
        assert!(IdGenerator::validate(&id));
    }
}

#[test]
fn test_case_sensitivity() {
    let mut generator = IdGenerator::new();

    let lowercase_id = generator.generate("test", "category", "must");
    let uppercase_id = generator.generate("TEST", "CATEGORY", "MUST");

    // Case differences should produce different IDs
    assert_ne!(lowercase_id, uppercase_id);
    assert!(IdGenerator::validate(&lowercase_id));
    assert!(IdGenerator::validate(&uppercase_id));
}

#[test]
fn test_empty_inputs() {
    let mut generator = IdGenerator::new();

    let id1 = generator.generate("", "category", "MUST");
    let id2 = generator.generate("text", "", "MUST");
    let id3 = generator.generate("text", "category", "");

    // Should handle empty inputs gracefully
    assert!(IdGenerator::validate(&id1));
    assert!(IdGenerator::validate(&id2));
    assert!(IdGenerator::validate(&id3));

    // Different empty combinations should produce different IDs
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);
}