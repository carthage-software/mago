use std::process::ExitCode;

use clap::Parser;
use mago_names::Names;
use serde_json::json;
use termtree::Tree;

use mago_ast::node::NodeKind;
use mago_ast::Node;
use mago_interner::ThreadedInterner;
use mago_parser::parse_source;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_reporting::Issue;
use mago_source::SourceManager;

use crate::enum_variants;
use crate::error::Error;

/// Represents the `ast` command, which parses a PHP file and prints its abstract syntax tree (AST).
#[derive(Parser, Debug)]
#[command(
    name = "ast",
    about = "Parses a PHP file and prints its abstract syntax tree (AST).",
    long_about = "The `ast` command takes a PHP file as input, parses it, and outputs the abstract syntax tree (AST). The output can be displayed in a tree format or JSON format based on user preference."
)]
pub struct AstCommand {
    /// Path to the PHP file to be parsed.
    #[arg(long, short = 'f', help = "The PHP file to parse.", required = true)]
    pub file: String,

    #[arg(long, help = "Includes names in the output.")]
    pub include_names: bool,

    /// Outputs the AST in JSON format if specified.
    #[arg(long, help = "Outputs the result in JSON format.")]
    pub json: bool,

    /// Specifies the issue reporting target to use.
    #[arg(long, default_value_t, help = "The issue reporting target to use.", ignore_case = true, value_parser = enum_variants!(ReportingTarget))]
    pub reporting_target: ReportingTarget,

    /// Specifies the issue reporting format to use.
    #[arg(long, default_value_t, help = "The issue reporting format to use.", ignore_case = true, value_parser = enum_variants!(ReportingFormat))]
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
        mago_feedback::error!("File '{}' does not exist.", command.file);

        return Ok(ExitCode::FAILURE);
    }

    if !file_path.is_file() {
        mago_feedback::error!("The path '{}' is not a file.", command.file);

        return Ok(ExitCode::FAILURE);
    }

    // Initialize interner and source manager.
    let interner = ThreadedInterner::new();
    let source_manager = SourceManager::new(interner.clone());

    // Load the source file.
    let source_id = source_manager.insert_path(command.file.clone(), file_path, true);
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
            let names = Names::resolve(&interner, &ast);

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
