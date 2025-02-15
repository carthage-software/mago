use mago_ast::*;
use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;

use crate::internal::context::Context;

#[inline]
pub fn check_list(list: &List, context: &mut Context<'_>) {
    if !context.version.is_supported(Feature::TrailingCommaInListSyntax) {
        if let Some(token) = list.elements.get_trailing_token() {
            context.issues.push(
                Issue::error("Trailing comma in list syntax is only available in PHP 7.2 and above.")
                    .with_annotation(Annotation::primary(token.span).with_message("Trailing comma used here."))
                    .with_help("Upgrade to PHP 7.2 or later to use trailing commas in list syntax."),
            );
        }
    }

    if !context.version.is_supported(Feature::ListReferenceAssignment) {
        for element in list.elements.iter() {
            let value = match element {
                ArrayElement::KeyValue(kv) => kv.value.as_ref(),
                ArrayElement::Value(v) => v.value.as_ref(),
                _ => continue,
            };

            if let Expression::UnaryPrefix(UnaryPrefix {
                operator: UnaryPrefixOperator::Reference(reference), ..
            }) = value
            {
                context.issues.push(
                    Issue::error("Reference assignment in list syntax is only available in PHP 7.3 and above.")
                        .with_annotation(
                            Annotation::primary(reference.span()).with_message("Reference assignment used here."),
                        )
                        .with_help("Upgrade to PHP 7.3 or later to use reference assignment in list syntax."),
                );
            }
        }
    }
}
