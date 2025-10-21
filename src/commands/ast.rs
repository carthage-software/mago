//! AST inspection command implementation.
//!
//! This module implements the `mago ast` command, which provides detailed inspection
//! of how Mago parses PHP code. This is a diagnostic and educational tool that reveals
//! the internal structure of PHP code at various levels.
//!
//! # Inspection Modes
//!
//! The command supports three different inspection modes:
//!
//! - **AST Mode** (default): Displays the complete Abstract Syntax Tree
//! - **Token Mode** (`--tokens`): Shows the lexer's token stream
//! - **Names Mode** (`--names`): Displays resolved symbol names
//!
//! # Output Formats
//!
//! Each mode supports multiple output formats:
//!
//! - **Tree Format** (default): Human-readable tree structure with colors
//! - **JSON Format** (`--json`): Machine-parseable structured output
//! - **Debug Format** (`--debug`): Rust debug representation
//!
//! # Use Cases
//!
//! - **Parser Debugging**: Verify how code is being parsed
//! - **Learning**: Understand PHP's syntactic structure
//! - **Tool Development**: Build tools that work with Mago's AST
//! - **Issue Reporting**: Provide detailed context for parser bugs
//!
//! # AST Structure
//!
//! The AST (Abstract Syntax Tree) represents the hierarchical structure of PHP code
//! after parsing. It includes:
//!
//! - Program structure (namespaces, classes, functions)
//! - Statements (if, while, foreach, etc.)
//! - Expressions (calls, operations, literals)
//! - Type hints and declarations
//!
//! # Name Resolution
//!
//! The `--names` mode shows how identifiers are resolved to fully-qualified names,
//! taking into account use statements and namespace context.

use std::path::PathBuf;
use std::process::ExitCode;

use bumpalo::Bump;
use clap::ColorChoice;
use clap::Parser;
use colored::Colorize;
use serde_json::json;
use termtree::Tree;
use unicode_width::UnicodeWidthStr;

use mago_database::Database;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_syntax::ast::*;
use mago_syntax::error::ParseError;
use mago_syntax::lexer::Lexer;
use mago_syntax::parser::parse_file;
use mago_syntax_core::input::Input;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Maximum width for value columns in tree output.
const VALUE_COLUMN_WIDTH: usize = 50;

/// Command for inspecting the structure of PHP code.
///
/// This command provides multiple views into how Mago parses and processes PHP code,
/// from the token level up to the complete Abstract Syntax Tree. It's essential for
/// debugging parser issues and understanding code structure.
///
/// This command can tokenize a file, parse it into an Abstract Syntax Tree (AST),
/// and display the results in various formats. It's an essential utility for
/// debugging the parser, understanding code structure, or for integration with other tools.
#[derive(Parser, Debug)]
#[command(
    name = "ast",
    about = "Inspect the lexical and syntactical structure of a PHP file.",
    long_about = "Analyze and display the internal structure of PHP code.\n\n\
                  This command helps you understand how Mago parses your PHP code by showing:\n\
                  - The Abstract Syntax Tree (AST) structure (default)\n\
                  - Token stream from the lexer (--tokens)\n\
                  - Resolved symbol names (--names)\n\n\
                  Use this for debugging parsing issues, understanding code structure,\n\
                  or integrating with other development tools."
)]
pub struct AstCommand {
    /// The PHP file to analyze and display.
    ///
    /// This should be a valid PHP file that you want to inspect.
    /// The file will be parsed and its structure displayed according to the selected options.
    #[arg(required = true)]
    pub file: PathBuf,

    /// Show the token stream instead of the AST.
    ///
    /// This displays the sequence of tokens (keywords, operators, identifiers, etc.)
    /// that the lexer generates from your PHP code. Useful for understanding
    /// how the code is broken down into basic elements.
    #[arg(long)]
    pub tokens: bool,

    /// Output in machine-readable JSON format.
    ///
    /// Instead of the human-readable tree format, output structured JSON
    /// that can be processed by other tools or scripts.
    #[arg(long)]
    pub json: bool,

    /// Show resolved symbol names and their scope information.
    ///
    /// This displays how Mago resolves symbol names (classes, functions, constants)
    /// within their proper namespaces and scope contexts. Cannot be used with --tokens.
    #[arg(long, conflicts_with = "tokens")]
    pub names: bool,

    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

impl AstCommand {
    /// Executes the AST inspection command.
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let arena = Bump::new();
        let file = File::read(&configuration.source.workspace, &self.file, FileType::Host)?;

        if self.tokens {
            return self.print_tokens(configuration, color_choice, &arena, file);
        }

        let (program, error) = parse_file(&arena, &file);

        if self.json {
            print_ast_json(program, error.as_ref())?;
        } else if self.names {
            print_names(&arena, program)?;
        } else {
            print_ast_tree(program);
        }

        if let Some(error) = error {
            let issues = IssueCollection::from([Into::<Issue>::into(&error)]);
            let database = Database::single(file);

            return self
                .reporting
                .get_processor(create_orchestrator(&configuration, color_choice, false), database, color_choice)
                .process_issues(issues, None, false);
        }

        Ok(ExitCode::SUCCESS)
    }

    /// Prints the list of tokens from a file, either as a table or as JSON.
    fn print_tokens(
        self,
        configuration: Configuration,
        color_choice: ColorChoice,
        arena: &Bump,
        file: File,
    ) -> Result<ExitCode, Error> {
        let mut lexer = Lexer::new(arena, Input::from_file(&file));
        let mut tokens = Vec::new();
        loop {
            match lexer.advance() {
                Some(Ok(token)) => tokens.push(token),
                Some(Err(err)) => {
                    let issue = Into::<Issue>::into(&err);
                    let database = Database::single(file);

                    return self
                        .reporting
                        .get_processor(create_orchestrator(&configuration, color_choice, false), database, color_choice)
                        .process_issues(IssueCollection::from([issue]), None, false);
                }
                None => break,
            }
        }

        if self.json {
            println!("{}", serde_json::to_string_pretty(&tokens)?);
        } else {
            println!();
            println!("  {}", "Tokens".bold().underline());
            println!();
            println!("  {: <25} {: <50} {}", "Kind".bold(), "Value".bold(), "Span".bold());
            println!("  {0:─<25} {0:─<50} {0:─<20}", "");
            for token in tokens {
                let end_byte_index = token.value.char_indices().nth(48).map_or(token.value.len(), |(i, _)| i);
                let value_for_display = format!("{:?}", &token.value[..end_byte_index]);
                let visual_width = UnicodeWidthStr::width(value_for_display.as_str());
                let padding_needed = VALUE_COLUMN_WIDTH.saturating_sub(visual_width);

                let value_str = format!("{}{}", value_for_display, " ".repeat(padding_needed)).bright_black();
                let kind_str = format!("{:?}", token.kind).cyan();

                println!(
                    "  {: <25} {} {}",
                    kind_str,
                    value_str,
                    format!("[{}..{}]", token.span.start, token.span.end).dimmed()
                );
            }
            println!();
        }

        Ok(ExitCode::SUCCESS)
    }
}

/// Prints the AST as a rich, human-readable tree.
fn print_ast_tree(program: &Program) {
    let tree = node_to_tree(Node::Program(program));
    println!();
    println!("{}", tree);
    println!();
}

/// Prints the AST in a machine-readable, pretty-printed JSON format.
fn print_ast_json(program: &Program, error: Option<&ParseError>) -> Result<(), Error> {
    let result = json!({
        "program": program,
        "error": error.map(Into::<Issue>::into),
    });

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

/// Prints the list of resolved symbol names from the AST.
fn print_names<'arena>(arena: &'arena Bump, program: &Program<'arena>) -> Result<(), Error> {
    let resolver = NameResolver::new(arena);
    let names = resolver.resolve(program);

    println!();
    println!("  {}", "Resolved Names".bold().underline());
    println!();
    println!("  {: <10} {: <50} {}", "Offset".bold(), "Name".bold(), "Imported".bold());
    println!("  {0:─<10} {0:─<50} {0:─<10}", "");

    for (position, (name, is_imported)) in names.all() {
        let imported_str = if *is_imported { "✅".green() } else { "❌".red() };
        println!("  {: <10} {: <50} {}", format!("@{}", position).dimmed(), name.cyan(), imported_str);
    }
    println!();
    Ok(())
}

/// Recursively converts an AST `Node` into a rich `termtree::Tree`.
fn node_to_tree(node: Node) -> Tree<String> {
    let label = match node {
        // Semicolons!
        Node::Statement(Statement::Noop(_)) => {
            format!("{} {}", "Statement".bold().underline(), ";".red().bold())
        }
        Node::Terminator(Terminator::Semicolon(_)) => {
            format!("{} {}", "Terminator".dimmed(), ";".red().bold())
        }
        // Structural nodes
        Node::Program(_) => "Program".bold().underline().to_string(),
        Node::Statement(_) => "Statement".bold().underline().to_string(),
        Node::Expression(_) => "Expression".bold().underline().to_string(),
        // Literals
        Node::LiteralString(s) => {
            format!("{} {}", "LiteralString".green(), format!("{:?}", s.value.unwrap_or("")).yellow())
        }
        Node::LiteralInteger(i) => {
            format!("{} {}", "LiteralInteger".green(), i.value.map_or("?".to_string(), |v| v.to_string()).yellow())
        }
        Node::LiteralFloat(f) => format!("{} {}", "LiteralFloat".green(), f.value.to_string().yellow()),
        // Identifiers
        Node::LocalIdentifier(id) => format!("{} {}", "LocalIdentifier".cyan(), id.value.bright_black()),
        Node::QualifiedIdentifier(id) => format!("{} {}", "QualifiedIdentifier".cyan(), id.value.bright_black()),
        Node::FullyQualifiedIdentifier(id) => {
            format!("{} {}", "FullyQualifiedIdentifier".cyan(), id.value.bright_black())
        }
        // Variables
        Node::DirectVariable(var) => format!("{} {}", "DirectVariable".cyan(), var.name.yellow()),
        // Operators
        Node::BinaryOperator(op) => format!("{} {}", "BinaryOperator".magenta(), op.as_str().bold()),
        Node::UnaryPrefixOperator(op) => format!("{} {}", "UnaryPrefixOperator".magenta(), op.as_str().bold()),
        Node::UnaryPostfixOperator(op) => format!("{} {}", "UnaryPostfixOperator".magenta(), op.as_str().bold()),
        Node::AssignmentOperator(op) => format!("{} {}", "AssignmentOperator".magenta(), op.as_str().bold()),
        // Everything else -> Dimmed
        _ => format!("{}", node.kind().to_string().dimmed()),
    };

    let mut tree = Tree::new(label);
    for child in node.children() {
        tree.push(node_to_tree(child));
    }

    tree
}
