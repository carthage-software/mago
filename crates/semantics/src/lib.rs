//! # Mago Semantics Crate
//!
//! The `mago_semantics` crate provides semantic analysis capabilities for PHP code.
//! **Mago** is the name of the toolchain, and this crate processes PHP source code to generate
//! an abstract syntax tree (AST), performs name resolution, builds a symbol table, and detects
//! semantic issues.
//!
//! This crate is essential for the compilation pipeline of PHP within the Mago toolchain,
//! ensuring that source code adheres to the language's semantic rules before code generation
//! or interpretation.
//!
//! # Features
//!
//! - **Parsing**: Converts PHP source code into an AST.
//! - **Name Resolution**: Associates identifiers with their declarations.
//! - **Symbol Table Construction**: Records all symbols (classes, functions, variables) for quick lookup.
//! - **Semantic Analysis**: Checks for semantic correctness and reports issues.

use serde::Deserialize;
use serde::Serialize;

use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_parser::error::ParseError;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_walker::Walker;

use crate::context::Context;
use crate::walker::SemanticsWalker;

mod consts;
mod context;
mod walker;

/// The `Semantics` struct encapsulates all the information obtained after performing semantic analysis
/// on a PHP source code file. It includes the original source code, the parsed abstract syntax tree (AST),
/// any parse errors encountered, resolved names, the symbol table, and a collection of semantic issues.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Semantics {
    /// The original PHP source code, including its name and content.
    pub source: Source,

    /// The abstract syntax tree (AST) resulting from parsing the source code.
    pub program: Program,

    /// An optional parse error, if one occurred during parsing.
    pub parse_error: Option<ParseError>,

    /// The resolved names within the source code, used for identifier resolution.
    pub names: Names,

    /// A collection of semantic issues found during analysis, such as invalid inheritance,
    ///  improper returns, duplicate names, etc.
    pub issues: IssueCollection,
}

impl Semantics {
    /// Builds the `Semantics` object by performing parsing, name resolution, symbol table construction,
    /// and semantic analysis on the provided PHP source code.
    ///
    /// # Parameters
    ///
    /// - `interner`: A reference to a `ThreadedInterner` used for string interning, which helps in
    ///               efficiently handling string comparisons and memory usage.
    /// - `source`: The `Source` object representing the PHP source code to be analyzed.
    ///
    /// # Returns
    ///
    /// A `Semantics` object containing the results of the semantic analysis, including the AST,
    /// any parse errors, resolved names, the symbol table, and any semantic issues found.
    ///
    /// # Steps
    ///
    /// 1. **Parsing**: The source code is parsed into an abstract syntax tree (AST).
    ///    If there are syntax errors, they are captured in `parse_error`.
    /// 2. **Name Resolution**: Resolves all the names in the AST, linking identifiers to their declarations.
    /// 3. **Symbol Table Construction**: Builds a symbol table containing all the symbols (classes, functions, constants, etc.) defined in the source code.
    /// 4. **Semantic Analysis**: Checks the AST for semantic correctness, such as type checking, scope rules, etc., and collects any issues.
    pub fn build(interner: &ThreadedInterner, source: Source) -> Self {
        // Parse the source code into an AST.
        // The parser returns a tuple containing the AST and an optional parse error.
        let (program, parse_error) = mago_parser::parse_source(interner, &source);

        // Resolve names in the AST.
        // This step links identifiers to their declarations, handling scopes and imports.
        let names = Names::resolve(interner, &program);

        // Perform semantic analysis and collect issues.
        // This includes checks for type correctness, proper usage of constructs, etc.
        let mut context = Context::new(interner, &program, &names);
        SemanticsWalker.walk_program(&program, &mut context);
        let issues = context.take_issue_collection();

        // Return the Semantics object containing all analysis results.
        Self { source, program, parse_error, names, issues }
    }

    /// Determines whether the semantic analysis was successful,
    /// i.e., no parse errors or semantic issues were found.
    pub fn is_valid(&self) -> bool {
        self.parse_error.is_none() && self.issues.is_empty()
    }

    /// Determines whether the source code contains any parse errors.
    pub fn has_parse_error(&self) -> bool {
        self.parse_error.is_some()
    }

    /// Determines whether the source code contains any semantic issues.
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Checks if the source code is external, i.e., not a user-defined file.
    pub fn is_external(&self) -> bool {
        self.source.identifier.is_external()
    }
}
