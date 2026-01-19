//! Implementation of the 'delete' command

use crate::cli::args::DeleteArgs;
use crate::storage::jsonl::JsonlStorage;
use crate::utils::workspace::Workspace;

/// Run the delete command
pub fn run(args: DeleteArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Discover workspace
    let workspace = Workspace::discover()?;
    let storage = JsonlStorage::new(workspace.constraints_dir());

    // Read the constraint first to get its category (for user feedback)
    let constraint = storage.read_constraint_by_id(&args.id)?;
    let category = constraint.category.clone();

    // Delete the constraint
    storage.delete_constraint(&category, &args.id)?;

    // Output result
    println!("Constraint {} deleted successfully.", args.id);

    Ok(())
}
