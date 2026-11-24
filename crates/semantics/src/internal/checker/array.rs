use mago_database::file::HasFileId;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::ArrayElement;
use mago_syntax::cst::Expression;
use mago_syntax::cst::List;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;

use crate::internal::context::Context;

pub fn check_destructuring_target(target: &Expression, context: &mut Context<'_, '_, '_>) {
    let elements = match target {
        Expression::Array(array) => &array.elements,
        Expression::List(list) => &list.elements,
        _ => return,
    };

    if elements.iter().all(|element| matches!(element, ArrayElement::Missing(_))) {
        context.report(
            Issue::error("Cannot use empty list.")
                .with_annotation(Annotation::primary(target.span()).with_message("Empty list used here."))
                .with_help("Add at least one element to the list."),
        );
    }

    for element in elements.iter() {
        let value = match element {
            ArrayElement::KeyValue(kv) => kv.value,
            ArrayElement::Value(v) => v.value,
            _ => continue,
        };

        check_destructuring_target(value, context);
    }
}

#[inline]
pub fn check_list(list: &List, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::TrailingCommaInListSyntax)
        && let Some(token) = list.elements.get_trailing_token()
    {
        context.report(
            Issue::error("Trailing comma in list syntax is only available in PHP 7.2 and above.")
                .with_annotation(
                    Annotation::primary(token.span_for(context.source_file.file_id()))
                        .with_message("Trailing comma used here."),
                )
                .with_help("Upgrade to PHP 7.2 or later to use trailing commas in list syntax."),
        );
    }

    if !context.version.is_supported(Feature::ListReferenceAssignment) {
        for element in &list.elements {
            let value = match element {
                ArrayElement::KeyValue(kv) => kv.value,
                ArrayElement::Value(v) => v.value,
                _ => continue,
            };

            if let Expression::UnaryPrefix(UnaryPrefix {
                operator: UnaryPrefixOperator::Reference(reference), ..
            }) = value
            {
                context.report(
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
