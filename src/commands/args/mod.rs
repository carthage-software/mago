//! Shared command-line argument structures.
//!
//! This module provides reusable argument groups that can be flattened into
//! multiple command definitions using [`clap`]'s flatten functionality. This
//! promotes consistency across commands and reduces duplication.
//!
//! # Argument Groups
//!
//! - [`reporting::ReportingArgs`]: Core issue reporting and fixing options
//! - [`baseline_reporting::BaselineReportingArgs`]: Baseline management combined with reporting
//!
//! # Design Pattern
//!
//! These argument structs use the builder pattern with [`clap::Parser`] to define
//! reusable CLI argument sets. Commands that need reporting functionality can
//! flatten these structs into their own argument definitions:
//!
//! ```ignore
//! #[derive(Parser)]
//! struct MyCommand {
//!     #[clap(flatten)]
//!     reporting: ReportingArgs,
//! }
//! ```
//!
//! This ensures consistent behavior and documentation across all commands that
//! support reporting or baseline functionality.

pub mod baseline_reporting;
pub mod reporting;
