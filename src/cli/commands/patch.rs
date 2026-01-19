//! Implementation of the 'patch' command

use crate::cli::args::PatchArgs;
use crate::core::constraint::ConstraintUpdate;
use crate::storage::jsonl::JsonlStorage;
use crate::utils::workspace::Workspace;

/// Run the patch command
pub fn run(args: PatchArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Discover workspace
    let workspace = Workspace::discover()?;
    let storage = JsonlStorage::new(workspace.constraints_dir());

    // Read existing constraint
    let constraint_id = args.id.clone();
    let mut constraint = storage.read_constraint_by_id(&constraint_id)?;

    // Build update from provided arguments
    let mut update = ConstraintUpdate::default();

    if let Some(text) = args.text {
        update.text = Some(text);
    }

    if let Some(tags) = args.tags {
        update.tags = Some(tags);
    }

    if let Some(priority) = args.priority {
        update.priority = Some(Some(priority));
    }

    if let Some(references) = args.references {
        update.references = Some(references);
    }

    if let Some(verification) = args.verification {
        update.verification = Some(Some(verification));
    }

    // Apply the update
    constraint.update(update)?;

    // Write back to storage
    storage.write_constraint(&constraint)?;

    // Output result
    println!("Constraint {} updated successfully.", constraint.id);

    Ok(())
}
