//! Provides a configurable, high-performance formatter for PHP code.
//!
//! This crate defines the main [`Formatter`] entry point, which orchestrates the process
//! of parsing source code, converting it into an intermediate document model, and
//! printing it as a well-formatted string according to customizable settings.

use std::borrow::Cow;

use bumpalo::Bump;
use mago_database::file::File;
use mago_php_version::PHPVersion;
use mago_syntax::ast::Program;
use mago_syntax::error::ParseError;
use mago_syntax::parser::parse_file;

use crate::document::Document;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::printer::Printer;
use crate::settings::FormatSettings;

pub mod document;
pub mod settings;

mod internal;

/// The main entry point for formatting PHP code.
///
/// The `Formatter` orchestrates the entire formatting process, from parsing
/// the source code into an Abstract Syntax Tree (AST) to printing a well-formatted
/// string representation. It is configured with a specific PHP version, formatting
/// settings, and a string interner.
#[derive(Debug)]
pub struct Formatter<'arena> {
    arena: &'arena Bump,
    php_version: PHPVersion,
    settings: FormatSettings,
}

impl<'arena> Formatter<'arena> {
    /// Creates a new `Formatter` with the specified configuration.
    pub fn new(arena: &'arena Bump, php_version: PHPVersion, settings: FormatSettings) -> Self {
        Self { arena, php_version, settings }
    }

    /// Formats a string of PHP code.
    ///
    /// This is a high-level convenience method that handles the creation of an ephemeral
    /// [`File`] internally. It is ideal for formatting code snippets or sources that
    /// do not exist on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the input code contains syntax errors.
    pub fn format_code(&self, name: Cow<'static, str>, code: Cow<'static, str>) -> Result<&'arena str, ParseError> {
        let file = File::ephemeral(name, code);

        self.format_file(&file)
    }

    /// Formats the contents of a [`File`].
    ///
    /// This method will first parse the file's content into an AST and then format it.
    /// It should be used when you already have a `File` instance, for example, from
    /// a `mago_database::Database`.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the file's content contains syntax errors.
    pub fn format_file<'ctx>(&self, file: &'ctx File) -> Result<&'arena str, ParseError> {
        let (program, error) = parse_file(self.arena, file);
        if let Some(error) = error {
            return Err(error);
        }

        Ok(self.format(file, program))
    }

    /// Formats a pre-parsed [`Program`] (AST).
    ///
    /// This is the lowest-level formatting method that operates directly on the AST.
    /// It first builds an intermediate [`Document`] representation and then prints it.
    /// This is useful if you have already parsed the code and want to avoid re-parsing.
    pub fn format<'ctx>(&self, file: &'ctx File, program: &'arena Program<'arena>) -> &'arena str {
        let document = self.build(file, program);

        self.print(document, Some(file.size as usize))
    }

    /// Converts a program's AST into a structured [`Document`] model.
    ///
    /// The document model is an intermediate representation that describes the
    /// layout of the code with elements like groups, indentation, and line breaks.
    /// This is a separate step from printing, allowing for potential inspection or
    /// manipulation of the layout before rendering.
    pub fn build(&self, file: &File, program: &'arena Program<'arena>) -> Document<'arena> {
        program.format(&mut FormatterState::new(self.arena, program, file, self.php_version, self.settings))
    }

    /// Renders a [`Document`] model into a formatted string.
    ///
    /// The printer traverses the document model and generates the final text output
    /// according to the configured format settings.
    ///
    /// # Arguments
    ///
    /// * `document` - The document model to print.
    /// * `capacity_hint` - An optional hint for pre-allocating the output string's
    ///   capacity, which can improve performance for large documents.
    ///
    /// # Returns
    ///
    /// A formatted string representation of the document.
    pub fn print(&self, document: Document<'arena>, capacity_hint: Option<usize>) -> &'arena str {
        Printer::new(self.arena, document, capacity_hint.unwrap_or(0), self.settings).build()
    }
}
