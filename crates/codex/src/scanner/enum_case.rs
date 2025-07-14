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
        EnumCaseItem::Unit(item) => {
            let mut meta = EnumCaseMetadata::new(item.name.value, item.name.span, span);

            meta.attributes = attributes;
            meta.value_type = None;
            meta.is_backed = false;
            meta.is_deprecated = false;
            meta
        }
        EnumCaseItem::Backed(item) => {
            let mut meta = EnumCaseMetadata::new(item.name.value, item.name.span, span);

            meta.attributes = attributes;
            meta.is_backed = true;
            meta.is_deprecated = false;
            meta.value_type =
                infer(context.interner, context.resolved_names, &item.value).map(|u| u.get_single_owned());

            meta
        }
    }
}
