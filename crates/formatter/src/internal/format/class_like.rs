use mago_ast::*;
use mago_span::HasSpan;
use mago_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::settings::BraceStyle;

use super::block::block_is_empty;

pub fn print_class_like_body<'a>(
    f: &mut FormatterState<'a>,
    left_brace: &'a Span,
    class_like_members: &'a Sequence<ClassLikeMember>,
    right_brace: &'a Span,
    anonymous_class_signature_id: Option<GroupIdentifier>,
) -> Document<'a> {
    let is_body_empty = block_is_empty(f, left_brace, right_brace);
    let should_inline = is_body_empty
        && if anonymous_class_signature_id.is_some() {
            f.settings.inline_empty_anonymous_class_braces
        } else {
            f.settings.inline_empty_classlike_braces
        };

    let length = class_like_members.len();
    let class_like_members = {
        let mut contents = vec![];
        contents.push(Document::String("{"));
        if let Some(c) = f.print_trailing_comments(*left_brace) {
            contents.push(c);
        }

        if length != 0 {
            let mut members = vec![Document::Line(Line::hard())];
            for (i, item) in class_like_members.iter().enumerate() {
                members.push(item.format(f));

                if i < (length - 1) {
                    members.push(Document::Line(Line::hard()));
                    if should_add_empty_line_after(f, item) || f.is_next_line_empty(item.span()) {
                        members.push(Document::Line(Line::hard()));
                    }
                }
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
        if should_inline {
            Document::space()
        } else {
            match anonymous_class_signature_id {
                Some(signature_id) => match f.settings.closure_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => Document::IfBreak(
                        IfBreak::new(
                            Document::space(),
                            Document::Array(vec![Document::Line(Line::hard()), Document::BreakParent]),
                        )
                        .with_id(signature_id),
                    ),
                },
                None => match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => Document::Array(vec![Document::Line(Line::hard()), Document::BreakParent]),
                },
            }
        },
        class_like_members,
    ]))
}

#[inline(always)]
const fn should_add_empty_line_after<'a>(f: &mut FormatterState<'a>, class_like_member: &'a ClassLikeMember) -> bool {
    match class_like_member {
        ClassLikeMember::TraitUse(_) => f.settings.empty_line_after_trait_use,
        ClassLikeMember::Constant(_) => f.settings.empty_line_after_class_like_constant,
        ClassLikeMember::Property(_) => f.settings.empty_line_after_property,
        ClassLikeMember::EnumCase(_) => f.settings.empty_line_after_enum_case,
        ClassLikeMember::Method(_) => f.settings.empty_line_after_method,
    }
}
