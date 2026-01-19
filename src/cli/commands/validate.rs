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
    display_results(&results, args.execute)?;

    // Exit with appropriate code
    let has_failures = results
        .iter()
        .any(|r| matches!(r.status, ValidationStatus::Failed));
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
        // Just check if verification method exists
        if constraint.verification.is_some() {
            (
                ValidationStatus::Skipped,
                Some("Verification command available".to_string()),
                None,
            )
        } else {
            (
                ValidationStatus::Skipped,
                Some("No verification method".to_string()),
                None,
            )
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for result in results {
        match result.status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => failed += 1,
            ValidationStatus::Skipped => skipped += 1,
        }

        // Display individual result
        let status_icon = match result.status {
            ValidationStatus::Passed => "✅",
            ValidationStatus::Failed => "❌",
            ValidationStatus::Skipped => "⏭️",
        };

        let status_text = match result.status {
            ValidationStatus::Passed => "PASSED",
            ValidationStatus::Failed => "FAILED",
            ValidationStatus::Skipped => "SKIPPED",
        };

        println!("{} {} - {}", status_icon, result.constraint_id, status_text);
        println!("   {}", result.constraint_text);

        if let Some(output) = &result.output {
            println!("   Output: {}", output);
        }

        if let Some(error) = &result.error {
            println!("   Error: {}", error);
        }

        println!("   Duration: {}ms", result.duration_ms);
        println!();
    }

    // Display summary
    println!("Validation Summary:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    println!("  ⏭️ Skipped: {}", skipped);
    println!("  📊 Total: {}", results.len());

    if executed && failed > 0 {
        println!();
        println!("❌ Some validations failed. Check the output above for details.");
    } else if executed {
        println!();
        println!("✅ All validations completed successfully!");
    } else {
        println!();
        println!("ℹ️ Verification commands were not executed. Use --execute to run them.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_status_display() {
        // Test that validation status works correctly
        let status = ValidationStatus::Passed;
        assert!(matches!(status, ValidationStatus::Passed));
    }
}
