use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;
use crate::internal::format::Format;
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
            for (i, item) in class_like_members.iter().enumerate() {
                let member_kind = match item {
                    ClassLikeMember::TraitUse(_) => ClassLikeMemberKind::TraitUse,
                    ClassLikeMember::Constant(_) => ClassLikeMemberKind::Constant,
                    ClassLikeMember::Property(_) => ClassLikeMemberKind::Property,
                    ClassLikeMember::EnumCase(_) => ClassLikeMemberKind::EnumCase,
                    ClassLikeMember::Method(_) => ClassLikeMemberKind::Method,
                };

                if i != 0 && !last_has_line_after && should_add_empty_line_before(f, member_kind, last_member_kind) {
                    members.push(Document::Line(Line::hard()));
                }

                members.push(item.format(f));

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
                    BraceStyle::NextLine => Document::Array(vec![in f.arena; Document::Line(Line::hard()), Document::BreakParent]),
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
