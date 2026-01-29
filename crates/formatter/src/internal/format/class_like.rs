use std::cmp::Ordering;

use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::EnumCaseItem;
use mago_syntax::ast::Method;
use mago_syntax::ast::Modifier;
use mago_syntax::ast::PlainProperty;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyItem;
use mago_syntax::ast::Sequence;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::alignment::AlignmentWidths;
use crate::internal::format::alignment::detect_class_member_alignment_runs;
use crate::internal::format::alignment::get_member_alignment;
use crate::internal::format::assignment::AssignmentAlignment;
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

pub fn print_class_like_body<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    left_brace: &'arena Span,
    class_like_members: &'arena Sequence<'arena, ClassLikeMember<'arena>>,
    right_brace: &'arena Span,
    anonymous_class_signature_id: Option<GroupIdentifier>,
) -> Document<'arena> {
    let is_body_empty = block_is_empty(f, left_brace, right_brace);
    let should_inline = is_body_empty
        && if anonymous_class_signature_id.is_some() {
            f.settings.inline_empty_anonymous_class_braces
        } else {
            f.settings.inline_empty_classlike_braces
        };

    let length = class_like_members.len();
    let class_like_members = {
        let mut contents = vec![in f.arena;];
        contents.push(Document::String("{"));
        if let Some(c) = f.print_trailing_comments(*left_brace) {
            contents.push(c);
        }

        if length != 0 {
            let mut last_member_kind = None;
            let mut last_has_line_after = false;
            let mut members = vec![in f.arena; Document::Line(Line::hard())];

            // If enabled, add an empty line directly after the opening brace.
            // This forces a blank line between the `{` and the first member
            // regardless of the original source layout.
            if f.settings.empty_line_after_class_like_open {
                members.push(Document::Line(Line::hard()));
            }

            // Conditionally sort members if sort_class_methods is enabled
            let members_to_format = if f.settings.sort_class_methods {
                sort_class_members(f.arena, class_like_members)
            } else {
                class_like_members.iter().collect_in::<Vec<_>>(f.arena)
            };

            let alignment_runs = detect_class_member_alignment_runs(f, class_like_members.as_slice());

            let mut i = 0;
            let mut formatted_count = 0;
            while i < length {
                let item = members_to_format[i];
                let member_start = item.span().start.offset;
                let member_end = item.span().end.offset;

                if let Some(region) = f.get_ignore_region_for(member_start).copied() {
                    let preserved = f.get_source_slice(region.start, region.end);
                    if formatted_count > 0 && !last_has_line_after {
                        members.push(Document::Line(Line::hard()));
                    }

                    members.push(Document::String(preserved));

                    f.skip_comments_until(region.end);

                    while i < length && members_to_format[i].span().end.offset <= region.end {
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

                if let Some(widths) = get_member_alignment(&alignment_runs, i) {
                    let alignment = calculate_member_alignment(item, &widths);
                    f.set_alignment_context(Some(alignment));
                }

                members.push(item.format(f));

                f.set_alignment_context(None);

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

        contents.push(Document::String("}"));
        if let Some(comments) = f.print_trailing_comments(*right_brace) {
            contents.push(comments);
        }

        Document::Group(Group::new(contents))
    };

    Document::Group(Group::new(vec![
        in f.arena;
        if should_inline {
            Document::space()
        } else {
            match anonymous_class_signature_id {
                Some(signature_id) => match f.settings.closure_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::AlwaysNextLine => Document::Array(vec![in f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                    BraceStyle::NextLine => Document::IfBreak(
                        IfBreak::new(
                            f.arena,
                            Document::space(),
                            Document::Array(vec![in f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                        )
                        .with_id(signature_id),
                    ),
                },
                None => match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine | BraceStyle::AlwaysNextLine => Document::Array(vec![in f.arena; Document::Line(Line::hard()), Document::BreakParent]),
                },
            }
        },
        class_like_members,
    ]))
}

#[inline]
fn should_add_empty_line_before(
    f: &mut FormatterState<'_, '_>,
    class_like_member_kind: ClassLikeMemberKind,
    last_class_like_member_kind: Option<ClassLikeMemberKind>,
) -> bool {
    if let Some(last_member_kind) = last_class_like_member_kind
        && last_member_kind != class_like_member_kind
        && f.settings.separate_class_like_members
    {
        true
    } else {
        false
    }
}

#[inline]
const fn should_add_empty_line_after(
    f: &mut FormatterState<'_, '_>,
    class_like_member_kind: ClassLikeMemberKind,
) -> bool {
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
fn sort_class_members<'arena>(
    arena: &'arena bumpalo::Bump,
    members: &'arena Sequence<'arena, ClassLikeMember<'arena>>,
) -> Vec<'arena, &'arena ClassLikeMember<'arena>> {
    let mut sorted_members = members.iter().collect_in::<Vec<_>>(arena);

    // Use stable sort to preserve relative order of non-methods and equal methods
    sorted_members.sort_by(|a, b| {
        match (*a, *b) {
            // Only compare and sort methods; keep all other members in their original relative order
            (ClassLikeMember::Method(method_a), ClassLikeMember::Method(method_b)) => {
                compare_methods(method_a, method_b)
            }
            // Non-methods stay in original order relative to each other
            _ => Ordering::Equal,
        }
    });

    sorted_members
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
    let a_is_constructor = a_name.eq_ignore_ascii_case("__construct");
    let b_is_constructor = b_name.eq_ignore_ascii_case("__construct");

    if a_is_constructor && !b_is_constructor {
        return Ordering::Less;
    }
    if b_is_constructor && !a_is_constructor {
        return Ordering::Greater;
    }

    // 2. Destructor always comes last
    let a_is_destructor = a_name.eq_ignore_ascii_case("__destruct");
    let b_is_destructor = b_name.eq_ignore_ascii_case("__destruct");

    if a_is_destructor && !b_is_destructor {
        return Ordering::Greater;
    }
    if b_is_destructor && !a_is_destructor {
        return Ordering::Less;
    }

    // 3. Other magic methods (excluding constructor and destructor) come before destructor but after regular methods
    let a_is_magic = a_name.starts_with("__") && !a_is_constructor && !a_is_destructor;
    let b_is_magic = b_name.starts_with("__") && !b_is_constructor && !b_is_destructor;

    match (a_is_magic, b_is_magic) {
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (true, true) => {
            // Both are magic methods, sort alphabetically
            return a_name.to_lowercase().cmp(&b_name.to_lowercase());
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
    a_name.to_lowercase().cmp(&b_name.to_lowercase())
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

/// Calculate alignment padding for a class member based on the run's max widths.
fn calculate_member_alignment(member: &ClassLikeMember<'_>, widths: &AlignmentWidths) -> AssignmentAlignment {
    let (current_type_width, current_name_width) = match member {
        ClassLikeMember::Property(Property::Plain(p)) => {
            let type_width = get_plain_property_type_width(p);
            let name_width = p
                .items
                .iter()
                .filter_map(|item| {
                    if matches!(item, PropertyItem::Concrete(_)) { Some(item.variable().name.len()) } else { None }
                })
                .max()
                .unwrap_or(0);
            (type_width, name_width)
        }
        ClassLikeMember::Constant(constant) => {
            (0, constant.items.iter().map(|item| item.name.value.len()).max().unwrap_or(0))
        }
        ClassLikeMember::EnumCase(case) => {
            if let EnumCaseItem::Backed(backed) = &case.item {
                (0, backed.name.value.len())
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    };

    let type_padding = widths.type_width.saturating_sub(current_type_width);
    let name_padding = widths.name_width.saturating_sub(current_name_width);

    AssignmentAlignment { type_padding, name_padding }
}

fn get_plain_property_type_width(prop: &PlainProperty<'_>) -> usize {
    prop.hint.as_ref().map_or(0, |h| h.span().length() as usize)
}
