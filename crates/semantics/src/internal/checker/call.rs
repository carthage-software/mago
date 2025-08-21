use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_syntax::ast::*;

use crate::internal::checker::expression::check_for_new_without_parenthesis;
use crate::internal::context::Context;

#[inline]
pub fn check_call(call: &Call, context: &mut Context<'_, '_, '_>) {
    match call {
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
