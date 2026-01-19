//! Implementation of the 'list' command

use crate::cli::args::ListArgs;
use crate::storage::jsonl::JsonlStorage;
use crate::utils::workspace::Workspace;

/// Run the list command
pub fn run(args: ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Discover workspace
    let workspace = Workspace::discover()?;
    let storage = JsonlStorage::new(workspace.constraints_dir());

    // Get constraints
    let constraints = if let Some(category) = &args.category {
        storage.read_category_constraints(category)?
    } else {
        storage.read_all_constraints()?
    };

    // Output results
    if args.format == "json" {
        output_json(&constraints)?;
    } else {
        output_human(&constraints)?;
    }

    Ok(())
}

/// Output constraints in JSON format
fn output_json(
    constraints: &[crate::core::constraint::Constraint],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(constraints)?);
    Ok(())
}

/// Output constraints in human-readable format
fn output_human(
    constraints: &[crate::core::constraint::Constraint],
) -> Result<(), Box<dyn std::error::Error>> {
    if constraints.is_empty() {
        println!("No constraints found.");
        return Ok(());
    }

    println!("Found {} constraint(s):", constraints.len());
    println!();

    for constraint in constraints {
        println!(
            "{}: {} [{}] {}",
            constraint.id,
            format_constraint_type(&constraint.r#type),
            constraint.category,
            constraint.text
        );

        println!(
            "  Author: {} | Created: {} | Status: {}",
            constraint.author,
            constraint.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            format_validation_status(&constraint.validation_status)
        );

        if !constraint.tags.is_empty() {
            println!("  Tags: {}", constraint.tags.join(", "));
        }

        if let Some(priority) = &constraint.priority {
            println!("  Priority: {}", priority);
        }

        if let Some(verification) = &constraint.verification {
            println!("  Verification: {}", verification);
        }

        if !constraint.references.is_empty() {
            println!("  References: {}", constraint.references);
        }

        println!();
    }

    Ok(())
}

/// Format constraint type for display
fn format_constraint_type(
    constraint_type: &crate::core::constraint::ConstraintType,
) -> &'static str {
    match constraint_type {
        crate::core::constraint::ConstraintType::Must => "MUST",
        crate::core::constraint::ConstraintType::Shall => "SHALL",
        crate::core::constraint::ConstraintType::Should => "SHOULD",
        crate::core::constraint::ConstraintType::May => "MAY",
        crate::core::constraint::ConstraintType::Forbidden => "FORBIDDEN",
    }
}

/// Format validation status for display
fn format_validation_status(status: &crate::core::constraint::ValidationStatus) -> &'static str {
    match status {
        crate::core::constraint::ValidationStatus::Valid => "Valid",
        crate::core::constraint::ValidationStatus::Invalid => "Invalid",
        crate::core::constraint::ValidationStatus::Warning => "Warning",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constraint::ConstraintType;

    #[test]
    fn test_format_constraint_type() {
        assert_eq!(format_constraint_type(&ConstraintType::Must), "MUST");
        assert_eq!(format_constraint_type(&ConstraintType::Shall), "SHALL");
        assert_eq!(format_constraint_type(&ConstraintType::Should), "SHOULD");
        assert_eq!(format_constraint_type(&ConstraintType::May), "MAY");
        assert_eq!(
            format_constraint_type(&ConstraintType::Forbidden),
            "FORBIDDEN"
        );
    }

    #[test]
    fn test_format_validation_status() {
        use crate::core::constraint::ValidationStatus;
        assert_eq!(format_validation_status(&ValidationStatus::Valid), "Valid");
        assert_eq!(
            format_validation_status(&ValidationStatus::Invalid),
            "Invalid"
        );
        assert_eq!(
            format_validation_status(&ValidationStatus::Warning),
            "Warning"
        );
    }
}
