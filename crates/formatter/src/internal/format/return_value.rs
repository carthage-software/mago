use mago_ast::*;
use mago_span::HasSpan;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::misc::has_new_line_in_range;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::utils::get_left_side;
use crate::internal::utils::has_naked_left_side;

pub fn format_return_value<'a>(f: &mut FormatterState<'a>, value: &'a Expression) -> Document<'a> {
    let mut contents = Vec::new();

    if return_argument_has_leading_comment(f, value) {
        contents.push(Document::String("("));
        contents.push(Document::Indent(vec![Document::Line(Line::hard()), value.format(f)]));
        contents.push(Document::Line(Line::hard()));
        contents.push(Document::String(")"));
    } else {
        let mut expression = value;
        while let Expression::Parenthesized(parenthesized) = expression {
            expression = &parenthesized.expression;
        }

        if matches!(expression, Expression::Binary(binary) if !is_simple_expression(&binary.lhs) && !is_simple_expression(&binary.rhs))
            || matches!(expression, Expression::Conditional(conditional) if (
                conditional.then.is_none() || (
                    matches!(conditional.then.as_ref().map(|e| e.as_ref()), Some(Expression::Conditional(_))) &&
                    matches!(conditional.r#else.as_ref(), Expression::Conditional(_))
                )
            ))
        {
            contents.push(Document::Group(Group::new(vec![
                Document::IfBreak(IfBreak::then(Document::String("("))),
                Document::Indent(vec![Document::Line(Line::soft()), value.format(f)]),
                Document::Line(Line::soft()),
                Document::IfBreak(IfBreak::then(Document::String(")"))),
            ])));
        } else {
            contents.push(value.format(f));
        }
    }

    Document::Array(contents)
}

fn return_argument_has_leading_comment<'a>(f: &mut FormatterState<'a>, argument: &'a Expression) -> bool {
    if f.has_leading_own_line_comment(argument.span())
        || f.has_comment_with_filter(argument.span(), CommentFlags::Leading, |comment| {
            has_new_line_in_range(f.source_text, comment.start, comment.end)
        })
    {
        return true;
    }

    if has_naked_left_side(argument) {
        let mut left_most = argument;
        while let Some(new_left_most) = get_left_side(left_most) {
            left_most = new_left_most;

            if f.has_leading_own_line_comment(left_most.span()) {
                return true;
            }
        }
    }

    false
}
