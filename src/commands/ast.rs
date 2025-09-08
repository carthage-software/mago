use std::path::PathBuf;
use std::process::ExitCode;

use bumpalo::Bump;
use clap::Parser;
use colored::Colorize;
use mago_database::Database;
use mago_reporting::IssueCollection;
use serde_json::json;
use termtree::Tree;

use mago_database::file::File;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_syntax::ast::*;
use mago_syntax::error::ParseError;
use mago_syntax::lexer::Lexer;
use mago_syntax::parser::parse_file;
use mago_syntax_core::input::Input;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::error::Error;

/// A powerful tool for inspecting the lexical and syntactical structure of PHP code.
///
/// This command can tokenize a file, parse it into an Abstract Syntax Tree (AST),
/// and display the results in various formats. It's an essential utility for
/// debugging the parser, understanding code structure, or for integration with other tools.
#[derive(Parser, Debug)]
#[command(name = "ast", about = "Inspect the lexical and syntactical structure of a PHP file.", long_about)]
pub struct AstCommand {
    /// The PHP file to inspect.
    #[arg(required = true)]
    pub file: PathBuf,

    /// Display the stream of lexer tokens. Combine with --json for JSON output.
    #[arg(long, help = "Display the stream of lexer tokens instead of the AST")]
    pub tokens: bool,

    /// Display the output in a machine-readable JSON format.
    #[arg(long, help = "Display the output in a machine-readable JSON format")]
    pub json: bool,

    /// Display the list of resolved symbol names.
    #[arg(
        long,
        help = "Display the list of resolved symbol names",
        // Tokens and Names are fundamentally different views
        conflicts_with = "tokens"
    )]
    pub names: bool,

    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

impl AstCommand {
    /// Executes the AST inspection command.
    pub fn execute(self, configuration: Configuration, should_use_colors: bool) -> Result<ExitCode, Error> {
        let arena = Bump::new();
        let file = File::read(&configuration.source.workspace, &self.file, FileType::Host)?;

        if self.tokens {
            return self.print_tokens(configuration, should_use_colors, &arena, file);
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

            return self.reporting.process_issues(issues, configuration, should_use_colors, database);
        }

        Ok(ExitCode::SUCCESS)
    }

    /// Prints the list of tokens from a file, either as a table or as JSON.
    fn print_tokens(
        self,
        configuration: Configuration,
        should_use_colors: bool,
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

                    self.reporting.process_issues(
                        IssueCollection::from([issue]),
                        configuration,
                        should_use_colors,
                        database,
                    )?;

                    return Ok(ExitCode::FAILURE);
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
                let value_str = format!("{:?}", token.value).bright_black();
                let kind_str = format!("{:?}", token.kind).cyan();
                println!(
                    "  {: <25} {: <50} {}",
                    kind_str,
                    &value_str[..value_str.len().min(48)],
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
