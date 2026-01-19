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
fn test_validate_no_constraints() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No constraints found to validate"));
}

#[test]
fn test_validate_without_execute() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraints with and without verification
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Test with verification")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'test passed'");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Test without verification")
        .arg("--author").arg("test");
    add_cmd2.assert().success();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validating 2 constraint(s)"))
        .stdout(predicate::str::contains("SKIPPED"))
        .stdout(predicate::str::contains("Verification command available"))
        .stdout(predicate::str::contains("No verification method"))
        .stdout(predicate::str::contains("Skipped: 2"))
        .stdout(predicate::str::contains("Verification commands were not executed"));
}

#[test]
fn test_validate_with_execute_success() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraint with successful verification command
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Constraint with successful verification")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'verification successful' && exit 0");
    add_cmd.assert().success();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .arg("--execute");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PASSED"))
        .stdout(predicate::str::contains("verification successful"))
        .stdout(predicate::str::contains("Passed: 1"))
        .stdout(predicate::str::contains("All validations completed successfully"));
}

#[test]
fn test_validate_with_execute_failure() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraint with failing verification command
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Constraint with failing verification")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'verification failed' && exit 1");
    add_cmd.assert().success();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .arg("--execute");

    cmd.assert()
        .failure()  // Should exit with code 1
        .stdout(predicate::str::contains("FAILED"))
        .stdout(predicate::str::contains("verification failed"))
        .stdout(predicate::str::contains("Failed: 1"))
        .stdout(predicate::str::contains("Some validations failed"));
}

#[test]
fn test_validate_category_filter() {
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
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'security ok'");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Testing constraint")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'testing ok'");
    add_cmd2.assert().success();

    // Validate only security category
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .arg("--category").arg("security");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validating 1 constraint(s)"))
        .stdout(predicate::str::contains("Security constraint"))
        .stdout(predicate::str::is_not(predicate::str::contains("Testing constraint")));
}

#[test]
fn test_validate_specific_constraint() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add multiple constraints
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("First constraint")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'first'");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Second constraint")
        .arg("--author").arg("test")
        .arg("--verification").arg("echo 'second'");
    add_cmd2.assert().success();

    // Get the ID of the first constraint
    let output = add_cmd1.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Validate only the specific constraint
    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .arg("--id").arg(id);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validating 1 constraint(s)"))
        .stdout(predicate::str::contains("First constraint"))
        .stdout(predicate::str::is_not(predicate::str::contains("Second constraint")));
}

#[test]
fn test_validate_mixed_results() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add constraints with different verification outcomes
    let mut add_cmd1 = constraint_command();
    add_cmd1.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("test")
        .arg("--text").arg("Passing constraint")
        .arg("--author").arg("test")
        .arg("--verification").arg("exit 0");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("test")
        .arg("--text").arg("Failing constraint")
        .arg("--author").arg("test")
        .arg("--verification").arg("exit 1");
    add_cmd2.assert().success();

    let mut add_cmd3 = constraint_command();
    add_cmd3.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MAY")
        .arg("--category").arg("test")
        .arg("--text").arg("No verification")
        .arg("--author").arg("test");
    add_cmd3.assert().success();

    let mut cmd = constraint_command();
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .arg("--execute");

    cmd.assert()
        .failure()  // Should fail due to failing constraint
        .stdout(predicate::str::contains("Validating 3 constraint(s)"))
        .stdout(predicate::str::contains("PASSED"))
        .stdout(predicate::str::contains("FAILED"))
        .stdout(predicate::str::contains("SKIPPED"))
        .stdout(predicate::str::contains("Passed: 1"))
        .stdout(predicate::str::contains("Failed: 1"))
        .stdout(predicate::str::contains("Skipped: 1"));
}

#[test]
fn test_validate_help_output() {
    let mut cmd = run_constraint_command(&["validate", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("validate_help_output", &stdout, &cli_snapshot_settings());
}