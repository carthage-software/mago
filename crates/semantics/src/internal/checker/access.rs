use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::checker::expression::check_for_new_without_parenthesis;
use crate::internal::context::Context;

#[inline]
pub fn check_access(access: &Access, context: &mut Context<'_, '_, '_>) {
    match access {
        Access::Property(property_access) => {
            check_for_new_without_parenthesis(property_access.object, context, "property access");
        }
        Access::NullSafeProperty(null_safe_access) => {
            if !context.version.is_supported(Feature::NullSafeOperator) {
                context.report(
                    Issue::error("Nullsafe operator is available in PHP 8.0 and above.")
                        .with_annotation(
                            Annotation::primary(null_safe_access.question_mark_arrow)
                                .with_message("Nullsafe operator used here."),
                        )
                        .with_help("Upgrade to PHP 8.0 or later to use nullsafe method calls."),
                );
            }

            check_for_new_without_parenthesis(null_safe_access.object, context, "nullsafe property access");
        }
        Access::ClassConstant(class_constant_access) => {
            if context.version.is_supported(Feature::AccessClassOnObject) {
                return;
            }

            // If the class is an identifier, static, self, or parent, it's fine.
            if let Expression::Identifier(_) | Expression::Static(_) | Expression::Self_(_) | Expression::Parent(_) =
                class_constant_access.class
            {
                return;
            }

            // If the constant is not an identifier, we don't care.
            let ClassLikeConstantSelector::Identifier(local_identifier) = &class_constant_access.constant else {
                return;
            };

            // If the constant is not `class`, we don't care.
            let value = local_identifier.value;
            if !value.eq_ignore_ascii_case("class") {
                return;
            }

            context.report(
                Issue::error("Accessing the `class` constant on an object is only available in PHP 8.0 and above.")
                    .with_annotation(
                        Annotation::primary(class_constant_access.span()).with_message("`class` constant used here."),
                    )
                    .with_help("Use `get_class($object)` instead to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
            );
        }
        _ => {}
    }
}
