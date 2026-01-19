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
fn test_delete_constraint() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("MUST")
        .arg("--category").arg("security")
        .arg("--text").arg("Constraint to delete")
        .arg("--author").arg("test-author");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Verify constraint exists before deletion
    let mut list_cmd1 = constraint_command();
    list_cmd1.current_dir(&temp_dir)
        .arg("list");
    list_cmd1.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s)"))
        .stdout(predicate::str::contains("Constraint to delete"));

    // Delete the constraint
    let mut delete_cmd = constraint_command();
    delete_cmd.current_dir(&temp_dir)
        .arg("delete")
        .arg(id);

    delete_cmd.assert()
        .success()
        .stdout(predicate::str::contains("deleted successfully"));

    // Verify constraint is gone
    let mut list_cmd2 = constraint_command();
    list_cmd2.current_dir(&temp_dir)
        .arg("list");
    list_cmd2.assert()
        .success()
        .stdout(predicate::str::contains("No constraints found."));
}

#[test]
fn test_delete_nonexistent_constraint() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut delete_cmd = constraint_command();
    delete_cmd.current_dir(&temp_dir)
        .arg("delete")
        .arg("nt-nonexistent");

    delete_cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Constraint not found"));
}

#[test]
fn test_delete_invalid_id_format() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    let mut delete_cmd = constraint_command();
    delete_cmd.current_dir(&temp_dir)
        .arg("delete")
        .arg("invalid-id");

    delete_cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid constraint ID format"));
}

#[test]
fn test_delete_constraint_file_removal() {
    let temp_dir = TempDir::new().unwrap();

    // Create .newton directory structure
    fs::create_dir_all(temp_dir.path().join(".newton/constraints")).unwrap();

    // Add a test constraint
    let mut add_cmd = constraint_command();
    add_cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("FORBIDDEN")
        .arg("--category").arg("security")
        .arg("--text").arg("No hardcoded secrets")
        .arg("--author").arg("security-team");
    add_cmd.assert().success();

    // Get the constraint ID
    let output = add_cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id_line = stdout.lines().find(|line| line.contains("Constraint added with ID:")).unwrap();
    let id = id_line.split("ID: ").nth(1).unwrap().trim();

    // Verify file exists
    let file_path = temp_dir.path().join(format!(".newton/constraints/security/{}.jsonl", id));
    assert!(file_path.exists());

    // Delete the constraint
    let mut delete_cmd = constraint_command();
    delete_cmd.current_dir(&temp_dir)
        .arg("delete")
        .arg(id);
    delete_cmd.assert().success();

    // Verify file is removed
    assert!(!file_path.exists());
}

#[test]
fn test_delete_multiple_constraints() {
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
        .arg("--author").arg("test-author");
    add_cmd1.assert().success();

    let mut add_cmd2 = constraint_command();
    add_cmd2.current_dir(&temp_dir)
        .arg("add")
        .arg("--type").arg("SHOULD")
        .arg("--category").arg("testing")
        .arg("--text").arg("Second constraint")
        .arg("--author").arg("test-author");
    add_cmd2.assert().success();

    // Get both IDs
    let output1 = add_cmd1.output().unwrap();
    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let id1 = stdout1.lines().find(|line| line.contains("Constraint added with ID:")).unwrap()
        .split("ID: ").nth(1).unwrap().trim();

    let output2 = add_cmd2.output().unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();
    let id2 = stdout2.lines().find(|line| line.contains("Constraint added with ID:")).unwrap()
        .split("ID: ").nth(1).unwrap().trim();

    // Delete first constraint
    let mut delete_cmd1 = constraint_command();
    delete_cmd1.current_dir(&temp_dir)
        .arg("delete")
        .arg(id1);
    delete_cmd1.assert().success();

    // Verify only second constraint remains
    let mut list_cmd = constraint_command();
    list_cmd.current_dir(&temp_dir)
        .arg("list");
    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 1 constraint(s)"))
        .stdout(predicate::str::contains("Second constraint"))
        .stdout(predicate::str::is_not(predicate::str::contains("First constraint")));

    // Delete second constraint
    let mut delete_cmd2 = constraint_command();
    delete_cmd2.current_dir(&temp_dir)
        .arg("delete")
        .arg(id2);
    delete_cmd2.assert().success();

    // Verify no constraints remain
    let mut list_cmd2 = constraint_command();
    list_cmd2.current_dir(&temp_dir)
        .arg("list");
    list_cmd2.assert()
        .success()
        .stdout(predicate::str::contains("No constraints found."));
}

#[test]
fn test_delete_help_output() {
    let mut cmd = run_constraint_command(&["delete", "--help"]);

    cmd.assert().success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot_with_settings!("delete_help_output", &stdout, &cli_snapshot_settings());
}