//! Alignment of consecutive assignment-like constructs.
//!
//! The formatter only marks where a column may be aligned. The printer pads the marks once the
//! layout is final, so alignment never depends on how the source was laid out.

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ClassLikeConstant;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::EnumCaseItem;
use mago_syntax::cst::Modifier;
use mago_syntax::cst::Property;
use mago_syntax::cst::PropertyItem;
use mago_syntax::cst::Sequence;

use crate::document::Column;
use crate::document::Document;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;

/// The table that the next assignment-like node adds a row to.
#[derive(Debug, Clone, Copy)]
pub struct Alignment {
    pub table: GroupIdentifier,
    pub adjacent: bool,
    pub row_started: bool,
}

impl Alignment {
    pub const fn new(table: GroupIdentifier, adjacent: bool) -> Self {
        Self { table, adjacent, row_started: false }
    }

    pub const fn started(mut self) -> Self {
        self.row_started = true;
        self
    }

    pub fn row<'arena, A>(self) -> Document<'arena, A>
    where
        A: Arena,
    {
        if self.row_started { Document::empty() } else { self.cell(0) }
    }

    pub fn anchor<'arena, A>(self) -> Document<'arena, A>
    where
        A: Arena,
    {
        self.cell(Column::ANCHOR)
    }

    pub fn cell<'arena, A>(self, index: u8) -> Document<'arena, A>
    where
        A: Arena,
    {
        Document::Column(Column { table: self.table, index, shift: 0, gutter: 0, adjacent: self.adjacent })
    }
}

/// Check if there's a comment between two spans.
pub fn has_comment_between<A>(f: &FormatterState<'_, '_, A>, prev_span: Span, next_span: Span) -> bool
where
    A: Arena,
{
    f.all_comments.iter().any(|comment| {
        let comment_span = comment.span;
        comment_span.start_offset() >= prev_span.end_offset() && comment_span.end_offset() <= next_span.start_offset()
    })
}

pub fn has_attributes(member: &ClassLikeMember<'_>) -> bool {
    match member {
        ClassLikeMember::Property(Property::Plain(property)) => !property.attribute_lists.is_empty(),
        ClassLikeMember::Property(Property::Hooked(property)) => !property.attribute_lists.is_empty(),
        ClassLikeMember::Constant(constant) => !constant.attribute_lists.is_empty(),
        ClassLikeMember::EnumCase(case) => !case.attribute_lists.is_empty(),
        _ => false,
    }
}

/// Whether a class-like member can be a row of an alignment table.
pub fn is_alignable_member(member: &ClassLikeMember<'_>) -> bool {
    match member {
        ClassLikeMember::Property(property) => is_plain_property_with_default(property),
        ClassLikeMember::Constant(_) => true,
        ClassLikeMember::EnumCase(case) => matches!(case.item, EnumCaseItem::Backed(_)),
        _ => false,
    }
}

/// Check if a property is a plain property with a default value.
/// Hooked properties are excluded from alignment because they span multiple lines.
fn is_plain_property_with_default(prop: &Property<'_>) -> bool {
    match prop {
        Property::Plain(p) => p.items.iter().any(|item| matches!(item, PropertyItem::Concrete(_))),
        Property::Hooked(_) => false,
    }
}

/// Check if two members have compatible modifiers for alignment.
/// Members with different modifier combinations should not be aligned together.
pub fn have_compatible_modifiers(prev: &ClassLikeMember<'_>, curr: &ClassLikeMember<'_>) -> bool {
    match (prev, curr) {
        (ClassLikeMember::Property(prev_prop), ClassLikeMember::Property(curr_prop)) => {
            prev_prop.hint().is_some() == curr_prop.hint().is_some()
                && get_property_modifier_signature(prev_prop) == get_property_modifier_signature(curr_prop)
        }
        (ClassLikeMember::Constant(prev_const), ClassLikeMember::Constant(curr_const)) => {
            prev_const.hint.is_some() == curr_const.hint.is_some()
                && get_constant_modifier_signature(prev_const) == get_constant_modifier_signature(curr_const)
        }
        (ClassLikeMember::EnumCase(_), ClassLikeMember::EnumCase(_)) => true,
        _ => false,
    }
}

/// Get a signature representing the modifiers of a property.
fn get_property_modifier_signature(prop: &Property<'_>) -> u32 {
    let modifiers = match prop {
        Property::Plain(p) => &p.modifiers,
        Property::Hooked(p) => &p.modifiers,
    };
    calculate_modifier_signature(modifiers)
}

/// Get a signature representing the modifiers of a constant.
fn get_constant_modifier_signature(constant: &ClassLikeConstant<'_>) -> u32 {
    calculate_modifier_signature(&constant.modifiers)
}

/// Calculate a numeric signature from modifiers.
/// Each modifier type gets a bit in the signature.
fn calculate_modifier_signature(modifiers: &Sequence<'_, Modifier<'_>>) -> u32 {
    let mut sig = 0u32;
    for modifier in modifiers.iter() {
        sig |= match modifier {
            Modifier::Public(_) => 1 << 0,
            Modifier::Protected(_) => 1 << 1,
            Modifier::Private(_) => 1 << 2,
            Modifier::Static(_) => 1 << 3,
            Modifier::Final(_) => 1 << 4,
            Modifier::Abstract(_) => 1 << 5,
            Modifier::Readonly(_) => 1 << 6,
            Modifier::PublicSet(_) => 1 << 7,
            Modifier::ProtectedSet(_) => 1 << 8,
            Modifier::PrivateSet(_) => 1 << 9,
        };
    }
    sig
}
