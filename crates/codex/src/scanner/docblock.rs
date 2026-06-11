use mago_allocator::Arena;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::AssertSubject;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::ParamTagValue;
use mago_phpdoc_syntax::cst::Tag;
use mago_phpdoc_syntax::cst::TagVendor;
use mago_phpdoc_syntax::cst::TypelessParamTagValue;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;
use mago_word::concat_word;
use mago_word::word;

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

    Some(PHPDocParser::parse_with_span(context.arena, docblock.value, docblock.span))
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
        AssertSubject::Method { parameter, method, .. } => concat_word!(parameter.value, b"->", method.value, b"()"),
        AssertSubject::Property { parameter, property, .. } => concat_word!(parameter.value, b"->", property.value),
    }
}
