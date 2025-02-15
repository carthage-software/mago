use serde::Deserialize;
use serde::Serialize;

use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_parser::error::ParseError;
use mago_php_version::PHPVersion;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_source::SourceCategory;

use crate::internal;

/// `Module` represents a processed PHP source code module.
///
/// It encapsulates the original source, resolved names, any parsing errors,
/// optional reflection data (if enabled), and a collection of semantic issues.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub source: Source,
    pub names: Names,
    pub parse_error: Option<ParseError>,
    pub reflection: Option<CodebaseReflection>,
    pub issues: IssueCollection,
}

/// `ModuleBuildOptions` configures the behavior of the module building process.
///
/// The options determine whether the module should perform reflection, validation, or both:
/// - `reflect`: When true, reflection is performed.
/// - `validate`: When true, semantic validation is performed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModuleBuildOptions {
    pub reflect: bool,
    pub validate: bool,
}

impl Module {
    /// Builds a `Module` from a given PHP source.
    ///
    /// This convenience function parses the PHP source code to generate an abstract syntax tree (AST),
    /// resolves names, optionally performs reflection and/or semantic validation based on the provided
    /// build options, and collects any issues encountered during analysis.
    ///
    /// Internally, it delegates to [`build_with_ast`] and discards the AST.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to a `ThreadedInterner` used for efficient string interning.
    /// * `version` - The PHP version guiding the parsing and analysis.
    /// * `source` - The PHP source code encapsulated in a `Source` struct.
    /// * `options` - The build options controlling reflection and validation.
    ///
    /// # Returns
    ///
    /// A new `Module` instance containing the resolved names, any parse errors, optional reflection data,
    /// and a collection of issues.
    #[inline(always)]
    pub fn build(
        interner: &ThreadedInterner,
        version: PHPVersion,
        source: Source,
        options: ModuleBuildOptions,
    ) -> Self {
        let (module, _) = Self::build_with_ast(interner, version, source, options);

        module
    }

    /// Builds a `Module` from a given PHP source and returns the associated AST.
    ///
    /// This function performs the complete module processing workflow:
    /// - Parses the PHP source to generate an AST (`Program`).
    /// - Resolves symbol names.
    /// - Optionally performs reflection and/or semantic validation based on the provided build options.
    /// - Collects any issues encountered during analysis.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to a `ThreadedInterner` for efficient string interning.
    /// * `version` - The PHP version used to guide parsing and analysis.
    /// * `source` - The PHP source code encapsulated in a `Source` struct.
    /// * `options` - The build options that control reflection and validation.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A `Module` instance with the processed data (names, reflection, issues, etc.).
    /// - The `Program` representing the parsed abstract syntax tree (AST) of the source.
    ///
    /// This is useful when the AST is required immediately before moving the module
    /// to another context (e.g., across threads) so that the cost of re-parsing can be avoided.
    #[inline(always)]
    pub fn build_with_ast(
        interner: &ThreadedInterner,
        version: PHPVersion,
        source: Source,
        options: ModuleBuildOptions,
    ) -> (Self, Program) {
        let (program, parse_error) = mago_parser::parse_source(interner, &source);
        let names = Names::resolve(interner, &program);
        let (reflection, issues) = internal::build(interner, version, &source, &program, &names, options);
        let module = Self { source, parse_error, names, reflection, issues };

        (module, program)
    }

    /// Parses the module's source code to generate an abstract syntax tree (AST).
    ///
    /// For performance reasons, the AST is not stored within the module. If an AST is needed later,
    /// this method re-parses the source code on demand.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to a `ThreadedInterner` used for efficient string handling during parsing.
    ///
    /// # Returns
    ///
    /// A `Program` representing the abstract syntax tree (AST) of the module.
    pub fn parse(&self, interner: &ThreadedInterner) -> Program {
        mago_parser::parse_source(interner, &self.source).0
    }

    /// Retrieves the category of the module's source.
    ///
    /// This method extracts the source category (e.g., user-defined, built-in, external)
    /// from the module's source identifier.
    #[inline]
    pub const fn category(&self) -> SourceCategory {
        self.source.identifier.category()
    }
}

impl ModuleBuildOptions {
    /// Creates a new `ModuleBuildOptions` with the specified settings.
    #[inline(always)]
    pub const fn new(reflect: bool, validate: bool) -> Self {
        Self { reflect, validate }
    }

    /// Returns build options configured for reflection only (without validation).
    #[inline(always)]
    pub const fn reflection() -> Self {
        Self { reflect: true, validate: false }
    }

    /// Returns build options configured for validation only (without reflection).
    #[inline(always)]
    pub const fn validation() -> Self {
        Self { reflect: false, validate: true }
    }
}

impl Default for ModuleBuildOptions {
    /// Returns the default build options with both reflection and validation enabled.
    fn default() -> Self {
        Self::new(true, true)
    }
}
