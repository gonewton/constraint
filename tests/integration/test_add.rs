use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use super::snapshot_helpers::{run_constraint_command, cli_snapshot_settings};
use insta::assert_snapshot_with_settings;

fn constraint_command() -> Command {
    let mut cmd = Command::cargo_bin("constraint").unwrap();
    cmd.current_dir(std::env::current_dir().unwrap());
    cmd
}

#[test]
fn test_add_constraint_basic() {
    let temp_dir = TempDir::new().unwrap();

    // Change to temp directory
    std::env::set_current_dir(&temp_dir).unwrap();

    // Create .newton directory structure
    fs::create_dir_all(".newton/constraints").unwrap();

    // Restore original directory
    std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap()).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("All passwords must be hashed")
        .arg("--author").arg("test-author");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint added with ID:"));

    // Verify file was created
    let constraint_files: Vec<_> = fs::read_dir(temp_dir.path().join(".newton/constraints/security"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();

    assert_eq!(constraint_files.len(), 1);
    let file_path = &constraint_files[0];
    assert!(file_path.extension().unwrap() == "jsonl");

    // Verify file content
    let content = fs::read_to_string(file_path).unwrap();
    let constraint: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(constraint["type"], "MUST");
    assert_eq!(constraint["category"], "security");
    assert_eq!(constraint["text"], "All passwords must be hashed");
    assert_eq!(constraint["author"], "test-author");
    assert_eq!(constraint["version"], 1);
    assert!(constraint["id"].as_str().unwrap().starts_with("nt-"));
}

#[test]
fn test_add_help_output() {
    let mut cmd = run_constraint_command(&["add", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("add_help_output", &stdout, &cli_snapshot_settings());
}

#[test]
fn test_add_constraint_with_explicit_id() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Unit tests are recommended")
        .arg("--author").arg("developer")
        .arg("--id").arg("nt-custom");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint added: nt-custom"));

    // Verify file was created with correct name
    let file_path = temp_dir.path().join(".newton/constraints/testing/nt-custom.jsonl");
    assert!(file_path.exists());

    // Verify content
    let content = fs::read_to_string(&file_path).unwrap();
    let constraint: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(constraint["id"], "nt-custom");
    assert_eq!(constraint["type"], "SHOULD");
    assert_eq!(constraint["category"], "testing");
}

#[test]
fn test_add_constraint_with_tags_and_verification() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("performance")
        .arg("--text").arg("Response time must be under 200ms")
        .arg("--author").arg("architect")
        .arg("--tags").arg("api,performance")
        .arg("--verification").arg("./scripts/benchmark-api.sh");

    cmd.assert().success();

    // Find the created file
    let constraint_files: Vec<_> = fs::read_dir(temp_dir.path().join(".newton/constraints/performance"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();

    assert_eq!(constraint_files.len(), 1);
    let content = fs::read_to_string(&constraint_files[0]).unwrap();
    let constraint: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(constraint["tags"], serde_json::json!(["api", "performance"]));
    assert_eq!(constraint["verification"], "./scripts/benchmark-api.sh");
}

#[test]
fn test_add_constraint_invalid_type() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("INVALID")
        .arg("--category").arg("security")
        .arg("--text").arg("Test constraint")
        .arg("--author").arg("test");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid constraint type"));
}

#[test]
fn test_add_constraint_invalid_category() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("INVALID-CATEGORY")
        .arg("--text").arg("Test constraint")
        .arg("--author").arg("test");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Category must be lowercase"));
}

#[test]
fn test_add_constraint_workspace_creation() {
    let temp_dir = TempDir::new().unwrap();

    // Don't create .newton directory - test auto-creation
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Test auto-creation")
        .arg("--author").arg("test");

    cmd.assert().success();

    // Verify .newton structure was created
    assert!(temp_dir.path().join(".newton").exists());
    assert!(temp_dir.path().join(".newton/constraints").exists());
    assert!(temp_dir.path().join(".newton/constraints/security").exists());
}