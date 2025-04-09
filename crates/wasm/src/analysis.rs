//! # Analysis Module
//!
//! The `analysis` module contains the primary data structures and logic for
//! performing a **combined parse, semantic analysis, lint, and (optional)**
//! formatting pass on PHP code via Mago. It provides a high-level interface
//! (see [`AnalysisResults::analyze`]) that accepts code plus configuration,
//! and returns detailed information about the analyzed code, including parse
//! and semantic issues, linter results, and a formatted output.

use std::collections::HashSet;

use serde::Serialize;

use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_linter::Linter;
use mago_linter::settings::Settings;
use mago_project::module::Module;
use mago_project::module::ModuleBuildOptions;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_syntax::ast::Program;

/// Represents the result of analyzing and formatting PHP code.
///
/// This struct encapsulates various aspects of the PHP code analysis process,
/// providing detailed insights into the source code, including:
///
/// - Interned strings used in the code.
/// - The resulting AST ([`Program`]) after parsing.
/// - An optional parse error if the code was invalid.
/// - Resolved name information (e.g., whether a name was imported).
/// - A collection of semantic issues found during analysis.
/// - A collection of linter issues found during linting (according to [`Settings`]).
/// - Optionally, a `formatted` version of the code if no parse error was encountered.
///
/// **Note:** This struct is serialized into JSON (via [`serde`]) for use in
/// WebAssembly and browser environments. For direct Rust usage, it can be
/// used as-is in a native context.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResults {
    /// A set of interned strings used in the source code.
    ///
    /// Each string is represented as a tuple containing a [`StringIdentifier`]
    /// and the actual string value.
    pub strings: HashSet<(StringIdentifier, String)>,

    /// The abstract syntax tree (AST) resulting from parsing the source code.
    ///
    /// If [`parse_error`](Self::parse_error) is `Some`, this AST may be incomplete or invalid.
    pub program: Program,

    /// An optional parse error, if one occurred during parsing.
    ///
    /// If this is `Some`, then [`formatted`](Self::formatted) will be `None`,
    /// because the code could not be validly parsed.
    pub parse_error: Option<Issue>,

    /// The resolved names within the source code.
    ///
    /// Each entry is a tuple `(byte_offset, (identifier, imported))`, where:
    /// - `byte_offset` indicates where in the source the name is used,
    /// - `identifier` is the [`StringIdentifier`] for the interned name,
    /// - `imported` indicates whether this name was imported or locally declared.
    pub names: HashSet<(usize, (StringIdentifier, bool))>,

    /// The formatted version of the source code, if there were no parse errors.
    ///
    /// This is produced by Mago’s internal formatter and is only set if
    /// [`parse_error`](Self::parse_error) is `None`.
    pub formatted: Option<String>,

    /// A collection of semantic issues found during semantic analysis.
    ///
    /// These might include invalid modifier usage, incorrect return types,
    /// or undefined variable references, among others.
    pub semantic_issues: IssueCollection,

    /// A collection of linter issues discovered based on the specified
    /// [`Settings`](mago_linter::settings::Settings).
    ///
    /// Includes warnings or notices about potential coding standard violations,
    /// unused variables, or other recommended improvements.
    pub linter_issues: IssueCollection,
}

impl AnalysisResults {
    /// Analyzes and (optionally) formats the provided PHP `code`.
    ///
    /// This function performs the following steps:
    /// 1. **Parsing** via [`Module::build`] to generate an AST and detect syntax errors.
    /// 2. **Semantic Analysis** to resolve names and check for issues (e.g. undefined variables).
    /// 3. **Linting** according to the specified [`Settings`](mago_linter::settings::Settings).
    /// 4. **Formatting** using [`FormatSettings`](mago_formatter::settings::FormatSettings),
    ///    provided no parse errors were found.
    ///
    /// # Arguments
    ///
    /// * `code` - A `String` containing the PHP code to be analyzed.
    /// * `linter_settings` - Configuration for which linter plugins and rules should run.
    /// * `format_settings` - Configuration for how the code should be formatted.
    ///
    /// # Returns
    ///
    /// Returns an [`AnalysisResults`] containing the AST, parse/semantic/linter issues,
    /// and formatted code (if no parse error).
    pub fn analyze(code: String, lint_settings: Settings, format_settings: FormatSettings) -> Self {
        let interner = ThreadedInterner::new();
        let source = Source::standalone(&interner, "code.php", &code);
        let (mut module, program) =
            Module::build_with_ast(&interner, lint_settings.php_version, source, ModuleBuildOptions::validation());
        let mut formatted = None;
        if module.parse_error.is_none() {
            let formatter = Formatter::new(&interner, lint_settings.php_version, format_settings);

            // Only format if there are no parse errors
            formatted = Some(formatter.format(&module.source, &program));
        }

        let linter =
            Linter::with_all_plugins(lint_settings, interner.clone(), module.reflection.take().unwrap_or_default());
        let linter_issues = linter.lint(&module);

        Self {
            strings: interner.all().into_iter().map(|(id, value)| (id, value.to_string())).collect(),
            program,
            parse_error: module.parse_error.as_ref().map(|e| e.into()),
            names: module.names.all().into_iter().map(|(offset, (id, imported))| (*offset, (*id, *imported))).collect(),
            formatted,
            semantic_issues: module.issues,
            linter_issues,
        }
    }
}
