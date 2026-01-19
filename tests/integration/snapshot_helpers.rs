//! Helper utilities for snapshot testing

use assert_cmd::Command;
use std::path::Path;

/// Run constraint command and return result for snapshot testing
pub fn run_constraint_command(args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("constraint").unwrap();
    cmd.args(args);
    cmd
}

/// Get snapshot settings for CLI output
pub fn cli_snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings
}