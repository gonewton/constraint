//! # Newton Constraints CLI Tool
//!
//! A command-line tool for managing project constraints with RFC 2119 compliance.
//! Constraints are stored as versioned JSONL files and support automated validation.
//!
//! ## Features
//!
//! - Define constraints using RFC 2119 terminology (MUST, SHALL, SHOULD, MAY, FORBIDDEN)
//! - Organize constraints by categories (security, testing, performance, etc.)
//! - Search and browse existing constraints
//! - Automated compliance validation with verification commands
//! - Versioned storage with automatic migration support
//!
//! ## Usage
//!
//! ```bash
//! constraint add --type MUST --category security --text "All passwords must be hashed"
//! constraint list --category security
//! constraint validate --execute
//! ```

pub mod cli;
pub mod core;
pub mod storage;
pub mod utils;

/// Version of the constraint format
pub const CONSTRAINT_VERSION: u32 = 1;

/// Application name
pub const APP_NAME: &str = "constraint";

/// Re-export commonly used types
pub use core::constraint::{Constraint, ConstraintType, ValidationStatus};
pub use core::error::ConstraintError;
