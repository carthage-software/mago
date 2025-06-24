use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::metadata::enum_case::EnumCaseMetadata;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_enum_case(case: &EnumCase, context: &mut Context<'_>) -> EnumCaseMetadata {
    let span = case.span();
    let attributes = scan_attribute_lists(&case.attribute_lists, context);

    match &case.item {
        EnumCaseItem::Unit(enum_case_unit_item) => {
            EnumCaseMetadata::new(enum_case_unit_item.name.value, enum_case_unit_item.name.span, span)
                .with_attributes(attributes)
                .with_value_type(None)
                .with_is_backed(false)
                .with_is_deprecated(false)
        }
        EnumCaseItem::Backed(enum_case_backed_item) => {
            EnumCaseMetadata::new(enum_case_backed_item.name.value, enum_case_backed_item.name.span, span)
                .with_attributes(attributes)
                .with_is_backed(true)
                .with_is_deprecated(false)
                .with_value_type(
                    infer(context.interner, context.resolved_names, &enum_case_backed_item.value)
                        .map(|u| u.get_single_owned()),
                )
        }
    }
}
