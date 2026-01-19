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
fn test_patch_constraint_text() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Original requirement")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID from the output
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Update the constraint text
    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg(id)
        .arg("--text").arg("Updated requirement text");

    patch_cmd.assert()
        .success()
        .stdout(predicate::str::contains("updated successfully"));

    // Verify the change
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Updated requirement text"))
        .stdout(predicate::str::is_not(predicate::str::contains("Original requirement")));
}

#[test]
fn test_patch_constraint_tags() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Test constraint")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Update tags
    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg(id)
        .arg("--tags").arg("unit,integration");

    patch_cmd.assert().success();

    // Verify the change
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("unit, integration"));
}

#[test]
fn test_patch_constraint_priority() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MAY")
        .arg("--category").arg("optional")
        .arg("--text").arg("Optional feature")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Update priority
    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg(id)
        .arg("--priority").arg("P1");

    patch_cmd.assert().success();

    // Verify the change
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Priority: P1"));
}

#[test]
fn test_patch_constraint_verification() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Security check required")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Update verification
    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg(id)
        .arg("--verification").arg("./scripts/check-security.sh");

    patch_cmd.assert().success();

    // Verify the change
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");

    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Verification: ./scripts/check-security.sh"));
}

#[test]
fn test_patch_nonexistent_constraint() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg("nt-nonexistent")
        .arg("--text").arg("Updated text");

    patch_cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Constraint not found"));
}

#[test]
fn test_patch_invalid_id_format() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut patch_cmd = constraint_command();
    patch_cmd.current_dir(&temp_dir)
        .arg("patch")
        .arg("invalid-id")
        .arg("--text").arg("Updated text");

    patch_cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid constraint ID format"));
}

#[test]
fn test_patch_help_output() {
    let mut cmd = run_constraint_command(&["patch", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("patch_help_output", &stdout, &cli_snapshot_settings());
}