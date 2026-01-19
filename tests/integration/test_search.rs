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
fn test_search_no_results() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("All passwords must be hashed")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Search for non-matching term
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("search")
        .arg("nonexistent");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No constraints found matching query 'nonexistent'"));
}

#[test]
fn test_search_text_content() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add test constraints
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

    // Search for "password"
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("search")
        .arg("password");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s) matching query 'password'"))
        .stdout(predicate::str::contains("All passwords must be hashed"))
        .stdout(predicate::str::is_not(predicate::str::contains("Unit tests are recommended")));
}

#[test]
fn test_search_category_filter() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraints in different categories with similar content
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Code must be reviewed")
        .arg("--author").arg("test-author");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Code must be reviewed")
        .arg("--author").arg("test-author");
    add_cmd2.assert().success();

    // Search for "reviewed" in security category only
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("search")
        .arg("reviewed")
        .arg("--category").arg("security");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s) matching query 'reviewed'"))
        .stdout(predicate::str::contains("security"))
        .stdout(predicate::str::is_not(predicate::str::contains("testing")));
}

#[test]
fn test_search_multiple_matches() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraints with similar content
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("API must validate input")
        .arg("--author").arg("test-author");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("performance")
        .arg("--text").arg("API must respond quickly")
        .arg("--author").arg("test-author");
    add_cmd2.assert().success();

    // Search for "API" (should match both)
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("search")
        .arg("API");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 2 constraint(s) matching query 'API'"))
        .stdout(predicate::str::contains("validate input"))
        .stdout(predicate::str::contains("respond quickly"));
}

#[test]
fn test_search_json_output() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Test security constraint")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Search in JSON format
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("search")
        .arg("security")
        .arg("--format").arg("json");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse as JSON array
    let constraints: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(constraints.len(), 1);

    let constraint = &constraints[0];
    assert_eq!(constraint["type"], "MUST");
    assert_eq!(constraint["category"], "security");
    assert_eq!(constraint["text"], "Test security constraint");
    assert_eq!(constraint["author"], "test-author");
}

#[test]
fn test_search_help_output() {
    let mut cmd = run_constraint_command(&["search", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("search_help_output", &stdout, &cli_snapshot_settings());
}