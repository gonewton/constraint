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
fn test_list_empty_constraints() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No constraints found."));
}

#[test]
fn test_list_all_constraints() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add some test constraints
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("All passwords must be hashed")
        .arg("--author").arg("test-author");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Unit tests are recommended")
        .arg("--author").arg("test-author");
    add_cmd2.assert().success();

    // List all constraints
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 2 constraint(s):"))
        .stdout(predicate::str::contains("security"))
        .stdout(predicate::str::contains("testing"));
}

#[test]
fn test_list_category_filter() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraints in different categories
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Security constraint")
        .arg("--author").arg("test-author");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Testing constraint")
        .arg("--author").arg("test-author");
    add_cmd2.assert().success();

    // List only security constraints
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("list")
        .arg("--category").arg("security");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s):"))
        .stdout(predicate::str::contains("Security constraint"))
        .stdout(predicate::str::contains("security"))
        .stdout(predicate::str::is_not(predicate::str::contains("Testing constraint")));
}

#[test]
fn test_list_json_output() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Test constraint")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // List in JSON format
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("list")
        .arg("--format").arg("json");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse as JSON array
    let constraints: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(constraints.len(), 1);

    let constraint = &constraints[0];
    assert_eq!(constraint["type"], "MUST");
    assert_eq!(constraint["category"], "security");
    assert_eq!(constraint["text"], "Test constraint");
    assert_eq!(constraint["author"], "test-author");
    assert_eq!(constraint["version"], 1);
    assert!(constraint["id"].as_str().unwrap().starts_with("nt-"));
}

#[test]
fn test_list_help_output() {
    let mut cmd = run_constraint_command(&["list", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("list_help_output", &stdout, &cli_snapshot_settings());
}