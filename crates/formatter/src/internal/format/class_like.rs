use mago_ast::*;
use mago_span::HasSpan;
use mago_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::settings::BraceStyle;

pub fn print_class_like_body<'a>(
    f: &mut FormatterState<'a>,
    left_brace: &'a Span,
    class_like_members: &'a Sequence<ClassLikeMember>,
    right_brace: &'a Span,
) -> Document<'a> {
    let inline_empty = match f.settings.classlike_brace_style {
        BraceStyle::SameLine => true,
        BraceStyle::NextLine => false,
    };

    let length = class_like_members.len();
    let mut contents = vec![Document::String("{")];
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
    } else if length > 0 || !inline_empty {
        contents.push(Document::Line(Line::hard()));
    }

    contents.push(Document::String("}"));
    if let Some(comments) = f.print_trailing_comments(*right_brace) {
        contents.push(comments);
    }

    Document::Group(Group::new(contents))
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
