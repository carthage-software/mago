use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::builder::get_type_from_string;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::union::populate_union_type;
use mago_docblock::document::Element;
use mago_docblock::document::TagKind;
use mago_docblock::tag::parse_var_tag;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Expression;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;

/// Populates the context with variable types defined in the docblock.
///
/// This function retrieves all `@var`, `@psalm-var`, and `@phpstan-var` tags from the docblock
/// of the current statement in the context, parses their variable types, and inserts them
/// into the current block context.
///
/// # Arguments
///
/// * `context`: The main analysis context, providing access to the docblock parser and error collector.
/// * `block_context`: The current block context, which holds local variables and their types.
/// * `artifacts`: The analysis artifacts, which may be used to store or retrieve additional information.
/// * `override_existing`: A boolean indicating whether to override existing variable types in the block context.
pub fn populate_docblock_variables<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    override_existing: bool,
) {
    for (name, variable_type, variable_type_span) in get_docblock_variables(context, block_context, artifacts, true) {
        let Some(variable_name) = name else {
            continue;
        };

        insert_variable_from_docblock(
            context,
            block_context,
            variable_name,
            variable_type,
            variable_type_span,
            override_existing,
        );
    }
}

/// Retrieves all `@var`, `@psalm-var`, and `@phpstan-var` tags from the docblock of the
/// current statement in the context, parsing their variable types.
///
/// This function scans the docblock associated with the current statement in the context,
/// extracting all variable type declarations. It returns a vector of tuples, each containing:
///
/// - An optional variable name (if specified in the tag)
/// - The parsed type as a `TUnion`
/// - The span of the tag in the source code.
///
/// # Arguments
///
/// * `context`: The main analysis context, providing access to the docblock parser and error collector.
/// * `block_context`: The current block context, which may influence the parsing of docblocks.
/// * `artifacts`: The analysis artifacts, which may be used to store or retrieve additional information.
///
/// # Returns
///
/// A vector of tuples, where each tuple contains:
///
/// - `Option<String>`: The variable name if specified, or `None` if the tag is unnamed.
/// - `TUnion`: The parsed type from the tag.
/// - `Span`: The span of the tag in the source code.
pub fn get_docblock_variables<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    allow_tracing: bool,
) -> Vec<(Option<String>, TUnion, Span)> {
    let Some(elements) = context.get_parsed_docblock().map(|document| document.elements) else {
        return vec![];
    };

    elements
        .into_iter()
        // Filter out non-tag elements
        .filter_map(|element| match element {
            Element::Tag(tag) => Some(tag),
            _ => None,
        })
        .filter_map(|tag| {
            if allow_tracing && let TagKind::PsalmTrace = tag.kind {
                let variable_name = tag.description.trim();
                match block_context.locals.get(variable_name) {
                    Some(variable_type) => {
                        let variable_type_str = variable_type.get_id();


                        context.collector.report_with_code(
                            IssueCode::PsalmTrace,
                            Issue::note(format!(
                                "Trace: Type of `{variable_name}` is `{variable_type_str}`"
                            ))
                            .with_annotation(
                                Annotation::primary(tag.description_span)
                                    .with_message(format!("Type is: `{variable_type_str}`")),
                            )
                            .with_note(
                                "Spotted a `@psalm-trace` tag! While this works for compatibility, Mago has a more powerful way to inspect types.",
                            )
                            .with_help(
                                "For more flexible debugging, try using `Mago\\inspect()` directly in your code. It can inspect any expression, not just variables (e.g., `Mago\\inspect($foo->bar());`)."
                            ),
                        );
                    }
                    None => {
                        context.collector.report_with_code(
                            IssueCode::InvalidDocblock,
                            Issue::error(format!(
                                "Invalid `@psalm-trace`: Variable `{variable_name}` not found in this scope."
                            ))
                            .with_annotation(Annotation::primary(tag.description_span).with_message(
                                "This variable is not defined or is out of scope here",
                            ))
                            .with_help(
                                "Check for typos or ensure the variable is defined on a path that reaches this docblock.",
                            ),
                        );
                    }
                }

                return None;
            }

            if !matches!(tag.kind, TagKind::Var | TagKind::PsalmVar | TagKind::PhpstanVar) {
                return None;
            }

            let tag_content = tag.description;

            let var_tag = parse_var_tag(tag_content, tag.description_span)?;
            let variable_name = var_tag.variable.map(|v| v.to_string());
            let type_string = var_tag.type_string;

            match get_type_from_string(
                &type_string.value,
                type_string.span,
                &context.scope,
                &context.type_resolution_context,
                block_context.scope.get_class_like_name(),
            ) {
                Ok(mut variable_type) => {
                    populate_union_type(
                        &mut variable_type,
                        &context.codebase.symbols,
                        block_context.scope.get_reference_source().as_ref(),
                        &mut artifacts.symbol_references,
                        true,
                    );

                    Some((variable_name, variable_type, type_string.span))
                }
                Err(type_error) => {
                    context.collector.report_with_code(
                        IssueCode::InvalidDocblock,
                        Issue::error(format!(
                            "Invalid type in `@var` tag for variable `{}`.",
                            variable_name.as_deref().unwrap_or("expression")
                        ))
                        .with_annotation(Annotation::primary(type_error.span()).with_message(type_error.to_string()))
                        .with_note(type_error.note())
                        .with_help(type_error.help()),
                    );

                    None
                }
            }
        })
        .collect::<Vec<_>>()
}

/// Finds the last applicable `@var` tag for a given variable and parses its type string.
///
/// This function retrieves the docblock associated with the current statement from the
/// context. It then iterates through all `@var`, `@psalm-var`, and `@phpstan-var` tags
/// to find the last one that applies to the specified `variable_id`. If a matching
/// tag is found, it attempts to parse the type string into a `TUnion`.
///
/// If parsing fails, a detailed error is reported to the user.
///
/// # Arguments
///
/// * `context`: The main analysis context, providing access to the docblock parser and error collector.
/// * `variable_id`: The name of the variable (e.g., "$foo") for which to find a type hint.
/// * `variable_span`: The span of the variable's usage, used for error reporting context.
///
/// # Returns
///
/// An `Option<TUnion>` containing the parsed type if a valid, matching `@var` tag
/// was found and successfully parsed. Returns `None` otherwise.
pub fn get_type_from_var_docblock<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    value_expression_variable_id: Option<&str>,
    mut allow_unnamed: bool,
) -> Option<(TUnion, Span)> {
    allow_unnamed = allow_unnamed && !block_context.inside_return && !block_context.inside_loop_expressions;

    get_docblock_variables(context, block_context, artifacts, false)
        .into_iter()
        .filter(|(var_name, _, _)| match var_name {
            None if allow_unnamed => true,
            Some(name) if Some(name.as_str()) == value_expression_variable_id => true,
            _ => false,
        })
        .next_back()
        .map(|(_, variable_type, variable_type_span)| (variable_type, variable_type_span))
}

/// Inserts a variable type from a docblock into the current block context.
///
/// This function is used to handle `@var` tags in docblocks, allowing the
/// type of a variable to be defined or overridden based on the docblock's
/// annotations. It checks if the variable already exists in the block context,
/// and if so, it verifies that the new type is compatible with the existing type.
///
/// # Arguments
///
/// * `context`: The main analysis context, providing access to the error collector.
/// * `block_context`: The current block context, which holds local variables and their types.
/// * `variable_name`: The name of the variable as specified in the docblock.
/// * `variable_type`: The type of the variable as a `TUnion`, parsed from the docblock.
/// * `variable_type_span`: The span of the variable type in the source code, used for error reporting.
/// * `override_existing`: A boolean indicating whether to override an existing variable type
pub fn insert_variable_from_docblock<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    variable_name: String,
    variable_type: TUnion,
    variable_type_span: Span,
    override_existing: bool,
) {
    if !override_existing && block_context.locals.contains_key(&variable_name) {
        return;
    }

    if let Some(previous_type) = block_context.locals.remove(&variable_name)
        && !can_expression_types_be_identical(context.codebase, &previous_type, &variable_type, false, true)
    {
        let variable_type_str = variable_type.get_id();
        let previous_type_str = previous_type.get_id();

        context.collector.report_with_code(
            IssueCode::DocblockTypeMismatch,
            Issue::error(format!("Docblock type mismatch for variable `{variable_name}`."))
                .with_annotation(
                    Annotation::primary(variable_type_span)
                        .with_message(format!("This docblock asserts the type should be `{variable_type_str}`, but it was previously defined as `{previous_type_str}`.")),
                )
                .with_note("The type of the variable defined in the docblock does not match the previously defined type.")
                .with_help(format!(
                    "Change the docblock type to match `{previous_type_str}`, or update the variable definition to a compatible type `{variable_type_str}`."
                )),
        );
    }

    block_context.locals.insert(variable_name, Rc::new(variable_type));
}

pub fn check_docblock_type_incompatibility<'ctx>(
    context: &mut Context<'ctx, '_>,
    value_expression_variable_id: Option<&str>,
    value_expression_span: Span,
    inferred_type: &TUnion,
    docblock_type: &TUnion,
    dockblock_type_span: Span,
    source_expression: Option<&Expression>,
) {
    if !can_expression_types_be_identical(context.codebase, inferred_type, docblock_type, false, true) {
        // Get clean string representations of the types for the error message.
        let docblock_type_str = docblock_type.get_id();
        let inferred_type_str = inferred_type.get_id();

        let mut issue = if let Some(value_expression_variable_id) = value_expression_variable_id {
            Issue::error(format!("Docblock type mismatch for variable `{value_expression_variable_id}`."))
                .with_annotation(
                    Annotation::primary(dockblock_type_span)
                        .with_message(format!("This docblock asserts the type should be `{docblock_type_str}`...")),
                )
        } else {
            Issue::error("Docblock type mismatch for expression.".to_string()).with_annotation(
                Annotation::primary(dockblock_type_span)
                    .with_message(format!("This docblock asserts the type should be `{docblock_type_str}`...")),
            )
        };

        if let Some(value_expression_variable_id) = value_expression_variable_id {
            if let Some(source_expression) = source_expression {
                issue = issue.with_annotation(Annotation::secondary(source_expression.span()).with_message(format!(
                    "...but this expression provides an incompatible type `{inferred_type_str}`."
                )));
            }

            issue = issue.with_annotation(
                Annotation::secondary(value_expression_span)
                    .with_message(format!("The assignment to `{value_expression_variable_id}` here is invalid.")),
            ) .with_note(
                "The type of the assigned value and the `@var` docblock type have no overlap, making this assignment impossible."
            )
            .with_help(format!(
                "Change the assigned value to match `{docblock_type_str}`, or update the `@var` tag to a compatible type."
            ));
        } else {
            issue = issue.with_annotation(
                Annotation::secondary(value_expression_span)
                    .with_message(format!("...but this expression provides an incompatible type `{inferred_type_str}`.")),
            )
            .with_note(
                "The type resolved from the docblock and the type of the expression have no overlap, making the docblock type invalid.",
            )
            .with_help(format!(
                "Change the expression to match `{docblock_type_str}`, or update the `@var` tag to a compatible type."
            ));
        }

        context.collector.report_with_code(IssueCode::DocblockTypeMismatch, issue);
    }
}
