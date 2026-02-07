//! Command-line argument definitions using clap

use clap::{Parser, Subcommand};

/// Newton Constraints CLI - Manage project constraints with RFC 2119 compliance
#[derive(Parser)]
#[command(name = "constraint")]
#[command(about = "A CLI tool for managing project constraints with RFC 2119 compliance")]
#[command(version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new constraint
    Add(AddArgs),
    /// List constraints
    List(ListArgs),
    /// Search constraints by text
    Search(SearchArgs),
    /// Update an existing constraint
    Patch(PatchArgs),
    /// Delete a constraint
    Delete(DeleteArgs),
    /// Validate constraint compliance
    Validate(ValidateArgs),
}

/// Arguments for adding a constraint
#[derive(Parser)]
pub struct AddArgs {
    /// Constraint type (MUST, SHALL, SHOULD, MAY, FORBIDDEN)
    #[arg(short = 'T', long)]
    pub r#type: String,

    /// Category for the constraint (e.g., security, testing, performance)
    #[arg(short = 'c', long)]
    pub category: String,

    /// Constraint text/description
    #[arg(short = 'x', long)]
    pub text: String,

    /// Author of the constraint
    #[arg(short = 'A', long)]
    pub author: String,

    /// Explicit constraint ID (optional, auto-generated if not provided)
    #[arg(short = 'i', long)]
    pub id: Option<String>,

    /// Comma-separated list of tags
    #[arg(short = 'g', long, value_delimiter = ',')]
    pub tags: Vec<String>,

    /// Priority level (P1, P2, P3)
    #[arg(short = 'P', long)]
    pub priority: Option<String>,

    /// Reference information
    #[arg(short = 'R', long)]
    pub references: Option<String>,

    /// Verification command/script
    #[arg(short = 'V', long)]
    pub verification: Option<String>,
}

/// Arguments for listing constraints
#[derive(Parser)]
pub struct ListArgs {
    /// Filter by category
    #[arg(short = 'C', long)]
    pub category: Option<String>,

    /// Output format (human, json)
    #[arg(short = 'o', long, default_value = "human")]
    pub format: String,
}

/// Arguments for searching constraints
#[derive(Parser)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Limit search to specific category
    #[arg(short = 'C', long)]
    pub category: Option<String>,

    /// Output format (human, json)
    #[arg(short = 'o', long, default_value = "human")]
    pub format: String,
}

/// Arguments for patching a constraint
#[derive(Parser)]
pub struct PatchArgs {
    /// Constraint ID to update
    pub id: String,

    /// Updated constraint text
    #[arg(short = 'x', long)]
    pub text: Option<String>,

    /// Updated tags (comma-separated)
    #[arg(short = 'g', long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Updated priority (P1, P2, P3)
    #[arg(short = 'P', long)]
    pub priority: Option<String>,

    /// Updated references
    #[arg(short = 'R', long)]
    pub references: Option<String>,

    /// Updated verification command
    #[arg(short = 'V', long)]
    pub verification: Option<String>,
}

/// Arguments for deleting a constraint
#[derive(Parser)]
pub struct DeleteArgs {
    /// Constraint ID to delete
    pub id: String,
}

/// Arguments for validating constraints
#[derive(Parser)]
pub struct ValidateArgs {
    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,

    /// Validate specific constraint ID
    #[arg(short = 'I', long)]
    pub id: Option<String>,

    /// Execute verification commands
    #[arg(short = 'E', long)]
    pub execute: bool,

    /// Verbose output with full verification details
    #[arg(short = 'v', long)]
    pub verbose: bool,
}
