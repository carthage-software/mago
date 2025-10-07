use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_enum_case<'ctx, 'arena>(
    case: &'arena EnumCase<'arena>,
    context: &mut Context<'ctx, 'arena>,
    scope: &NamespaceScope,
) -> EnumCaseMetadata {
    let span = case.span();
    let attributes = scan_attribute_lists(&case.attribute_lists, context);

    match &case.item {
        EnumCaseItem::Unit(item) => {
            let mut flags = MetadataFlags::UNIT_ENUM_CASE;
            if context.file.file_type.is_host() {
                flags |= MetadataFlags::USER_DEFINED;
            } else if context.file.file_type.is_builtin() {
                flags |= MetadataFlags::BUILTIN;
            }

            let mut meta = EnumCaseMetadata::new(atom(item.name.value), item.name.span, span, flags);

            meta.attributes = attributes;
            meta.value_type = None;
            meta
        }
        EnumCaseItem::Backed(item) => {
            let mut flags = MetadataFlags::BACKED_ENUM_CASE;
            if context.file.file_type.is_host() {
                flags |= MetadataFlags::USER_DEFINED;
            } else if context.file.file_type.is_builtin() {
                flags |= MetadataFlags::BUILTIN;
            }

            let mut meta = EnumCaseMetadata::new(atom(item.name.value), item.name.span, span, flags);

            meta.attributes = attributes;
            meta.value_type = infer(context, scope, &item.value).map(|u| u.get_single_owned());

            meta
        }
    }
}
