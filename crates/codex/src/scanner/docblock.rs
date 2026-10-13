use mago_allocator::Arena;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::AssertSubject;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::ParamTagValue;
use mago_phpdoc_syntax::cst::Tag;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::TagVendor;
use mago_phpdoc_syntax::cst::TypelessParamTagValue;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Trivia;
use mago_word::Word;
use mago_word::concat_word;
use mago_word::word;

use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;

const VENDORS_BY_ASCENDING_TRUST: [Option<TagVendor>; 5] =
    [None, Some(TagVendor::Phan), Some(TagVendor::PhpStan), Some(TagVendor::Psalm), Some(TagVendor::Mago)];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum HookParamTag<'arena> {
    Typed(ParamTagValue<'arena>),
    Typeless(TypelessParamTagValue<'arena>),
}

impl<'arena> HookParamTag<'arena> {
    #[must_use]
    pub fn get_type(&self) -> Option<&'arena Type<'arena>> {
        match self {
            HookParamTag::Typed(value) => Some(value.r#type),
            HookParamTag::Typeless(_) => None,
        }
    }
}

impl HasSpan for HookParamTag<'_> {
    fn span(&self) -> Span {
        match self {
            HookParamTag::Typed(value) => value.span(),
            HookParamTag::Typeless(value) => value.span(),
        }
    }
}

pub fn parse_docblock<'arena, A>(context: &Context<'_, 'arena, A>, node: impl HasSpan) -> Option<Document<'arena>>
where
    A: Arena,
{
    let docblock = context.get_docblock(node)?;

    Some(parse_docblock_trivia(context, docblock))
}

pub fn parse_docblock_trivia<'arena, A>(
    context: &Context<'_, 'arena, A>,
    docblock: &'arena Trivia<'arena>,
) -> Document<'arena>
where
    A: Arena,
{
    PHPDocParser::parse_without_trivia(context.arena, docblock.value, docblock.span)
}

pub fn find_most_trusted_tag<'arena, T>(
    document: &Document<'arena>,
    mut extract: impl FnMut(&'arena Tag<'arena>) -> Option<T>,
) -> Option<T> {
    let mut selected: Option<(Option<TagVendor>, T)> = None;
    for tag in document.tags() {
        if selected.as_ref().is_some_and(|(vendor, _)| tag.vendor < *vendor) {
            continue;
        }

        if let Some(value) = extract(tag) {
            selected = Some((tag.vendor, value));
        }
    }

    selected.map(|(_, value)| value)
}

/// Applies a docblock tag shared by every declaration metadata kind.
///
/// Returns whether the tag was handled.
#[inline]
pub fn apply_docblock_flag(
    flags: &mut MetadataFlags,
    explicit_docblock_flags: &mut MetadataFlags,
    flag: MetadataFlags,
    value: bool,
) {
    flags.set(flag, value);
    explicit_docblock_flags.insert(flag);
}

#[inline]
pub fn apply_common_metadata_flag(
    flags: &mut MetadataFlags,
    explicit_docblock_flags: &mut MetadataFlags,
    value: &TagValue<'_>,
) -> bool {
    let (flag, enabled) = match value {
        TagValue::Deprecated(_) => (MetadataFlags::DEPRECATED, true),
        TagValue::NotDeprecated(_) => (MetadataFlags::DEPRECATED, false),
        TagValue::Internal(_) => (MetadataFlags::INTERNAL, true),
        TagValue::Experimental(_) => (MetadataFlags::EXPERIMENTAL, true),
        TagValue::Api(_) => (MetadataFlags::API, true),
        _ => return false,
    };

    apply_docblock_flag(flags, explicit_docblock_flags, flag, enabled);

    true
}

#[inline]
pub fn apply_function_like_metadata_flag(
    flags: &mut MetadataFlags,
    explicit_docblock_flags: &mut MetadataFlags,
    value: &TagValue<'_>,
) -> bool {
    if apply_common_metadata_flag(flags, explicit_docblock_flags, value) {
        return true;
    }

    let (flag, enabled) = match value {
        TagValue::Pure(_) => (MetadataFlags::PURE, true),
        TagValue::Impure(_) => (MetadataFlags::PURE, false),
        TagValue::ExternalMutationFree(_) => (MetadataFlags::EXTERNAL_MUTATION_FREE, true),
        TagValue::MustUse(_) => (MetadataFlags::MUST_USE, true),
        TagValue::NoNamedArguments(_) => (MetadataFlags::NO_NAMED_ARGUMENTS, true),
        TagValue::SuspendsFiber(_) => (MetadataFlags::SUSPENDS_FIBER, true),
        TagValue::MutationFree(_) => {
            apply_docblock_flag(flags, explicit_docblock_flags, MetadataFlags::MUTATION_FREE, true);
            apply_docblock_flag(flags, explicit_docblock_flags, MetadataFlags::EXTERNAL_MUTATION_FREE, true);

            return true;
        }
        _ => return false,
    };

    apply_docblock_flag(flags, explicit_docblock_flags, flag, enabled);

    true
}

#[inline]
pub fn apply_class_like_metadata_flag(
    flags: &mut MetadataFlags,
    explicit_docblock_flags: &mut MetadataFlags,
    value: &TagValue<'_>,
) -> bool {
    if apply_common_metadata_flag(flags, explicit_docblock_flags, value) {
        return true;
    }

    let flag = match value {
        TagValue::ConsistentConstructor(_) => MetadataFlags::CONSISTENT_CONSTRUCTOR,
        TagValue::ConsistentTemplates(_) => MetadataFlags::CONSISTENT_TEMPLATES,
        _ => return false,
    };

    apply_docblock_flag(flags, explicit_docblock_flags, flag, true);

    true
}

pub fn for_each_tag_by_ascending_trust<'arena>(
    document: &Document<'arena>,
    mut apply: impl FnMut(&'arena Tag<'arena>),
) {
    for vendor in VENDORS_BY_ASCENDING_TRUST {
        for tag in document.tags() {
            if tag.vendor == vendor {
                apply(tag);
            }
        }
    }
}

pub fn assertion_subject_word(subject: &AssertSubject<'_>) -> Word {
    match subject {
        AssertSubject::Parameter { variable } => word(variable.value),
        AssertSubject::Method { object, method, .. } => {
            concat_word!(assertion_subject_word(object).as_bytes(), b"->", method.value, b"()")
        }
        AssertSubject::Property { object, property, .. } => {
            concat_word!(assertion_subject_word(object).as_bytes(), b"->", property.value)
        }
    }
}
