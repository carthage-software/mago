//! Orchestrator for managing and coordinating Mago's analysis tools.
//!
//! The orchestrator crate provides a high-level interface for running various static analysis
//! tasks on PHP codebases. It coordinates between the database, parser, analyzer, linter,
//! formatter, and architectural guard to provide a unified workflow.
//!
//! # Architecture
//!
//! The orchestrator follows a service-oriented architecture where each tool (linter, analyzer,
//! formatter, guard) is encapsulated in its own service. The [`Orchestrator`] struct acts as
//! a factory and coordinator, managing:
//!
//! - **Database**: File system scanning and caching via [`mago_database::Database`]
//! - **Codebase**: Metadata and symbol references via [`mago_codex`]
//! - **Services**: Tool-specific services that operate on the database and codebase
//!
//! # Services
//!
//! The orchestrator provides four main services:
//!
//! - [`LintService`]: Runs linting rules on PHP code
//! - [`AnalysisService`]: Performs static analysis
//! - [`GuardService`]: Enforces architectural rules
//! - [`FormatService`]: Formats PHP code
//!
//! # Workflow
//!
//! A typical workflow involves:
//!
//! 1. Create an [`Orchestrator`] with an [`OrchestratorConfiguration`]
//! 2. Load the database using [`Orchestrator::load_database`]
//! 3. Obtain the desired service (e.g., [`Orchestrator::get_lint_service`])
//! 4. Run the service to get results

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use bumpalo::Bump;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::plugin::create_registry_with_plugins;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;
use mago_database::DatabaseConfiguration;
use mago_database::ReadDatabase;
use mago_database::exclusion::Exclusion;
use mago_database::file::File;
use mago_database::loader::DatabaseLoader;

use crate::service::analysis::AnalysisService;
use crate::service::format::FileFormatStatus;
use crate::service::format::FormatService;
use crate::service::guard::GuardService;
use crate::service::incremental_analysis::IncrementalAnalysisService;
use crate::service::lint::LintService;

pub use config::OrchestratorConfiguration;
pub use error::OrchestratorError;

pub mod config;
pub mod error;
pub mod progress;
pub mod service;

/// The main orchestrator for running operations on PHP code.
///
/// The [`Orchestrator`] is the central coordinator that provides factory methods for creating
/// various services (linting, analysis, formatting, guarding) and manages the shared configuration
/// and database loading.
///
/// # Responsibilities
///
/// - **Configuration Management**: Stores and provides access to the configuration for all services
/// - **Database Loading**: Handles file system scanning and database initialization
/// - **Service Creation**: Acts as a factory for creating tool-specific services
/// - **Path Management**: Manages source paths and exclusion patterns
#[derive(Debug)]
pub struct Orchestrator<'a> {
    /// Configuration for all operations.
    pub config: OrchestratorConfiguration<'a>,
    /// Plugin registry for the analyzer, lazily initialized.
    plugin_registry: OnceLock<Arc<PluginRegistry>>,
}

impl<'a> Orchestrator<'a> {
    /// Creates a new orchestrator with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration specifying PHP version, paths, tool settings, etc.
    #[must_use]
    pub fn new(config: OrchestratorConfiguration<'a>) -> Self {
        Self { config, plugin_registry: OnceLock::new() }
    }

    /// Gets the analyzer plugin registry, initializing it if necessary.
    ///
    /// This method returns a shared reference to the plugin registry used by the analysis service.
    /// If the registry has not been initialized yet, it creates a new one based on the configuration.
    ///
    /// # Returns
    ///
    /// An `Arc` pointing to the `PluginRegistry`.
    pub fn get_analyzer_plugin_registry(&self) -> Arc<PluginRegistry> {
        Arc::clone(self.plugin_registry.get_or_init(|| {
            Arc::new(create_registry_with_plugins(
                &self.config.analyzer_plugins,
                self.config.disable_default_analyzer_plugins,
            ))
        }))
    }

    /// Adds additional exclusion patterns to the orchestrator's configuration.
    ///
    /// These patterns will be used when loading the database to exclude files and directories
    /// from scanning. Patterns can be glob patterns (e.g., `"*.tmp"`, `"vendor/*"`) or
    /// direct paths.
    ///
    /// # Arguments
    ///
    /// * `patterns` - A vector of string patterns to exclude from file scanning
    pub fn add_exclude_patterns<T>(&mut self, patterns: impl Iterator<Item = &'a T>)
    where
        T: AsRef<str> + 'a,
    {
        self.config.excludes.extend(patterns.map(std::convert::AsRef::as_ref));
    }

    /// Sets new source paths and moves the old paths to the includes list.
    ///
    /// This method replaces the current source paths with the provided paths and moves
    /// the old source paths to the includes list. This is useful when you want to change
    /// the primary analysis targets while keeping the old paths as context providers.
    ///
    /// # Arguments
    ///
    /// * `paths` - The new source paths or glob patterns to analyze
    pub fn set_source_paths(&mut self, paths: impl IntoIterator<Item = impl AsRef<str>>) {
        let old_paths = std::mem::take(&mut self.config.paths);

        self.config.paths = paths.into_iter().map(|p| p.as_ref().to_string()).collect();
        self.config.includes.extend(old_paths);
    }

    /// Loads the database by scanning the file system according to the configuration.
    ///
    /// This method scans the workspace directory and builds a database of all PHP files
    /// according to the configured paths, includes, excludes, and extensions. The database
    /// provides fast access to file contents and metadata for all tools.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The root directory of the project to analyze
    /// * `include_externals` - Whether to include files from the `includes` list in the database.
    ///   External files (e.g., vendor dependencies) provide context for analysis but are not
    ///   directly analyzed, linted, or formatted.
    /// * `prelude_database` - An optional pre-existing database to merge with. This is useful
    ///   for including standard library or framework stubs.
    /// * `stdin_override` - When set, the file with the given logical name (workspace-relative path)
    ///   uses the given content instead of being read from disk. Used for editor integrations
    ///   (e.g. `--stdin-input`), so baseline and reporting use the real path.
    ///
    /// # Returns
    ///
    /// Returns a [`Database`] containing all discovered PHP files, or an [`OrchestratorError`]
    /// if the database could not be loaded.
    pub fn load_database<'b>(
        &'b self,
        workspace: &'a Path,
        include_externals: bool,
        prelude_database: Option<Database<'static>>,
        stdin_override: Option<(String, String)>,
    ) -> Result<Database<'a>, OrchestratorError>
    where
        'b: 'a,
    {
        /// Converts string patterns from the configuration into `Exclusion` types.
        fn create_excludes_from_patterns<'a>(patterns: &[&'a str], root: &Path) -> Vec<Exclusion<'a>> {
            patterns
                .iter()
                .map(|pattern| {
                    if pattern.contains('*') {
                        if let Some(stripped) = pattern.strip_prefix("./") {
                            let rooted_pattern = root.join(stripped).to_string_lossy().into_owned();

                            Exclusion::Pattern(Cow::Owned(rooted_pattern))
                        } else {
                            Exclusion::Pattern(Cow::Borrowed(pattern))
                        }
                    } else {
                        let path = PathBuf::from(pattern);
                        let path_buf = if path.is_absolute() { path } else { root.join(path) };

                        Exclusion::Path(Cow::Owned(path_buf.canonicalize().unwrap_or(path_buf)))
                    }
                })
                .collect()
        }

        let includes = if include_externals {
            self.config.includes.iter().map(|s| Cow::Borrowed(s.as_ref())).collect::<Vec<Cow<'a, str>>>()
        } else {
            Vec::new()
        };

        let configuration: DatabaseConfiguration<'a> = DatabaseConfiguration {
            workspace: Cow::Borrowed(workspace),
            paths: self.config.paths.iter().map(|s| Cow::Borrowed(s.as_ref())).collect(),
            includes,
            excludes: create_excludes_from_patterns(&self.config.excludes, workspace),
            extensions: self.config.extensions.iter().map(|s| Cow::Borrowed(*s)).collect(),
        };

        let mut loader = DatabaseLoader::new(configuration);

        if let Some(prelude_db) = prelude_database {
            loader = loader.with_database(prelude_db);
        }

        if let Some((name, content)) = stdin_override {
            loader = loader.with_stdin_override(name, content);
        }

        let result = loader.load().map_err(OrchestratorError::Database)?;

        Ok(result)
    }

    /// Creates a linting service with the current configuration.
    ///
    /// The linting service checks PHP code against a set of rules to identify potential
    /// issues, style violations, and code smells.
    ///
    /// # Arguments
    ///
    /// * `database` - A read-only database handle containing the PHP files to lint
    ///
    /// # Returns
    ///
    /// A [`LintService`] configured with the orchestrator's linter settings and progress bar preferences.
    pub fn get_lint_service(&self, database: ReadDatabase) -> LintService {
        LintService::new(
            database,
            self.config.linter_settings.clone(),
            self.config.parser_settings,
            self.config.use_progress_bars,
        )
    }

    /// Creates an architectural guard service with the current configuration.
    ///
    /// The guard service enforces architectural constraints and layer dependencies in your
    /// codebase, ensuring that code follows the defined architectural rules.
    ///
    /// # Arguments
    ///
    /// * `database` - A read-only database handle containing the PHP files to check
    /// * `codebase` - Metadata about the codebase structure and symbols
    ///
    /// # Returns
    ///
    /// A [`GuardService`] configured with the orchestrator's guard settings and progress bar preferences.
    pub fn get_guard_service(&self, database: ReadDatabase, codebase: CodebaseMetadata) -> GuardService {
        GuardService::new(
            database,
            codebase,
            self.config.guard_settings.clone(),
            self.config.parser_settings,
            self.config.use_progress_bars,
        )
    }

    /// Creates a static analysis service with the current configuration.
    ///
    /// The analysis service performs deep static analysis on PHP code, including type checking,
    /// control flow analysis, and detection of logical errors and type mismatches.
    ///
    /// For incremental/watch mode analysis, use [`get_incremental_analysis_service`](Self::get_incremental_analysis_service) instead.
    ///
    /// # Arguments
    ///
    /// * `database` - A read-only database handle containing the PHP files to analyze
    /// * `codebase` - Metadata about the codebase structure and symbols
    /// * `symbol_references` - Information about symbol usage and references across the codebase
    ///
    /// # Returns
    ///
    /// An [`AnalysisService`] configured with the orchestrator's analyzer settings and progress bar preferences.
    pub fn get_analysis_service(
        &self,
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
    ) -> AnalysisService {
        AnalysisService::new(
            database,
            codebase,
            symbol_references,
            self.config.analyzer_settings.clone(),
            self.config.parser_settings,
            self.config.use_progress_bars,
            self.get_analyzer_plugin_registry(),
        )
    }

    /// Creates an incremental analysis service for watch mode or LSP integration.
    ///
    /// The service manages its own incremental state internally and provides a clean API
    /// for running full and incremental analysis without being coupled to CLI output
    /// or file watchers.
    ///
    /// # Arguments
    ///
    /// * `database` - A read-only database handle containing the PHP files to analyze
    /// * `codebase` - Base codebase metadata (prelude only, no user symbols)
    /// * `symbol_references` - Base symbol references (prelude only)
    ///
    /// # Returns
    ///
    /// An [`IncrementalAnalysisService`] ready for analysis.
    pub fn get_incremental_analysis_service(
        &self,
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
    ) -> IncrementalAnalysisService {
        IncrementalAnalysisService::new(
            database,
            codebase,
            symbol_references,
            self.config.analyzer_settings.clone(),
            self.config.parser_settings,
            self.get_analyzer_plugin_registry(),
        )
    }

    /// Creates a code formatting service with the current configuration.
    ///
    /// The formatting service formats PHP code according to the configured style settings,
    /// ensuring consistent code style across the codebase.
    ///
    /// # Arguments
    ///
    /// * `database` - A read-only database handle containing the PHP files to format
    ///
    /// # Returns
    ///
    /// A [`FormatService`] configured with the orchestrator's formatter settings, PHP version,
    /// and progress bar preferences.
    pub fn get_format_service(&self, database: ReadDatabase) -> FormatService {
        FormatService::new(
            database,
            self.config.php_version,
            self.config.formatter_settings,
            self.config.parser_settings,
            self.config.use_progress_bars,
        )
    }

    /// Formats a single file according to the configured style settings.
    ///
    /// This is a convenience method for formatting an individual file without requiring
    /// a full database. It creates a temporary format service with an empty database and
    /// uses it to format the provided file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to format
    ///
    /// # Returns
    ///
    /// - `Ok(FileFormatStatus::Unchanged)` if the file is already properly formatted
    /// - `Ok(FileFormatStatus::Changed(String))` if the file was formatted, containing the new content
    /// - `Ok(FileFormatStatus::FailedToParse(ParseError))` if the file couldn't be parsed
    /// - `Err(OrchestratorError)` if formatting failed for other reasons
    ///
    /// # Performance
    ///
    /// This method allocates a new bump arena for each call. For formatting multiple files,
    /// consider using [`get_format_service`](Self::get_format_service) and calling the
    /// service's methods with a reused arena.
    pub fn format_file(&self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        let service = self.get_format_service(ReadDatabase::empty());

        service.format_file(file)
    }

    /// Formats a single file using a provided bump arena for allocations.
    ///
    /// This method is similar to [`format_file`](Self::format_file) but allows you to
    /// provide your own bump arena for memory allocations. This is more efficient when
    /// formatting multiple files sequentially, as you can reuse and reset the same arena.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to format
    /// * `arena` - A bump allocator for temporary allocations during formatting
    ///
    /// # Returns
    ///
    /// - `Ok(FileFormatStatus::Unchanged)` if the file is already properly formatted
    /// - `Ok(FileFormatStatus::Changed(String))` if the file was formatted, containing the new content
    /// - `Ok(FileFormatStatus::FailedToParse(ParseError))` if the file couldn't be parsed
    /// - `Err(OrchestratorError)` if formatting failed for other reasons
    ///
    /// # Performance
    ///
    /// Using this method with a reused arena (resetting it between calls) is significantly
    /// more efficient than calling [`format_file`](Self::format_file) repeatedly, as it
    /// avoids repeated allocator initialization.
    pub fn format_file_in(&self, file: &File, arena: &Bump) -> Result<FileFormatStatus, OrchestratorError> {
        let service = self.get_format_service(ReadDatabase::empty());

        service.format_file_in(file, arena)
    }
}
