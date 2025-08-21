use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::consts::*;
use crate::internal::context::Context;

#[inline]
pub fn check_top_level_statements<'ast, 'arena>(
    program: &'ast Program<'arena>,
    context: &mut Context<'_, 'ast, 'arena>,
) {
    let mut index = 0;
    let mut before = vec![];

    for statement in program.statements.iter() {
        if !matches!(
            statement,
            Statement::Declare(_)
                | Statement::OpeningTag(_)
                | Statement::Inline(Inline { kind: InlineKind::Shebang, .. })
        ) {
            index += 1;

            before.push(statement.span());

            continue;
        }

        if index == 0 {
            continue;
        }

        if let Statement::Declare(declare) = statement {
            for item in declare.items.iter() {
                if item.name.value.eq_ignore_ascii_case(STRICT_TYPES_DECLARE_DIRECTIVE) {
                    context.report(
                        Issue::error("Strict type declaration must be the first statement in the file.")
                            .with_annotation(
                                Annotation::primary(declare.span()).with_message("Strict type declaration found here."),
                            )
                            .with_annotations(before.iter().map(|span| {
                                Annotation::secondary(*span)
                                    .with_message("This statement appears before the strict type declaration.")
                            }))
                            .with_help("Move all statements before the strict type declaration to after it."),
                    );
                }
            }
        }
    }

    let mut index = 0;
    let mut before = vec![];

    for statement in program.statements.iter() {
        if !matches!(
            statement,
            Statement::Declare(_)
                | Statement::Namespace(_)
                | Statement::OpeningTag(_)
                | Statement::Inline(Inline { kind: InlineKind::Shebang, .. })
        ) {
            index += 1;

            before.push(statement.span());

            continue;
        }

        if index == 0 {
            continue;
        }

        if let Statement::Namespace(namespace) = statement {
            context.report(
                Issue::error("Namespace must be the first statement in the file.")
                    .with_annotation(
                        Annotation::primary(namespace.span()).with_message("Namespace statement found here."),
                    )
                    .with_annotations(before.iter().map(|span| {
                        Annotation::secondary(*span)
                            .with_message("This statement appears before the namespace declaration.")
                    }))
                    .with_help("Move all statements before the namespace declaration to after it."),
            );
        }
    }

    let namespaces =
        Node::Program(program).filter_map(|node| if let Node::Namespace(ns) = node { Some(*ns) } else { None });

    let mut last_unbraced = None;
    let mut last_braced = None;

    for namespace in namespaces {
        let mut namespace_span = namespace.namespace.span();
        if let Some(name) = &namespace.name {
            namespace_span = namespace_span.join(name.span());
        }

        match &namespace.body {
            NamespaceBody::Implicit(body) => {
                if namespace.name.is_none() {
                    context.report(
                        Issue::error("Unbraced namespace must be named.")
                            .with_annotation(
                                Annotation::primary(namespace.span().join(body.terminator.span()))
                                    .with_message("Unnamed unbraced namespace."),
                            )
                            .with_annotation(
                                Annotation::secondary(body.span()).with_message("Namespace body without a name."),
                            )
                            .with_help("Add a name to the unbraced namespace."),
                    );
                }

                last_unbraced = Some((namespace_span, body.span()));
                if let Some((last_namespace_span, last_body_span)) = last_braced {
                    context.report(
                        Issue::error("Cannot mix unbraced namespace declarations with braced namespace declarations.")
                            .with_annotation(
                                Annotation::primary(namespace_span)
                                    .with_message("This is an unbraced namespace declaration."),
                            )
                            .with_annotations([
                                Annotation::primary(last_namespace_span)
                                    .with_message("Previous braced namespace declaration."),
                                Annotation::secondary(last_body_span).with_message("Braced namespace body."),
                                Annotation::secondary(body.span()).with_message("Unbraced namespace body."),
                            ])
                            .with_help(
                                "Use consistent namespace declaration styles: either all braced or all unbraced.",
                            ),
                    );
                }
            }
            NamespaceBody::BraceDelimited(body) => {
                last_braced = Some((namespace_span, body.span()));

                if let Some((last_namespace_span, last_body_span)) = last_unbraced {
                    context.report(
                        Issue::error("Cannot mix braced namespace declarations with unbraced namespace declarations.")
                            .with_annotation(
                                Annotation::primary(namespace_span)
                                    .with_message("This is a braced namespace declaration."),
                            )
                            .with_annotations([
                                Annotation::primary(last_namespace_span)
                                    .with_message("Previous unbraced namespace declaration."),
                                Annotation::secondary(last_body_span).with_message("Unbraced namespace body."),
                                Annotation::secondary(body.span()).with_message("Braced namespace body."),
                            ])
                            .with_help(
                                "Use consistent namespace declaration styles: either all braced or all unbraced.",
                            ),
                    );
                }
            }
        }
    }
}

#[inline]
pub fn check_declare(declare: &Declare, context: &mut Context<'_, '_, '_>) {
    for item in declare.items.iter() {
        let name = item.name.value;

        match name.to_ascii_lowercase().as_str() {
            STRICT_TYPES_DECLARE_DIRECTIVE => {
                let value = match &item.value {
                    Expression::Literal(Literal::Integer(LiteralInteger { value, .. })) => *value,
                    _ => None,
                };

                if !matches!(value, Some(0) | Some(1)) {
                    context.report(
                        Issue::error("The `strict_types` directive must be set to either `0` or `1`.")
                            .with_annotation(
                                Annotation::primary(item.value.span())
                                    .with_message("Invalid value assigned to the directive."),
                            )
                            .with_note("The `strict_types` directive controls strict type enforcement and only accepts `0` (disabled) or `1` (enabled).")
                            .with_help("Set the directive value to either `0` or `1`."),
                    );
                }

                if context.ancestors.len() > 2 {
                    // get the span of the parent, and label it.
                    let parent = context.ancestors[context.ancestors.len() - 2];

                    context.report(
                        Issue::error("The `strict_types` directive must be declared at the top level.")
                            .with_annotation(
                                Annotation::primary(declare.span()).with_message("Directive declared here."),
                            )
                            .with_annotation(
                                Annotation::secondary(parent)
                                    .with_message("This statement should follow the `strict_types` directive."),
                            )
                            .with_help("Move the `strict_types` declaration to the top level of the file."),
                    );
                }
            }
            TICKS_DECLARE_DIRECTIVE => {
                if !matches!(item.value, Expression::Literal(Literal::Integer(_))) {
                    context.report(
                        Issue::error("The `ticks` directive must be set to a literal integer.")
                            .with_annotation(
                                Annotation::primary(item.value.span())
                                    .with_message("Invalid value assigned to the directive."),
                            )
                            .with_note(
                                "The `ticks` directive requires a literal integer value to specify the tick interval.",
                            )
                            .with_help("Provide a literal integer value for the `ticks` directive."),
                    );
                }
            }
            ENCODING_DECLARE_DIRECTIVE => {
                if !matches!(item.value, Expression::Literal(Literal::String(_))) {
                    context.report(
                        Issue::error("The `encoding` declare directive must be set to a literal string")
                            .with_annotation(
                                Annotation::primary(item.value.span())
                                    .with_message("Invalid value assigned to the directive."),
                            )
                            .with_note("The `encoding` directive requires a literal string value to specify the character encoding.")
                            .with_help("Provide a literal string value for the `encoding` directive."),
                    );
                }
            }
            _ => {
                context.report(
                    Issue::error(format!(
                        "`{}` is not a supported `declare` directive. Supported directives are: `{}`.",
                        name,
                        DECLARE_DIRECTIVES.join("`, `")
                    ))
                    .with_annotation(
                        Annotation::primary(item.name.span()).with_message("Unsupported directive used here."),
                    )
                    .with_note("Only specific directives are allowed in `declare` statements.")
                    .with_help(format!("Use one of the supported directives: `{}`.", DECLARE_DIRECTIVES.join("`, `"))),
                );
            }
        }
    }
}

#[inline]
pub fn check_namespace(namespace: &Namespace, context: &mut Context<'_, '_, '_>) {
    if context.ancestors.len() > 2 {
        // get the span of the parent, and label it.
        let parent = context.ancestors[context.ancestors.len() - 2];

        context.report(
            Issue::error("Namespace declaration must be at the top level.")
                .with_annotation(Annotation::primary(namespace.span()).with_message("Namespace declared here."))
                .with_annotation(
                    Annotation::secondary(parent)
                        .with_message("This statement should come after the namespace declaration."),
                )
                .with_note(
                    "Namespace declarations define the scope of the code and should always appear at the top level.",
                )
                .with_help("Move the namespace declaration to the top level of the file."),
        );
    }
}

#[inline]
pub fn check_goto<'ast, 'arena>(goto: &'ast Goto<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
    let all_labels = Node::Program(context.program)
        .filter_map(|node| if let Node::Label(label) = node { Some(*label) } else { None });

    if all_labels.iter().any(|l| l.name.value == goto.label.value) {
        return;
    }

    // If we reach this point, the label was not found.
    // Attempt to find a label with the same name but different case.
    // If found, suggest the correct label.
    let going_to = goto.label.value;
    let mut suggestions = vec![];

    for label in all_labels {
        let label_name = label.name.value;
        if label.name.value.eq_ignore_ascii_case(going_to) {
            suggestions.push((label_name, label.name.span));
        }
    }

    let mut issue = Issue::error(format!("Undefined `goto` label `{going_to}`."))
        .with_annotation(Annotation::primary(goto.label.span).with_message("This `goto` label is not defined."))
        .with_annotations(
            suggestions
                .iter()
                .map(|(name, span)| Annotation::secondary(*span).with_message(format!("Did you mean `{name}`?"))),
        );

    if suggestions.len() == 1 {
        issue = issue
            .with_note(format!("The `goto` label `{}` was not found. Did you mean `{}`?", going_to, suggestions[0].0));
    } else if !suggestions.is_empty() {
        let names = suggestions.iter().map(|(name, _)| format!("`{name}`")).collect::<Vec<_>>().join(", ");
        issue = issue.with_note(format!(
            "The `goto` label `{going_to}` was not found. Did you mean one of the following: {names}?"
        ));
    }

    context.report(issue);
}
