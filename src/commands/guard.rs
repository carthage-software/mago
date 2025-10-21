//! Architectural guard command implementation.
//!
//! This module implements the `mago guard` command, which enforces architectural
//! rules and layer dependencies in PHP codebases. The guard helps maintain clean
//! architecture by ensuring code follows defined structural constraints.
//!
//! # Purpose
//!
//! The guard command validates that:
//!
//! - **Layer Boundaries**: Different architectural layers respect dependency rules
//! - **Dependency Direction**: Dependencies flow in the correct direction
//! - **Symbol Access**: Only allowed symbol types are accessed across boundaries
//! - **Namespace Isolation**: Namespaces remain properly isolated
//!
//! # Guard Rules
//!
//! Rules are defined in `mago.toml` under the `[guard]` section and specify:
//!
//! - Which namespaces/layers exist
//! - What dependencies are allowed between them
//! - Which symbol types (classes, functions, etc.) are permitted
//! - Exceptions for specific cases
//!
//! # Analysis Process
//!
//! The guard analyzes symbol dependencies by:
//!
//! 1. Building a complete codebase model with all symbols
//! 2. Tracking all symbol references and dependencies
//! 3. Validating each dependency against the defined rules
//! 4. Reporting violations as issues
//!
//! # Common Use Cases
//!
//! - Enforcing hexagonal/onion architecture
//! - Preventing domain layer from depending on infrastructure
//! - Ensuring presentation layer doesn't access data layer directly
//! - Maintaining module boundaries in modular monoliths

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_prelude::Prelude;

use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Command for enforcing architectural rules and layer dependencies.
///
/// The `guard` command performs architectural boundary checking on your PHP codebase.
/// It analyzes symbol dependencies and ensures they comply with the architectural rules
/// defined in your configuration.
///
/// Guard helps enforce:
///
/// • Layer boundaries between different parts of your application
/// • Dependency direction rules (e.g., domain should not depend on infrastructure)
/// • Allowed symbol types for specific dependencies
/// • Namespace isolation and architectural constraints
///
/// You can define rules in your `mago.toml` file to specify which namespaces can
/// depend on others and what types of symbols are allowed.
#[derive(Parser, Debug)]
#[command(name = "guard")]
pub struct GuardCommand {
    /// Specific files or directories to check instead of using configuration.
    ///
    /// When provided, these paths override the source configuration in mago.toml.
    /// The guard will focus only on the specified files or directories.
    ///
    /// This is useful for targeted checking, testing changes, or integrating
    /// with development workflows and CI systems.
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Disable built-in PHP and library stubs for checking.
    ///
    /// By default, guard uses stubs for built-in PHP functions and popular
    /// libraries to provide accurate symbol information. Disabling this may result
    /// in more warnings when external symbols can't be resolved.
    #[arg(long, default_value_t = false)]
    pub no_stubs: bool,

    /// Arguments related to reporting issues with baseline support.
    #[clap(flatten)]
    pub baseline_reporting: BaselineReportingArgs,
}

impl GuardCommand {
    /// Executes the architectural guard checking process.
    ///
    /// This method orchestrates the complete guard validation workflow:
    ///
    /// 1. **Load Prelude**: Decode embedded stubs for PHP built-ins (unless `--no-stubs`)
    /// 2. **Create Orchestrator**: Initialize with configuration and color settings
    /// 3. **Apply Overrides**: Use `path` argument if provided to override config paths
    /// 4. **Load Database**: Scan workspace and include external files for context
    /// 5. **Validate Files**: Ensure at least one host file exists to check
    /// 6. **Create Service**: Initialize guard service with database and codebase metadata
    /// 7. **Run Checks**: Validate dependencies against architectural rules
    /// 8. **Process Results**: Report violations through baseline processor
    ///
    /// # Arguments
    ///
    /// * `configuration` - The loaded configuration containing guard rules
    /// * `color_choice` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` if checking completed successfully
    /// - `Err(Error)` if database loading, checking, or reporting failed
    ///
    /// # Guard Rules
    ///
    /// Rules are read from `configuration.guard.rules` and define which dependencies
    /// are allowed between different namespaces or layers. Violations are reported
    /// as issues with details about the forbidden dependency.
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let Prelude { database, metadata, .. } = if self.no_stubs {
            Prelude::default()
        } else {
            Prelude::decode(PRELUDE_BYTES).expect("Failed to decode embedded prelude")
        };

        let mut orchestrator = create_orchestrator(&configuration, color_choice, false);
        orchestrator.add_exclude_patterns(configuration.guard.excludes.iter());
        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter());
        }

        let database = orchestrator.load_database(&configuration.source.workspace, true, Some(database))?;

        if !database.files().any(|f| f.file_type == FileType::Host) {
            tracing::warn!("No files found to check with guard.");

            return Ok(ExitCode::SUCCESS);
        }

        let service = orchestrator.get_guard_service(database.read_only(), metadata);
        let issues = service.run()?;

        let baseline = configuration.guard.baseline.as_deref();
        let processor = self.baseline_reporting.get_processor(orchestrator, database, color_choice, baseline);

        processor.process_issues(issues)
    }
}
