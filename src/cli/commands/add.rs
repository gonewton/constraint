//! Implementation of the 'add' command

use crate::cli::args::AddArgs;
use crate::core::constraint::{Constraint, ConstraintType};
use crate::core::error::ConstraintError;
use crate::utils::workspace::Workspace;

/// Run the add command
pub fn run(args: AddArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Parse constraint type
    let constraint_type = parse_constraint_type(&args.r#type)?;

    // Validate category format
    validate_category(&args.category)?;

    // Create constraint
    let constraint = Constraint::new(crate::core::constraint::ConstraintParams {
        r#type: constraint_type,
        category: args.category.clone(),
        text: args.text.clone(),
        author: args.author.clone(),
        id: args.id.clone(),
        tags: args.tags.clone(),
        priority: args.priority.clone(),
        references: args.references.unwrap_or_default(),
        verification: args.verification.clone(),
    })?;

    // Initialize workspace if needed
    let workspace = Workspace::discover().or_else(|_: crate::core::error::ConstraintError| -> Result<Workspace, Box<dyn std::error::Error>> {
        // If workspace doesn't exist, create it
        let workspace = Workspace::discover().unwrap_or_else(|_| {
            // Create workspace in current directory
            let current = std::env::current_dir().unwrap();
            let newton_dir = current.join(".newton");
            std::fs::create_dir_all(&newton_dir).unwrap();
            std::fs::create_dir_all(newton_dir.join("constraints")).unwrap();
            Workspace::new(newton_dir)
        });
        workspace.ensure_structure()?;
        Ok(workspace)
    })?;

    // Save constraint
    save_constraint(&workspace, &constraint)?;

    // Output result
    if args.id.is_none() {
        println!("Constraint added with ID: {}", constraint.id);
    } else {
        println!("Constraint added: {}", constraint.id);
    }

    Ok(())
}

/// Parse constraint type from string
fn parse_constraint_type(type_str: &str) -> Result<ConstraintType, ConstraintError> {
    match type_str.to_uppercase().as_str() {
        "MUST" => Ok(ConstraintType::Must),
        "SHALL" => Ok(ConstraintType::Shall),
        "SHOULD" => Ok(ConstraintType::Should),
        "MAY" => Ok(ConstraintType::May),
        "FORBIDDEN" => Ok(ConstraintType::Forbidden),
        _ => Err(ConstraintError::InvalidConstraintType(type_str.to_string())),
    }
}

/// Validate category format
fn validate_category(category: &str) -> Result<(), ConstraintError> {
    let regex = regex::Regex::new(r"^[a-z0-9-]+$")
        .map_err(|_| ConstraintError::Validation("Invalid category regex".to_string()))?;

    if !regex.is_match(category) {
        return Err(ConstraintError::Validation(
            "Category must be lowercase alphanumeric with hyphens only".to_string(),
        ));
    }

    Ok(())
}

/// Save constraint to storage
fn save_constraint(workspace: &Workspace, constraint: &Constraint) -> Result<(), ConstraintError> {
    use crate::storage::jsonl::JsonlStorage;
    use std::fs;

    // Ensure category directory exists
    let category_dir = workspace.category_dir(&constraint.category);
    fs::create_dir_all(&category_dir)?;

    // Create storage instance
    let storage = JsonlStorage::new(workspace.constraints_dir());

    // Save constraint
    storage.write_constraint(constraint)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constraint_type() {
        assert!(matches!(
            parse_constraint_type("MUST"),
            Ok(ConstraintType::Must)
        ));
        assert!(matches!(
            parse_constraint_type("SHALL"),
            Ok(ConstraintType::Shall)
        ));
        assert!(matches!(
            parse_constraint_type("SHOULD"),
            Ok(ConstraintType::Should)
        ));
        assert!(matches!(
            parse_constraint_type("MAY"),
            Ok(ConstraintType::May)
        ));
        assert!(matches!(
            parse_constraint_type("FORBIDDEN"),
            Ok(ConstraintType::Forbidden)
        ));

        assert!(matches!(
            parse_constraint_type("invalid"),
            Err(ConstraintError::InvalidConstraintType(_))
        ));
    }

    #[test]
    fn test_validate_category() {
        assert!(validate_category("security").is_ok());
        assert!(validate_category("test-category").is_ok());
        assert!(validate_category("category123").is_ok());

        assert!(validate_category("INVALID").is_err());
        assert!(validate_category("category_name").is_err());
        assert!(validate_category("category@name").is_err());
    }
}
