use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;

use crate::internal::checker::expression::check_for_new_without_parenthesis;
use crate::internal::context::Context;

#[inline]
pub fn check_call(call: &Call, context: &mut Context<'_, '_, '_>) {
    match call {
        Call::Function(function_call) => {
            check_for_closure_invocation_without_parentheses(function_call.function, context);
        }
        Call::Method(method_call) => {
            check_for_new_without_parenthesis(method_call.object, context, "method call");
        }
        Call::NullSafeMethod(null_safe_method_call) => {
            if !context.version.is_supported(Feature::NullSafeOperator) {
                context.report(
                    Issue::error("Nullsafe operator is available in PHP 8.0 and above.")
                        .with_annotation(
                            Annotation::primary(null_safe_method_call.question_mark_arrow)
                                .with_message("Nullsafe operator used here."),
                        )
                        .with_help("Upgrade to PHP 8.0 or later to use nullsafe method calls."),
                );
            }

            check_for_new_without_parenthesis(null_safe_method_call.object, context, "nullsafe method call");
        }
        _ => {}
    }
}

/// Checks for closures being immediately invoked without wrapping parentheses,
/// e.g. `function(){}()`.
///
/// PHP requires these to be wrapped: `(function(){})()`.
fn check_for_closure_invocation_without_parentheses(function: &Expression<'_>, context: &mut Context<'_, '_, '_>) {
    if let Expression::Closure(closure) = function {
        context.report(
            Issue::error("Immediately invoked closure must be wrapped in parentheses.")
                .with_annotation(
                    Annotation::primary(closure.span())
                        .with_message("Closure is invoked here without wrapping parentheses."),
                )
                .with_help("Wrap the closure in parentheses before invoking it, e.g. `(function() { ... })()`."),
        );
    }
}
