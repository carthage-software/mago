use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_for_new_without_parenthesis(object_expr: &Expression, context: &mut Context<'_, '_, '_>, operation: &str) {
    if context.version.is_supported(Feature::NewWithoutParentheses) {
        return;
    }

    let Expression::Instantiation(instantiation) = object_expr else {
        return;
    };

    context.report(
        Issue::error(format!(
            "Direct {operation} on `new` expressions without parentheses is only available in PHP 8.4 and above."
        ))
        .with_annotation(
            Annotation::primary(instantiation.span())
                .with_message(format!("Unparenthesized `new` expression used for {operation}.")),
        ),
    );
}

#[inline]
pub fn check_unary_prefix_operator(unary_prefix_operator: &UnaryPrefixOperator, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::UnsetCast)
        && let UnaryPrefixOperator::UnsetCast(span, _) = unary_prefix_operator
    {
        context.report(
            Issue::error("The `unset` cast is no longer supported in PHP 8.0 and later.")
                .with_annotation(Annotation::primary(*span).with_message("Unset cast used here.")),
        );
    }

    if !context.version.is_supported(Feature::VoidCast)
        && let UnaryPrefixOperator::VoidCast(span, _) = unary_prefix_operator
    {
        context.report(
            Issue::error("The `void` cast is only available in PHP 8.5 and later.")
                .with_annotation(Annotation::primary(*span).with_message("Void cast used here.")),
        );
    }
}
