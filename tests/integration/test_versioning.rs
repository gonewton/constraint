use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn constraint_command() -> Command {
    let mut cmd = Command::cargo_bin("constraint").unwrap();
    cmd.current_dir(std::env::current_dir().unwrap());
    cmd
}

#[test]
fn test_version_auto_upgrade_on_read() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Manually create a V1 constraint file (simulating old format)
    let constraint_data = r#"{
        "version": 1,
        "id": "nt-test01",
        "type": "MUST",
        "category": "test",
        "text": "Test constraint",
        "tags": [],
        "author": "test-author",
        "references": "",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "validation_status": "Valid"
    }"#;

    let file_path = temp_dir.path().join(".newton/constraints/test/nt-test01.jsonl");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, constraint_data).unwrap();

    // Read the constraint (should auto-upgrade)
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s)"))
        .stdout(predicate::str::contains("Test constraint"));

    // Verify the file still exists and is readable
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();

    // Parse and verify version
    let constraint: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(constraint["version"], 1); // Should remain V1 since it's current
    assert_eq!(constraint["id"], "nt-test01");
}

#[test]
fn test_version_write_with_current_version() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a constraint (should write with current version)
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Version test constraint")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Find the created file
    let constraint_files: Vec<_> = fs::read_dir(temp_dir.path().join(".newton/constraints/test"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();

    assert_eq!(constraint_files.len(), 1);
    let content = fs::read_to_string(&constraint_files[0]).unwrap();
    let constraint: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should be written with current version (1)
    assert_eq!(constraint["version"], 1);
    assert_eq!(constraint["type"], "MUST");
    assert_eq!(constraint["category"], "test");
}

#[test]
fn test_backward_compatibility_read() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Create multiple constraints with different creation times to test ordering
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("First security constraint")
        .arg("--author").arg("security-team");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Testing constraint")
        .arg("--author").arg("qa-team");
    add_cmd2.assert().success();

    // List all constraints (tests reading multiple files with versioning)
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 2 constraint(s)"))
        .stdout(predicate::str::contains("First security constraint"))
        .stdout(predicate::str::contains("Testing constraint"));
}

#[test]
fn test_version_preservation_on_update() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Original text")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Update the constraint
    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg(id)
        .arg("--text").arg("Updated text");
    patch_cmd.assert().success();

    // Verify the constraint was updated and version preserved
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Updated text"))
        .stdout(predicate::str::is_not(predicate::str::contains("Original text")));
}

#[test]
fn test_corrupted_constraint_file_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Create a corrupted constraint file
    let file_path = temp_dir.path().join(".newton/constraints/test/nt-corrupt.jsonl");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "invalid json content").unwrap();

    // Also create a valid constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Valid constraint")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // List should handle the corrupted file gracefully
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    // Should still show the valid constraint despite the corrupted file
    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s)"))
        .stdout(predicate::str::contains("Valid constraint"));
}