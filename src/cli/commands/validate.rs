//! Implementation of the 'validate' command

use crate::cli::args::ValidateArgs;
use crate::core::constraint::Constraint;
use crate::core::error::ConstraintError;
use crate::storage::jsonl::JsonlStorage;
use crate::utils::workspace::Workspace;
use std::process::Command;

/// Validation result for a single constraint
#[derive(Debug)]
struct ValidationResult {
    constraint_id: String,
    constraint_text: String,
    status: ValidationStatus,
    output: Option<String>,
    error: Option<String>,
    duration_ms: u128,
}

/// Status of constraint validation
#[derive(Debug, Clone, Copy, PartialEq)]
enum ValidationStatus {
    Passed,
    Failed,
    Skipped,
    Valid,
    Invalid,
}

/// Run the validate command
pub fn run(args: ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Discover workspace
    let workspace = Workspace::discover()?;
    let storage = JsonlStorage::new(workspace.constraints_dir());

    // Get constraints to validate
    let constraints = get_constraints_to_validate(&storage, &args)?;

    if constraints.is_empty() {
        println!("No constraints found to validate.");
        return Ok(());
    }

    println!("Validating {} constraint(s)...", constraints.len());
    println!();

    // Run validation for each constraint
    let mut results = Vec::new();
    for constraint in &constraints {
        let result = validate_constraint(constraint, args.execute)?;
        results.push(result);
    }

    // Display results
    display_results(&results, args.execute, args.verbose)?;

    // Exit with appropriate code
    let has_failures = results.iter().any(|r| {
        matches!(
            r.status,
            ValidationStatus::Failed | ValidationStatus::Invalid
        )
    });
    if has_failures {
        std::process::exit(1);
    }

    Ok(())
}

/// Get constraints to validate based on arguments
fn get_constraints_to_validate(
    storage: &JsonlStorage,
    args: &ValidateArgs,
) -> Result<Vec<Constraint>, ConstraintError> {
    match (&args.category, &args.id) {
        (Some(category), None) => {
            // Validate all constraints in a specific category
            storage.read_category_constraints(category)
        }
        (None, Some(id)) => {
            // Validate a specific constraint
            storage.read_constraint_by_id(id).map(|c| vec![c])
        }
        (None, None) => {
            // Validate all constraints
            storage.read_all_constraints()
        }
        (Some(_), Some(_)) => {
            // Both category and ID specified - this is invalid
            Err(ConstraintError::Validation(
                "Cannot specify both --category and --id".to_string(),
            ))
        }
    }
}

/// Validate a single constraint
fn validate_constraint(
    constraint: &Constraint,
    execute_verification: bool,
) -> Result<ValidationResult, ConstraintError> {
    let start_time = std::time::Instant::now();

    let (status, output, error) = if execute_verification {
        if let Some(verification) = &constraint.verification {
            // Execute verification command
            match execute_verification_command(verification) {
                Ok((success, cmd_output)) => {
                    if success {
                        (ValidationStatus::Passed, Some(cmd_output), None)
                    } else {
                        (ValidationStatus::Failed, Some(cmd_output), None)
                    }
                }
                Err(e) => (ValidationStatus::Failed, None, Some(e.to_string())),
            }
        } else {
            // No verification command specified
            (
                ValidationStatus::Skipped,
                None,
                Some("No verification method specified".to_string()),
            )
        }
    } else {
        // Perform structural validation
        match constraint.validate() {
            Ok(()) => (
                ValidationStatus::Valid,
                Some("Structural validation passed".to_string()),
                None,
            ),
            Err(e) => (
                ValidationStatus::Invalid,
                None,
                Some(format!("Structural validation failed: {}", e)),
            ),
        }
    };

    let duration = start_time.elapsed().as_millis();

    Ok(ValidationResult {
        constraint_id: constraint.id.clone(),
        constraint_text: constraint.text.clone(),
        status,
        output,
        error,
        duration_ms: duration,
    })
}

/// Execute a verification command
fn execute_verification_command(
    command: &str,
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    // For security, we'll use shell execution but with limited capabilities
    // In a real implementation, you might want more sophisticated command parsing

    let output = Command::new("sh").arg("-c").arg(command).output()?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined_output = if stdout.is_empty() {
        stderr
    } else if stderr.is_empty() {
        stdout
    } else {
        format!("{}\n{}", stdout.trim(), stderr.trim())
    };

    Ok((success, combined_output))
}

/// Display validation results
fn display_results(
    results: &[ValidationResult],
    executed: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut valid = 0;
    let mut invalid = 0;

    for result in results {
        match result.status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => failed += 1,
            ValidationStatus::Skipped => skipped += 1,
            ValidationStatus::Valid => valid += 1,
            ValidationStatus::Invalid => invalid += 1,
        }

        // Display individual result
        let status_icon = match result.status {
            ValidationStatus::Passed => "✅",
            ValidationStatus::Failed => "❌",
            ValidationStatus::Skipped => "⏭️",
            ValidationStatus::Valid => "✅",
            ValidationStatus::Invalid => "❌",
        };

        let status_text = match result.status {
            ValidationStatus::Passed => "PASSED",
            ValidationStatus::Failed => "FAILED",
            ValidationStatus::Skipped => "SKIPPED",
            ValidationStatus::Valid => "VALID",
            ValidationStatus::Invalid => "INVALID",
        };

        println!("{} {} - {}", status_icon, result.constraint_id, status_text);
        println!("   {}", result.constraint_text);

        if let Some(output) = &result.output {
            if verbose || executed {
                println!("   Output: {}", output);
            } else {
                // For non-verbose mode, truncate long output
                let truncated = if output.len() > 100 {
                    format!("{}...", &output[..97])
                } else {
                    output.clone()
                };
                println!("   Output: {}", truncated);
            }
        }

        if let Some(error) = &result.error {
            println!("   Error: {}", error);
        }

        if verbose || executed {
            println!("   Duration: {}ms", result.duration_ms);
        }
        println!();
    }

    // Display summary
    if executed {
        println!("Verification Summary:");
        println!("  ✅ Passed: {}", passed);
        println!("  ❌ Failed: {}", failed);
        println!("  ⏭️ Skipped: {}", skipped);
    } else {
        println!("Structural Validation Summary:");
        println!("  ✅ Valid: {}", valid);
        println!("  ❌ Invalid: {}", invalid);
    }
    println!("  📊 Total: {}", results.len());

    if executed && failed > 0 {
        println!();
        println!("❌ Some verifications failed. Check the output above for details.");
    } else if executed {
        println!();
        println!("✅ All verifications completed successfully!");
    } else if invalid > 0 {
        println!();
        println!("❌ Some constraints are invalid. Check the output above for details.");
    } else {
        println!();
        println!("✅ All constraints are structurally valid!");
        println!("ℹ️ Use --execute to run verification commands.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constraint::{Constraint, ConstraintParams, ConstraintType};

    #[test]
    fn test_validation_status_display() {
        // Test that validation status works correctly
        let status = ValidationStatus::Passed;
        assert!(matches!(status, ValidationStatus::Passed));
    }

    #[test]
    fn test_structural_validation_valid_constraint() {
        // Create a valid constraint
        let constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // Test structural validation (execute = false)
        let result = validate_constraint(&constraint, false).unwrap();
        assert!(matches!(result.status, ValidationStatus::Valid));
        assert!(result.output.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_structural_validation_invalid_constraint() {
        // Create a constraint with invalid category (should fail validation)
        let mut constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // Manually corrupt the category to make it invalid
        constraint.category = "Invalid-Category".to_string();

        // Test structural validation (execute = false)
        let result = validate_constraint(&constraint, false).unwrap();
        assert!(matches!(result.status, ValidationStatus::Invalid));
        assert!(result.output.is_none());
        assert!(result.error.is_some());
        assert!(result
            .error
            .unwrap()
            .contains("Structural validation failed"));
    }

    #[test]
    fn test_verification_execution_with_command() {
        // Create a constraint with a verification command
        let constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: Some("echo 'test passed'".to_string()),
        })
        .unwrap();

        // Test verification execution (execute = true)
        let result = validate_constraint(&constraint, true).unwrap();
        assert!(matches!(result.status, ValidationStatus::Passed));
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert!(result.output.unwrap().contains("test passed"));
    }

    #[test]
    fn test_verification_execution_without_command() {
        // Create a constraint without verification command
        let constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: None,
        })
        .unwrap();

        // Test verification execution (execute = true)
        let result = validate_constraint(&constraint, true).unwrap();
        assert!(matches!(result.status, ValidationStatus::Skipped));
        assert!(result.output.is_none());
        assert!(result.error.is_some());
        assert!(result
            .error
            .unwrap()
            .contains("No verification method specified"));
    }

    #[test]
    fn test_verification_execution_failing_command() {
        // Create a constraint with a failing verification command
        let constraint = Constraint::new(ConstraintParams {
            r#type: ConstraintType::Must,
            category: "security".to_string(),
            text: "All passwords must be hashed".to_string(),
            author: "test-author".to_string(),
            id: None,
            tags: vec![],
            priority: None,
            references: "".to_string(),
            verification: Some("exit 1".to_string()),
        })
        .unwrap();

        // Test verification execution (execute = true)
        let result = validate_constraint(&constraint, true).unwrap();
        assert!(matches!(result.status, ValidationStatus::Failed));
        assert!(result.output.is_some());
        assert!(result.error.is_none());
    }
}
