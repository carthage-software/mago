use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::misc::has_new_line_in_range;
use crate::internal::format::misc::is_expandable_expression;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::format::misc::is_simple_single_line_expression;
use crate::internal::utils::get_left_side;
use crate::internal::utils::has_naked_left_side;
use crate::internal::utils::unwrap_parenthesized;

pub fn format_return_value<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    value: &'arena Expression<'arena>,
) -> Document<'arena> {
    let value = unwrap_parenthesized(value);

    if return_argument_has_leading_comment(f, value) {
        let was_inside_parens = f.is_wrapped_in_parens;
        f.is_wrapped_in_parens = true;
        let value_doc = value.format(f);
        f.is_wrapped_in_parens = was_inside_parens;

        return Document::Array(vec![
            in f.arena;
            (Document::String("(")),
            (Document::Indent(vec![in f.arena; Document::Line(Line::hard()), value_doc])),
            (Document::Line(Line::hard())),
            (Document::String(")")),
        ]);
    }

    if should_wrap_return_value(f, value) {
        let was_inside_parens = f.is_wrapped_in_parens;
        f.is_wrapped_in_parens = true;
        let value_doc = value.format(f);
        f.is_wrapped_in_parens = was_inside_parens;

        let group_id = f.next_id();

        return Document::Group(
            Group::new(vec![
                in f.arena;
                Document::IfBreak(IfBreak::then(f.arena, Document::String("("))),
                Document::IndentIfBreak(IndentIfBreak::new(
                    group_id,
                    vec![in f.arena; Document::Line(Line::soft()), value_doc],
                )),
                Document::Line(Line::soft()),
                Document::IfBreak(IfBreak::then(f.arena, Document::String(")"))),
            ])
            .with_id(group_id),
        );
    }

    value.format(f)
}

fn return_argument_has_leading_comment<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    argument: &'arena Expression<'arena>,
) -> bool {
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

fn should_wrap_return_value<'arena>(f: &mut FormatterState<'_, 'arena>, value: &'arena Expression<'arena>) -> bool {
    match value {
        Expression::Binary(binary) => {
            if is_simple_expression_or_binary(f, binary.lhs) && is_expandable_expression(binary.rhs, true) {
                return false;
            }

            if is_expandable_expression(binary.lhs, false) && is_simple_expression_or_binary(f, binary.rhs) {
                return false;
            }

            true
        }
        Expression::Conditional(conditional) => match conditional.then {
            Some(then) => then.is_conditional() && conditional.r#else.is_conditional(),
            _ => true,
        },
        _ => false,
    }
}

fn is_simple_expression_or_binary<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    expr: &'arena Expression<'arena>,
) -> bool {
    match expr {
        Expression::Binary(binary) => {
            should_inline_simple_expression(f, binary.lhs) && should_inline_simple_expression(f, binary.rhs)
        }
        _ => should_inline_simple_expression(f, expr),
    }
}

fn should_inline_simple_expression<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    expr: &'arena Expression<'arena>,
) -> bool {
    if is_simple_expression(expr) {
        return true;
    }

    is_simple_single_line_expression(f, expr)
}
