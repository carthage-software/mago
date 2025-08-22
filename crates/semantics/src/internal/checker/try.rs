use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_try<'ctx, 'arena>(r#try: &Try<'arena>, context: &mut Context<'ctx, '_, 'arena>) {
    for catch in r#try.catch_clauses.iter() {
        if catch.variable.is_none() && !context.version.is_supported(Feature::CatchOptionalVariable) {
            context.report(
                Issue::error("Cannot use optional variable in `catch` clause.")
                    .with_annotation(Annotation::primary(catch.span()).with_message("`catch` clause without variable."))
                    .with_note("Before PHP 8.0, `catch` clauses must have a variable to store the exception.")
                    .with_help("Add a variable to the `catch` clause to store the exception, or upgrade to PHP 8.0.")
                    .with_link("https://www.php.net/manual/en/language.exceptions.php"),
            );
        }

        let union_supported = context.version.is_supported(Feature::CatchUnionType);

        check_try_catch_hint(&catch.hint, union_supported, context);
    }

    if !r#try.catch_clauses.is_empty() || r#try.finally_clause.is_some() {
        return;
    }

    context.report(
        Issue::error("Cannot use `try` without a `catch` or `finally` clause.")
            .with_annotation(
                Annotation::primary(r#try.span()).with_message("`try` statement without `catch` or `finally`."),
            )
            .with_note("Each `try` block must have at least one corresponding `catch` or `finally` clause.")
            .with_help("Add either a `catch` or `finally` clause to the `try` block.")
            .with_link("https://www.php.net/manual/en/language.exceptions.php"),
    );
}

#[inline]
fn check_try_catch_hint(hint: &Hint, union_supported: bool, context: &mut Context) {
    match hint {
        Hint::Identifier(_) => {}
        Hint::Union(union) => {
            if !union_supported {
                context.report(
                    Issue::error("Cannot use union type hint in `catch` clause.")
                        .with_annotation(
                            Annotation::primary(union.span()).with_message("Union type hint in `catch` clause."),
                        )
                        .with_note("Before PHP 7.1, `catch` clauses must have a single type hint.")
                        .with_help("Use a single type hint in the `catch` clause, or upgrade to PHP 7.1.")
                        .with_link("https://www.php.net/manual/en/language.exceptions.php"),
                );
            }

            check_try_catch_hint(union.left, union_supported, context);
            check_try_catch_hint(union.right, union_supported, context);
        }
        _ => {
            if union_supported {
                context.report(
                    Issue::error("Invalid type hint in `catch` clause.")
                        .with_annotation(
                            Annotation::primary(hint.span()).with_message("Invalid type hint in `catch` clause."),
                        )
                        .with_note("Only identifiers and union types are allowed in `catch` clauses.")
                        .with_help("Use an identifier or union type in the `catch` clause.")
                        .with_link("https://www.php.net/manual/en/language.exceptions.php"),
                );
            } else {
                context.report(
                    Issue::error("Invalid type hint in `catch` clause.")
                        .with_annotation(
                            Annotation::primary(hint.span()).with_message("Invalid type hint in `catch` clause."),
                        )
                        .with_note("Only identifiers are allowed in `catch` clauses.")
                        .with_help("Use an identifier in the `catch` clause.")
                        .with_link("https://www.php.net/manual/en/language.exceptions.php"),
                );
            }
        }
    }
}
