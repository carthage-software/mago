use indoc::indoc;

use mago_codex::class_exists;
use mago_codex::is_instance_of;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

const ERROR_CLASS: &str = "Error";

#[derive(Clone, Debug)]
pub struct DontCatchErrorRule;

impl Rule for DontCatchErrorRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Dont Catch Error", Level::Warning).with_description(indoc! {"
            Warns against catching instances of PHP's `Error` class and its critical subclasses.

            In PHP, errors such as `Error`, `TypeError`, `ParseError`, and `CompileError` indicate severe,
            unrecoverable issues in your application. Catching these errors can mask critical failures and lead
            to unpredictable behavior. It is best to let these errors propagate so that the application crashes,
            making the underlying issue easier to diagnose and fix.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Try(r#try) = node else { return LintDirective::default() };

        for catch_clause in r#try.catch_clauses.iter() {
            let errors = get_error_identifiers_from_hint(&catch_clause.hint, context);

            for error in errors {
                let issue =
                    Issue::new(context.level(), "Avoid catching PHP internal errors.")
                        .with_annotation(Annotation::primary(error.span()).with_message(
                            "This throwable is an instance of the `Error` class or one of its subclasses.",
                        ))
                        .with_annotation(
                            Annotation::secondary(catch_clause.span())
                                .with_message("This catch clause intercepts a critical error."),
                        )
                        .with_note("Catching these errors hides issues that should crash your app.")
                        .with_help("Remove or adjust this catch clause so errors propagate naturally.");

                context.report(issue);
            }
        }

        LintDirective::default()
    }
}

fn get_error_identifiers_from_hint<'a>(hint: &'a Hint, context: &LintContext<'_>) -> Vec<&'a Identifier> {
    let mut errors = Vec::new();
    match hint {
        Hint::Identifier(identifier) => {
            let exception_id = context.resolved_names.get(identifier);
            let error_id = context.interner.intern(ERROR_CLASS);

            if !class_exists(context.codebase, context.interner, exception_id) {
                return errors;
            }

            if !class_exists(context.codebase, context.interner, &error_id) {
                return errors;
            }

            if is_instance_of(context.codebase, context.interner, exception_id, &error_id) {
                errors.push(identifier);
            }
        }
        Hint::Union(union_hint) => {
            errors.extend(get_error_identifiers_from_hint(&union_hint.left, context));
            errors.extend(get_error_identifiers_from_hint(&union_hint.right, context));
        }
        _ => {}
    }

    errors
}
