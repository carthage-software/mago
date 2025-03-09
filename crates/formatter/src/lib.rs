use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_parser::error::ParseError;
use mago_parser::parse_source;
use mago_php_version::PHPVersion;
use mago_source::Source;

use crate::document::Document;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::printer::Printer;
use crate::settings::FormatSettings;

pub mod document;
pub mod settings;

mod internal;

/// Formatter for PHP code.
///
/// The `Formatter` is the main entry point for formatting PHP code. It allows for:
///
/// - Building an AST representation of the code
/// - Converting that AST into a document model
/// - Printing the document as a formatted string
#[derive(Debug)]
pub struct Formatter<'a> {
    interner: &'a ThreadedInterner,
    php_version: PHPVersion,
    settings: FormatSettings,
}

impl<'a> Formatter<'a> {
    /// Creates a new `Formatter` instance.
    ///
    /// # Arguments
    ///
    /// * `interner` - The interner to use for string interning.
    /// * `php_version` - The PHP version to target when formatting.
    /// * `settings` - The settings to use for formatting.
    ///
    /// # Returns
    ///
    /// A new `Formatter` instance configured with the given parameters.
    pub fn new(interner: &'a ThreadedInterner, php_version: PHPVersion, settings: FormatSettings) -> Self {
        Self { interner, php_version, settings }
    }

    /// Formats PHP code provided as a string.
    ///
    /// This method parses the provided code string into an AST and then formats it.
    /// It's a convenient way to format code snippets without manually creating
    /// a Source object.
    ///
    /// # Arguments
    ///
    /// * `name` - A name for the code snippet (used for error reporting).
    /// * `code` - The PHP code to format as a string.
    ///
    /// # Returns
    ///
    /// A Result containing either the formatted code as a string or a ParseError
    /// if the code couldn't be parsed.
    pub fn format_code(&self, name: &'a str, code: &'a str) -> Result<String, ParseError> {
        let source = Source::standalone(self.interner, name, code);

        self.format_source(&source)
    }

    /// Formats PHP code from a Source object.
    ///
    /// This method parses the provided Source into an AST and then formats it.
    /// This is useful when you already have a Source object but not a parsed AST.
    ///
    /// # Arguments
    ///
    /// * `source` - The Source object containing the PHP code to format.
    ///
    /// # Returns
    ///
    /// A Result containing either the formatted code as a string or a ParseError
    /// if the code couldn't be parsed.
    pub fn format_source(&self, source: &'a Source) -> Result<String, ParseError> {
        let (program, error) = parse_source(self.interner, source);
        if let Some(error) = error {
            return Err(error);
        }

        Ok(self.format(source, &program))
    }

    /// Formats a PHP program.
    ///
    /// This is a convenience method that combines `build` and `print` into a single operation.
    ///
    /// # Arguments
    ///
    /// * `source` - The source to use for the program.
    /// * `program` - A `Program` struct representing the AST of the program to format.
    ///
    /// # Returns
    ///
    /// The formatted program as a string.
    pub fn format(&self, source: &'a Source, program: &'a Program) -> String {
        let document = self.build(source, program);

        self.print(document, Some(source.size))
    }

    /// Builds a document model from a program AST.
    ///
    /// This method converts the AST into a document model that represents
    /// the logical structure of the formatted code.
    ///
    /// # Arguments
    ///
    /// * `source` - The source to use for the program.
    /// * `program` - A `Program` struct representing the AST of the program to build.
    ///
    /// # Returns
    ///
    /// A `Document` representing the structured format of the program.
    pub fn build(&self, source: &'a Source, program: &'a Program) -> Document<'a> {
        program.format(&mut FormatterState::new(self.interner, source, self.php_version, self.settings))
    }

    /// Prints a document model as a formatted string.
    ///
    /// This method takes a document model and renders it as formatted text
    /// according to the formatter settings.
    ///
    /// # Arguments
    ///
    /// * `document` - The document model to print.
    /// * `capacity_hint` - An optional hint for pre-allocating the output buffer size.
    ///   When available (e.g., from source code), this improves performance.
    ///
    /// # Returns
    ///
    /// The formatted document as a string.
    pub fn print(&self, document: Document<'a>, capacity_hint: Option<usize>) -> String {
        Printer::new(document, capacity_hint.unwrap_or(0), self.settings).build()
    }
}
