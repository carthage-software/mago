use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::token::GetPrecedence;
use mago_syntax::token::Precedence;
use node::NodeKind;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::format_token;
use crate::internal::utils::is_at_call_like_expression;
use crate::internal::utils::is_at_callee;
use crate::internal::utils::unwrap_parenthesized;

/// An internal-only enum to represent operators that should be formatted
/// like binary operators. This allows us to reuse the same complex formatting
/// logic for both true `BinaryOperator`s and other constructs like the
/// Elvis operator (`?:`) from a `Conditional` node, without polluting
/// the public AST in `mago_syntax`.
#[derive(Clone, Copy)]
pub(super) enum BinaryishOperator<'arena> {
    Binary(&'arena BinaryOperator<'arena>),
    Elvis(Span),
}

impl<'arena> BinaryishOperator<'arena> {
    fn precedence(self) -> Precedence {
        match self {
            Self::Binary(op) => op.precedence(),
            Self::Elvis(_) => Precedence::ElvisOrConditional,
        }
    }

    fn as_str(self) -> &'arena str {
        match self {
            Self::Binary(op) => op.as_str(),
            Self::Elvis(_) => "?:",
        }
    }

    fn span(self) -> Span {
        match self {
            Self::Binary(op) => op.span(),
            Self::Elvis(span) => span,
        }
    }

    fn is_elvis(self) -> bool {
        matches!(self, Self::Elvis(_))
    }

    fn is_comparison(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_comparison())
    }

    fn is_logical(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_logical())
    }

    fn is_null_coalesce(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_null_coalesce())
    }

    fn is_equality(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_equality())
    }

    fn is_concatenation(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_concatenation())
    }

    fn is_bitwise(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_bitwise())
    }

    fn is_bit_shift(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_bit_shift())
    }

    fn is_same_as(self, other: &BinaryishOperator<'arena>) -> bool {
        match (self, other) {
            (Self::Binary(op1), Self::Binary(op2)) => op1.is_same_as(op2),
            (Self::Elvis(_), Self::Elvis(_)) => true,
            _ => false,
        }
    }

    fn is_low_precedence(self) -> bool {
        matches!(self, Self::Binary(op) if op.is_low_precedence())
    }
}

pub(super) fn print_binaryish_expression<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    left: &'arena Expression<'arena>,
    operator: BinaryishOperator<'arena>,
    right: &'arena Expression<'arena>,
) -> Document<'arena> {
    let left = unwrap_parenthesized(left);
    let right = unwrap_parenthesized(right);

    let grandparent = f.grandparent_node();

    let is_inside_parenthesis = f.is_wrapped_in_parens
        || matches!(
            grandparent,
            Some(
                Node::If(_)
                    | Node::IfStatementBodyElseIfClause(_)
                    | Node::IfColonDelimitedBodyElseIfClause(_)
                    | Node::While(_)
                    | Node::Switch(_)
                    | Node::DoWhile(_)
                    | Node::Match(_)
                    | Node::PositionalArgument(_)
                    | Node::NamedArgument(_)
            )
        );

    let parts = print_binaryish_expression_parts(f, left, operator, right, is_inside_parenthesis, false);

    if is_inside_parenthesis {
        return Document::Array(parts);
    }

    if is_at_callee(f) || matches!(f.grandparent_node(), Some(Node::UnaryPrefix(_) | Node::UnaryPostfix(_))) {
        return Document::Group(Group::new(vec![
            in f.arena;
            Document::Indent(vec![in f.arena; Document::Line(Line::soft()), Document::Array(parts)]),
            Document::Line(Line::soft()),
        ]));
    }

    let should_not_indent = if let Some(Node::Binary(parent_binary)) = grandparent {
        (parent_binary.operator.is_comparison() && operator.is_comparison())
            || (parent_binary.operator.is_logical() && operator.is_logical())
    } else {
        matches!(grandparent, Some(Node::Return(_) | Node::Throw(_)))
            || matches!(grandparent, Some(Node::ArrowFunction(func)) if func.arrow.is_before(operator.span()))
            || matches!(grandparent, Some(Node::For(r#for)) if r#for.body.span().is_after(operator.span()))
            || (matches!(grandparent, Some(Node::Conditional(_)))
                && !matches!(f.great_grandparent_node(), Some(Node::Return(_) | Node::Throw(_)))
                && !is_at_call_like_expression(f))
    };

    let should_indent_if_inlining =
        matches!(grandparent, Some(Node::Assignment(_) | Node::PropertyItem(_) | Node::ConstantItem(_)))
            || matches!(grandparent, Some(Node::KeyValueArrayElement(_)));

    let same_precedence_sub_expression = match left {
        Expression::Binary(binary) => should_flatten(&BinaryishOperator::Binary(&binary.operator), &operator),
        Expression::Conditional(conditional @ Conditional { then: None, .. }) => {
            should_flatten(&BinaryishOperator::Elvis(conditional.question_mark.join(conditional.colon)), &operator)
        }
        _ => false,
    };

    let should_inline_logical_or_coalesce_rhs = should_inline_binary_rhs_expression(f, right, &operator);
    if should_not_indent
        || (should_inline_logical_or_coalesce_rhs && !same_precedence_sub_expression)
        || (!should_inline_logical_or_coalesce_rhs && should_indent_if_inlining)
    {
        return Document::Group(Group::new(parts));
    }

    let first_group_index = parts.iter().position(|part| matches!(part, Document::Group(_)));

    let split_index = first_group_index.unwrap_or(0);
    let mut head_parts = parts;
    let tail_parts = head_parts.split_off(split_index);

    let group_id = f.next_id();

    head_parts.push(Document::IndentIfBreak(IndentIfBreak::new(group_id, tail_parts)));

    Document::Group(Group::new(head_parts).with_id(group_id))
}

fn print_binaryish_expression_parts<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    left: &'arena Expression<'arena>,
    operator: BinaryishOperator<'arena>,
    right: &'arena Expression<'arena>,
    is_inside_parenthesis: bool,
    is_nested: bool,
) -> Vec<'arena, Document<'arena>> {
    let left = unwrap_parenthesized(left);
    let right = unwrap_parenthesized(right);
    let should_break = f
        .has_comment(operator.span(), CommentFlags::Trailing | CommentFlags::Leading | CommentFlags::Line)
        || f.has_comment(left.span(), CommentFlags::Trailing | CommentFlags::Line);

    let mut should_inline_this_level = !should_break && should_inline_binary_rhs_expression(f, right, &operator);
    should_inline_this_level = should_inline_this_level || f.is_in_inlined_binary_chain;

    let old_inlined_chain_state = f.is_in_inlined_binary_chain;
    f.is_in_inlined_binary_chain = should_inline_this_level;

    let mut parts = match left {
        Expression::Binary(binary) => {
            let binaryish_operator = BinaryishOperator::Binary(&binary.operator);
            if should_flatten(&operator, &binaryish_operator) {
                print_binaryish_expression_parts(
                    f,
                    binary.lhs,
                    binaryish_operator,
                    binary.rhs,
                    is_inside_parenthesis,
                    true,
                )
            } else {
                vec![in f.arena; left.format(f)]
            }
        }
        Expression::Conditional(conditional @ Conditional { then: None, .. }) => {
            let binaryish_operator = BinaryishOperator::Elvis(conditional.question_mark.join(conditional.colon));
            if should_flatten(&operator, &binaryish_operator) {
                print_binaryish_expression_parts(
                    f,
                    conditional.condition,
                    binaryish_operator,
                    conditional.r#else,
                    is_inside_parenthesis,
                    true,
                )
            } else {
                vec![in f.arena; left.format(f)]
            }
        }
        _ => vec![in f.arena; left.format(f)],
    };

    f.is_in_inlined_binary_chain = old_inlined_chain_state;

    let has_space_around = match operator {
        BinaryishOperator::Binary(BinaryOperator::StringConcat(_)) => {
            f.settings.space_around_concatenation_binary_operator
        }
        _ => true,
    };

    let line_before_operator = f.settings.line_before_binary_operator && !f.has_leading_own_line_comment(right.span());
    let operator_has_leading_comments = f.has_comment(operator.span(), CommentFlags::Leading);

    let right_document = vec![
        in f.arena;
        if operator_has_leading_comments || (line_before_operator && !should_inline_this_level) {
            Document::Line(if has_space_around { Line::default() } else { Line::soft() })
        } else {
            Document::String(if has_space_around { " " } else { "" })
        },
        format_token(f, operator.span(), operator.as_str()),
        if operator_has_leading_comments || line_before_operator || should_inline_this_level {
            Document::String(if has_space_around { " " } else { "" })
        } else {
            Document::Line(if has_space_around { Line::default() } else { Line::soft() })
        },
        if should_inline_this_level {
             Document::Group(Group::new(vec![in f.arena; right.format(f)]))
        } else {
            right.format(f)
        },
    ];

    let parent = f.parent_node();

    let should_group = !operator_has_leading_comments
        && !is_nested
        && (should_break
            || (!(is_inside_parenthesis && operator.is_logical())
                && parent.kind() != NodeKind::Binary
                && left.node_kind() != NodeKind::Binary
                && right.node_kind() != NodeKind::Binary));

    if should_group {
        parts.push(Document::Group(Group::new(right_document).with_break(should_break)));
    } else {
        parts.extend(right_document);
    }

    parts
}

pub(super) fn should_inline_binary_expression(f: &FormatterState, expression: &Expression) -> bool {
    match unwrap_parenthesized(expression) {
        Expression::Binary(operation) => {
            if operation.lhs.is_binary() || operation.rhs.is_binary() {
                return false;
            }
            should_inline_binary_rhs_expression(f, operation.rhs, &BinaryishOperator::Binary(&operation.operator))
        }
        Expression::Conditional(conditional @ Conditional { then: None, .. }) => should_inline_binary_rhs_expression(
            f,
            conditional.condition,
            &BinaryishOperator::Elvis(conditional.question_mark.join(conditional.colon)),
        ),
        _ => false,
    }
}

fn should_flatten<'arena>(operator: &BinaryishOperator<'arena>, parent_op: &BinaryishOperator<'arena>) -> bool {
    if operator.is_elvis() && parent_op.is_elvis() {
        return true;
    }

    if operator.is_low_precedence() {
        return false;
    }

    let self_precedence = operator.precedence();
    let parent_precedence = parent_op.precedence();

    if self_precedence != parent_precedence {
        return false;
    }

    if let BinaryishOperator::Binary(operator) = operator
        && let BinaryishOperator::Binary(parent_op) = parent_op
    {
        if operator.is_concatenation() && parent_op.is_concatenation() {
            return true;
        }

        if operator.is_arithmetic() && parent_op.is_arithmetic() {
            if matches!((operator, parent_op), (BinaryOperator::Exponentiation(_), BinaryOperator::Exponentiation(_))) {
                return false;
            }
            if matches!(operator, BinaryOperator::Subtraction(_) | BinaryOperator::Division(_))
                || matches!(parent_op, BinaryOperator::Subtraction(_) | BinaryOperator::Division(_))
            {
                return false;
            }
        }
    }

    if operator.is_bitwise() && parent_op.is_bitwise() && (operator.is_bit_shift() || parent_op.is_bit_shift()) {
        return false;
    }

    operator.is_same_as(parent_op)
}

fn should_inline_binary_rhs_expression(
    f: &FormatterState<'_, '_>,
    rhs: &Expression<'_>,
    operator: &BinaryishOperator<'_>,
) -> bool {
    if f.is_in_inlined_binary_chain {
        return true;
    }

    let always_inline_operator = operator.is_null_coalesce() || operator.is_equality() || operator.is_comparison();

    match unwrap_parenthesized(rhs) {
        Expression::Assignment(_) => true,
        Expression::Array(Array { elements, .. })
        | Expression::List(List { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. }) => {
            !elements.is_empty() && (always_inline_operator || operator.is_logical())
        }
        Expression::Match(_) => always_inline_operator || operator.is_elvis() || operator.is_concatenation(),
        Expression::Instantiation(_) | Expression::Closure(_) | Expression::Call(_) => {
            always_inline_operator || operator.is_elvis()
        }
        Expression::Binary(binary) => should_flatten(operator, &BinaryishOperator::Binary(&binary.operator)),
        Expression::Conditional(Conditional { then: None, .. }) => operator.is_elvis(),
        Expression::Throw(_) => operator.is_null_coalesce(),
        _ => false,
    }
}
