use mago_allocator::Arena;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::cst::EnumCase;
use mago_syntax::cst::EnumCaseItem;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::word;

use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer_with_class_constants;
use crate::scanner::version_claim::evaluate_version_attributes;

use super::super::ttype::union::TUnion;

#[inline]
pub fn scan_enum_case<'arena, A>(
    enum_name: Word,
    case: &'arena EnumCase<'arena>,
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    class_constants: &WordMap<ClassLikeConstantMetadata>,
) -> EnumCaseMetadata
where
    A: Arena,
{
    let span = case.span();
    let verdict = evaluate_version_attributes(&case.attribute_lists, context, context.php_version);
    let attributes = scan_attribute_lists(&case.attribute_lists, context);

    match &case.item {
        EnumCaseItem::Unit(item) => {
            let mut flags = MetadataFlags::UNIT_ENUM_CASE;
            flags |= MetadataFlags::origin_flags(context.file.file_type);

            let mut meta = EnumCaseMetadata::new(word(item.name.value), item.name.span, span, flags);

            meta.attributes = attributes;
            meta.value_type = None;
            meta.version_constraint = verdict.constraint;
            meta
        }
        EnumCaseItem::Backed(item) => {
            let mut flags = MetadataFlags::BACKED_ENUM_CASE;
            flags |= MetadataFlags::origin_flags(context.file.file_type);

            let mut meta = EnumCaseMetadata::new(word(item.name.value), item.name.span, span, flags);

            meta.attributes = attributes;
            meta.value_type = infer_with_class_constants(context, scope, item.value, Some(enum_name), class_constants)
                .map(TUnion::get_single_owned);
            meta.version_constraint = verdict.constraint;

            meta
        }
    }
}
