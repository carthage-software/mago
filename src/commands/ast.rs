use std::process::ExitCode;

use clap::Parser;
use mago_names::resolver::NameResolver;
use serde_json::json;
use termtree::Tree;

use mago_interner::ThreadedInterner;
use mago_reporting::Issue;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_source::SourceCategory;
use mago_source::SourceManager;
use mago_syntax::ast::Node;
use mago_syntax::ast::node::NodeKind;
use mago_syntax::parser::parse_source;

use crate::enum_variants;
use crate::error::Error;

/// Represents the `ast` command, which parses a PHP file and prints its abstract syntax tree (AST).
#[derive(Parser, Debug)]
#[command(
    name = "ast",
    about = "Parse and visualize the abstract syntax tree (AST) of a PHP file",
    long_about = r#"
The `ast` command parses a PHP file and outputs its abstract syntax tree (AST).

This command helps you understand the structure of your PHP code and debug parsing issues.
"#
)]
pub struct AstCommand {
    /// Path to the PHP file to be parsed.
    #[arg(long, short = 'f', help = "Specify the PHP file to parse", required = true)]
    pub file: String,

    /// Include resolved names in the output.
    #[arg(long, help = "Include resolved names in the output to show symbol resolution")]
    pub include_names: bool,

    /// Output the AST in JSON format for integration with other tools.
    #[arg(long, help = "Output the AST in JSON format")]
    pub json: bool,

    /// Specify where the results should be reported.
    #[arg(
        long,
        default_value_t,
        help = "Specify where the results should be reported",
        ignore_case = true,
        value_parser = enum_variants!(ReportingTarget)
    )]
    pub reporting_target: ReportingTarget,

    /// Choose the format for reporting issues.
    #[arg(
        long,
        default_value_t,
        help = "Choose the format for reporting issues",
        ignore_case = true,
        value_parser = enum_variants!(ReportingFormat)
    )]
    pub reporting_format: ReportingFormat,
}

/// Executes the AST command with the provided options.
///
/// # Arguments
///
/// * `command` - The `AstCommand` structure containing user-specified options.
///
/// # Returns
///
/// An `ExitCode` indicating the success or failure of the command.
///
/// # Errors
///
/// An error is returned if the file does not exist or is not readable.
pub async fn execute(command: AstCommand) -> Result<ExitCode, Error> {
    let file_path = std::path::Path::new(&command.file).to_path_buf();

    // Verify if the file exists and is readable.
    if !file_path.exists() {
        tracing::error!("File '{}' does not exist.", command.file);

        return Ok(ExitCode::FAILURE);
    }

    if !file_path.is_file() {
        tracing::error!("The path '{}' is not a file.", command.file);

        return Ok(ExitCode::FAILURE);
    }

    // Initialize interner and source manager.
    let interner = ThreadedInterner::new();
    let source_manager = SourceManager::new(interner.clone());

    // Load the source file.
    let source_id = source_manager.insert_path(command.file.clone(), file_path, SourceCategory::UserDefined);
    let source = source_manager.load(&source_id)?;

    // Parse the source file into an AST.
    let (ast, error) = parse_source(&interner, &source);

    let has_error = error.is_some();
    if command.json {
        // Prepare and display JSON output.
        let result = json!({
            "interner": interner.all().into_iter().collect::<Vec<_>>(),
            "program": ast,
            "error": error.map(|e| Into::<Issue>::into(&e)),
        });

        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        // Display the AST as a tree.
        let tree = node_to_tree(Node::Program(&ast));

        println!("{tree}");

        if command.include_names {
            let resolver = NameResolver::new(&interner);
            let names = resolver.resolve(&ast);

            for (position, (value, is_imported)) in names.all() {
                let name = interner.lookup(value);

                println!("{}: {}{}", position, name, if *is_imported { " (imported)" } else { "" });
            }
        }

        // Report errors if any exist.
        if let Some(error) = &error {
            let issue = Into::<Issue>::into(error);

            Reporter::new(interner.clone(), source_manager, command.reporting_target)
                .report([issue], command.reporting_format)?;
        }
    }

    Ok(if has_error { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

/// Converts an AST node into a tree structure for visualization.
///
/// # Arguments
///
/// * `node` - The AST node to be converted into a tree.
///
/// # Returns
///
/// A `Tree` representation of the AST node and its children.
fn node_to_tree(node: Node<'_>) -> Tree<NodeKind> {
    let mut tree = Tree::new(node.kind());
    for child in node.children() {
        tree.push(node_to_tree(child));
    }

    tree
}
