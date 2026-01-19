//! Command-line interface for the Newton Constraints tool

pub mod args;
pub mod commands;

pub use args::*;
pub use commands::*;

/// Run the CLI application
pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Add(args) => commands::add::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Search(args) => commands::search::run(args),
        Commands::Patch(args) => commands::patch::run(args),
        Commands::Delete(args) => commands::delete::run(args),
        Commands::Validate(args) => commands::validate::run(args),
    }
}
