use mago_allocator::Arena;
use mago_allocator::vec_in;
use std::cmp::Ordering;

use mago_allocator::CollectIn;
use mago_allocator::vec::Vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Method;
use mago_syntax::cst::Modifier;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Sequence;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::alignment::Alignment;
use crate::internal::format::alignment::has_attributes;
use crate::internal::format::alignment::has_comment_between;
use crate::internal::format::alignment::have_compatible_modifiers;
use crate::internal::format::alignment::is_alignable_member;
use crate::internal::format::block::block_is_empty;
use crate::settings::BraceStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ClassLikeMemberKind {
    TraitUse,
    Constant,
    Property,
    EnumCase,
    Method,
}

/// A class member paired with its associated comment index.
/// Used when sorting members to ensure comments move with their associated member.
#[derive(Clone, Copy)]
struct SortedMember<'arena> {
    member: &'arena ClassLikeMember<'arena>,
    comment_index: usize,
}

pub fn print_class_like_body<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    left_brace: &'arena Span,
    class_like_members: &'arena Sequence<'arena, ClassLikeMember<'arena>>,
    right_brace: &'arena Span,
    anonymous_class_signature_id: Option<GroupIdentifier>,
) -> Document<'arena, A>
where
    A: Arena,
{
    let is_body_empty = block_is_empty(f, left_brace, right_brace);
    let should_inline = is_body_empty
        && if anonymous_class_signature_id.is_some() {
            f.settings.inline_empty_anonymous_class_braces
        } else {
            f.settings.inline_empty_classlike_braces
        };

    let length = class_like_members.len();
    let class_like_members = {
        let mut contents = vec_in![f.arena;];
        contents.push(Document::String(b"{"));
        if let Some(c) = f.print_trailing_comments(*left_brace) {
            contents.push(c);
        }

        if length != 0 {
            let mut last_member_kind = None;
            let mut last_has_line_after = false;
            let mut members = vec_in![f.arena; Document::Line(Line::hard())];

            // If enabled, add an empty line directly after the opening brace.
            // This forces a blank line between the `{` and the first member
            // regardless of the original source layout.
            if f.settings.empty_line_after_class_like_open {
                members.push(Document::Line(Line::hard()));
            }

            // Conditionally sort members if sort_class_methods is enabled
            let (members_to_format, is_sorted) = if f.settings.sort_class_methods {
                (sort_class_members(f.arena, f, class_like_members), true)
            } else {
                // When not sorting, wrap members without comment tracking
                let wrapped: Vec<'arena, SortedMember<'arena>, A> = class_like_members
                    .iter()
                    .map(|member| SortedMember { member, comment_index: 0 })
                    .collect_in(f.arena);

                (wrapped, false)
            };

            let mut alignment: Option<(Alignment, &ClassLikeMember<'arena>)> = None;

            let mut i = 0;
            let mut formatted_count = 0;
            while i < length {
                let sorted_member = &members_to_format[i];
                let item = sorted_member.member;
                let member_start = item.span().start.offset;
                let member_end = item.span().end.offset;

                if let Some(region) = f
                    .get_ignore_region_for(member_start)
                    .filter(|region| {
                        region.start >= left_brace.end.offset
                            && region.end <= right_brace.start.offset
                            && member_end <= region.end
                            && class_like_members.iter().all(|member| {
                                let span = member.span();

                                !(span.start.offset < region.start && region.start < span.end.offset
                                    || span.start.offset < region.end && region.end < span.end.offset)
                            })
                    })
                    .copied()
                {
                    let preserved = f.get_source_slice(region.start, region.end);
                    if formatted_count > 0 && !last_has_line_after {
                        members.push(Document::Line(Line::hard()));
                    }

                    members.push(Document::String(preserved));

                    f.skip_comments_until(region.end);

                    while i < length && members_to_format[i].member.span().end.offset <= region.end {
                        i += 1;
                    }

                    if i < length {
                        members.push(Document::Line(Line::hard()));
                        if f.is_next_line_empty_after_index(region.end) {
                            members.push(Document::Line(Line::hard()));
                        }
                        last_has_line_after = true;
                    }

                    formatted_count += 1;
                    last_member_kind = None;
                    continue;
                }

                if let Some(marker_start) = f.consume_ignore_next_before(member_start) {
                    let preserved = f.get_source_slice(marker_start, member_end);
                    if formatted_count > 0 && !last_has_line_after {
                        members.push(Document::Line(Line::hard()));
                    }

                    members.push(Document::String(preserved));

                    f.skip_comments_until(member_end);
                    i += 1;

                    if i < length {
                        members.push(Document::Line(Line::hard()));
                        if f.is_next_line_empty_after_index(member_end) {
                            members.push(Document::Line(Line::hard()));
                            last_has_line_after = true;
                        } else {
                            last_has_line_after = false;
                        }
                    }

                    formatted_count += 1;
                    last_member_kind = None;
                    continue;
                }

                let member_kind = match item {
                    ClassLikeMember::TraitUse(_) => ClassLikeMemberKind::TraitUse,
                    ClassLikeMember::Constant(_) => ClassLikeMemberKind::Constant,
                    ClassLikeMember::Property(_) => ClassLikeMemberKind::Property,
                    ClassLikeMember::EnumCase(_) => ClassLikeMemberKind::EnumCase,
                    ClassLikeMember::Method(_) => ClassLikeMemberKind::Method,
                };

                if formatted_count != 0
                    && !last_has_line_after
                    && should_add_empty_line_before(f, member_kind, last_member_kind)
                {
                    members.push(Document::Line(Line::hard()));
                }

                if f.settings.align_assignment_like && is_alignable_member(item) {
                    let table = match alignment {
                        Some((table, previous))
                            if !last_has_line_after
                                && !has_attributes(item)
                                && have_compatible_modifiers(previous, item)
                                && !has_comment_between(f, previous.span(), item.span()) =>
                        {
                            table
                        }
                        _ => Alignment::new(f.next_id(), false),
                    };

                    alignment = Some((table, item));
                    f.set_alignment(Some(table));
                } else {
                    alignment = None;
                }

                if is_sorted {
                    f.set_next_comment_index(sorted_member.comment_index);
                }

                members.push(item.format(f));

                f.set_alignment(None);

                if i < (length - 1) {
                    members.push(Document::Line(Line::hard()));

                    if should_add_empty_line_after(f, member_kind) || f.is_next_line_empty(item.span()) {
                        members.push(Document::Line(Line::hard()));
                        last_has_line_after = true;
                    } else {
                        last_has_line_after = false;
                    }
                } else {
                    last_has_line_after = false;
                }

                last_member_kind = Some(member_kind);
                formatted_count += 1;
                i += 1;
            }

            if is_sorted && length > 0 {
                let last_member_end = class_like_members.iter().map(|m| m.span().end.offset).max().unwrap_or(0);

                let all_comments = f.all_comments();
                let mut final_index = f.get_next_comment_index();
                while let Some(comment) = all_comments.get(final_index) {
                    if comment.span.end.offset <= last_member_end {
                        final_index += 1;
                    } else {
                        break;
                    }
                }

                f.set_next_comment_index(final_index);
            }

            contents.push(Document::Indent(members));
        }

        if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
            if length > 0 && f.settings.empty_line_before_dangling_comments {
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(comments);
        } else if length > 0 || !should_inline {
            contents.push(Document::Line(Line::hard()));
        }

        if length > 0 && f.settings.empty_line_before_class_like_close {
            contents.push(Document::Line(Line::hard()));
        }

        contents.push(Document::String(b"}"));
        if let Some(comments) = f.print_trailing_comments(*right_brace) {
            contents.push(comments);
        }

        Document::Group(Group::new(contents))
    };

    Document::Group(Group::new(vec_in![f.arena;
        if should_inline {
            Document::space()
        } else {
            match anonymous_class_signature_id {
                Some(signature_id) => match f.settings.closure_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::AlwaysNextLine => Document::Array(vec_in![f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                    BraceStyle::NextLine => Document::IfBreak(
                        IfBreak::new(
                            f.arena,
                            Document::space(),
                            Document::Array(vec_in![f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                        )
                        .with_id(signature_id),
                    ),
                },
                None => match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine | BraceStyle::AlwaysNextLine => Document::Array(vec_in![f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                },
            }
        },
        class_like_members,
    ]))
}

#[inline]
fn should_add_empty_line_before<A>(
    f: &FormatterState<'_, '_, A>,
    class_like_member_kind: ClassLikeMemberKind,
    last_class_like_member_kind: Option<ClassLikeMemberKind>,
) -> bool
where
    A: Arena,
{
    f.settings.separate_class_like_members
        && last_class_like_member_kind.is_some_and(|last_member_kind| last_member_kind != class_like_member_kind)
}

#[inline]
const fn should_add_empty_line_after<A>(
    f: &mut FormatterState<'_, '_, A>,
    class_like_member_kind: ClassLikeMemberKind,
) -> bool
where
    A: Arena,
{
    match class_like_member_kind {
        ClassLikeMemberKind::TraitUse => f.settings.empty_line_after_trait_use,
        ClassLikeMemberKind::Constant => f.settings.empty_line_after_class_like_constant,
        ClassLikeMemberKind::Property => f.settings.empty_line_after_property,
        ClassLikeMemberKind::EnumCase => f.settings.empty_line_after_enum_case,
        ClassLikeMemberKind::Method => f.settings.empty_line_after_method,
    }
}

/// Sorts class members, specifically methods, according to a consistent ordering.
/// Non-method members (constants, properties, trait uses, enum cases) maintain their original order.
///
/// Returns sorted members paired with their original comment indices, ensuring that
/// leading comments (including doc comments) move with their associated methods.
fn sort_class_members<'arena, A>(
    arena: &'arena A,
    f: &FormatterState<'_, 'arena, A>,
    members: &'arena Sequence<'arena, ClassLikeMember<'arena>>,
) -> Vec<'arena, SortedMember<'arena>, A>
where
    A: Arena,
{
    let mut members_with_indices: Vec<'arena, SortedMember<'arena>, A> = Vec::new_in(arena);

    let all_comments = f.all_comments();
    let mut current_comment_index = f.get_next_comment_index();

    for member in members.iter() {
        let member_comment_index = current_comment_index;

        while let Some(comment) = all_comments.get(current_comment_index) {
            if comment.span.end.offset <= member.span().end.offset {
                current_comment_index += 1;
            } else {
                break;
            }
        }

        members_with_indices.push(SortedMember { member, comment_index: member_comment_index });
    }

    let mut methods: Vec<'arena, (&'arena Method<'arena>, SortedMember<'arena>), A> = members_with_indices
        .iter()
        .filter_map(|sorted| match sorted.member {
            ClassLikeMember::Method(method) => Some((method, *sorted)),
            _ => None,
        })
        .collect_in(arena);

    methods.sort_by(|(a, _), (b, _)| compare_methods(a, b));

    let slots = members_with_indices.iter_mut().filter(|sorted| matches!(sorted.member, ClassLikeMember::Method(_)));
    for (slot, (_, method)) in slots.zip(methods) {
        *slot = method;
    }

    members_with_indices
}

/// Compares two methods for sorting purposes.
///
/// Sorting order:
/// 1. Constructor (`__construct`) comes first
/// 2. Static methods (by visibility: public, protected, private)
/// 3. Instance methods (by visibility: public, protected, private)
/// 4. Other magic methods (e.g., `__toString`, `__get`, etc.)
/// 5. Destructor (`__destruct`) comes last
fn compare_methods<'arena>(a: &Method<'arena>, b: &Method<'arena>) -> Ordering {
    let a_name = a.name.value;
    let b_name = b.name.value;

    // 1. Constructor always comes first
    let a_is_constructor = a_name.eq_ignore_ascii_case(b"__construct");
    let b_is_constructor = b_name.eq_ignore_ascii_case(b"__construct");

    if a_is_constructor && !b_is_constructor {
        return Ordering::Less;
    }
    if b_is_constructor && !a_is_constructor {
        return Ordering::Greater;
    }

    // 2. Destructor always comes last
    let a_is_destructor = a_name.eq_ignore_ascii_case(b"__destruct");
    let b_is_destructor = b_name.eq_ignore_ascii_case(b"__destruct");

    if a_is_destructor && !b_is_destructor {
        return Ordering::Greater;
    }
    if b_is_destructor && !a_is_destructor {
        return Ordering::Less;
    }

    // 3. Other magic methods (excluding constructor and destructor) come before destructor but after regular methods
    let a_is_magic = a_name.starts_with(b"__") && !a_is_constructor && !a_is_destructor;
    let b_is_magic = b_name.starts_with(b"__") && !b_is_constructor && !b_is_destructor;

    match (a_is_magic, b_is_magic) {
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (true, true) => {
            // Both are magic methods, sort alphabetically
            return a_name.to_ascii_lowercase().cmp(&b_name.to_ascii_lowercase());
        }
        _ => {}
    }

    // 3. Sort by static vs instance (static comes first)
    let a_is_static = a.is_static();
    let b_is_static = b.is_static();

    match (a_is_static, b_is_static) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }

    // 4. Sort by visibility (public < protected < private)
    let a_visibility = get_visibility_order(&a.modifiers);
    let b_visibility = get_visibility_order(&b.modifiers);

    match a_visibility.cmp(&b_visibility) {
        Ordering::Equal => {}
        other => return other,
    }

    // 5. Sort by abstract vs concrete (abstract comes first)
    let a_is_abstract = a.is_abstract();
    let b_is_abstract = b.is_abstract();

    match (a_is_abstract, b_is_abstract) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }

    // 6. Sort alphabetically by name (case-insensitive)
    a_name.to_ascii_lowercase().cmp(&b_name.to_ascii_lowercase())
}

/// Returns a numeric order for visibility modifiers.
/// Public = 0, Protected = 1, Private = 2
/// If no visibility is specified, defaults to public (0).
fn get_visibility_order(modifiers: &Sequence<'_, Modifier<'_>>) -> u8 {
    if modifiers.contains_public() {
        0
    } else if modifiers.contains_protected() {
        1
    } else if modifiers.contains_private() {
        2
    } else {
        // Default to public if no visibility specified
        0
    }
}
