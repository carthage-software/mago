use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Constant;

use crate::internal::context::Context;

#[inline]
pub fn check_constant(constant: &Constant, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::ConstantAttributes) {
        for attribute_list in &constant.attribute_lists {
            context.report(
                Issue::error("Constant attributes are only available in PHP 8.5 and above.")
                    .with_annotation(
                        Annotation::primary(attribute_list.span()).with_message("Attribute list used here."),
                    )
                    .with_help("Upgrade to PHP 8.5 or later to use constant attributes."),
            );
        }
    }

    for item in &constant.items {
        if !item.value.is_constant(&context.version, true) {
            context.report(
                Issue::error("Constant value must be a constant expression.")
                    .with_annotation(
                        Annotation::primary(item.value.span()).with_message("This is not a constant expression."),
                    )
                    .with_help("Ensure the constant value is a constant expression."),
            );
        }
    }
}
