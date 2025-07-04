use mago_span::HasSpan;
use mago_syntax::ast::*;
use node::NodeKind;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::binaryish::should_flatten;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::utils::is_at_call_like_expression;
use crate::internal::utils::is_at_callee;
use crate::internal::utils::unwrap_parenthesized;

pub(super) fn print_binaryish_expression<'a>(
    f: &mut FormatterState<'a>,
    left: &'a Expression,
    operator: &'a BinaryOperator,
    right: &'a Expression,
) -> Document<'a> {
    let left = unwrap_parenthesized(left);
    let right = unwrap_parenthesized(right);

    let grandparent = f.grandparent_node();

    let is_inside_parenthesis = matches!(
        grandparent,
        Some(
            Node::If(_)
                | Node::IfStatementBodyElseIfClause(_)
                | Node::IfColonDelimitedBodyElseIfClause(_)
                | Node::While(_)
                | Node::Switch(_)
                | Node::DoWhile(_)
                | Node::Match(_)
        )
    );

    let parts = print_binaryish_expressions(f, left, operator, right, is_inside_parenthesis, false);

    //   if (
    //     $this->hasPlugin("dynamicImports") && $this->lookahead()->type === $tt->parenLeft
    //   ) {
    //
    // looks super weird, we want to break the children if the parent breaks
    //
    //   if (
    //     $this->hasPlugin("dynamicImports") &&
    //     $this->lookahead()->type === $tt->parenLeft
    //   ) {
    if is_inside_parenthesis {
        return Document::Indent(parts);
    }

    // Break between the parens in
    // unaries or in a member or specific call expression, i.e.
    //
    //   (
    //     a &&
    //     b &&
    //     c
    //   ).call()
    if is_at_callee(f)
        || matches!(
            f.grandparent_node(),
            Some(Node::UnaryPrefix(_) | Node::UnaryPostfix(_) | Node::KeyValueArrayElement(_))
        )
    {
        return Document::Group(Group::new(vec![
            Document::Indent(vec![Document::Line(Line::soft()), Document::Array(parts)]),
            Document::Line(Line::soft()),
        ]));
    }

    let should_not_indent = matches!(grandparent, Some(Node::Return(_) | Node::Throw(_)))
        || matches!(grandparent, Some(Node::ArrowFunction(func)) if func.arrow.is_before(operator.span()))
        || matches!(grandparent, Some(Node::For(r#for)) if r#for.body.span().is_after(operator.span()))
        || (matches!(grandparent, Some(Node::Conditional(_)))
            && !matches!(f.great_grandparent_node(), Some(Node::Return(_) | Node::Throw(_)))
            && !is_at_call_like_expression(f));

    let should_indent_if_inlining =
        matches!(grandparent, Some(Node::Assignment(_) | Node::PropertyItem(_) | Node::ConstantItem(_)))
            || matches!(grandparent, Some(Node::KeyValueArrayElement(_)));

    let same_precedence_sub_expression =
        matches!(left, Expression::Binary(binary) if should_flatten(operator, &binary.operator));

    let should_inline_logical_or_coalesce_rhs = should_inline_binary_rhs_expression(right, operator);
    if should_not_indent
        || (should_inline_logical_or_coalesce_rhs && !same_precedence_sub_expression)
        || (!should_inline_logical_or_coalesce_rhs && should_indent_if_inlining)
    {
        return Document::Group(Group::new(parts));
    }

    let first_group_index = parts.iter().position(|part| matches!(part, Document::Group(_)));

    // Separate the leftmost expression, possibly with its leading comments.
    let split_index = first_group_index.unwrap_or(0);
    let mut head_parts = parts[..split_index].to_vec();
    let tail_parts = parts[split_index..].to_vec();

    // Don't include the initial expression in the indentation
    // level. The first item is guaranteed to be the first
    // left-most expression.
    head_parts.push(Document::IndentIfBreak(IndentIfBreak::new(tail_parts)));

    Document::Group(Group::new(head_parts))
}

pub(super) fn print_binaryish_expressions<'a>(
    f: &mut FormatterState<'a>,
    left: &'a Expression,
    operator: &BinaryOperator,
    right: &'a Expression,
    is_inside_parenthesis: bool,
    is_nested: bool,
) -> Vec<Document<'a>> {
    let left = unwrap_parenthesized(left);
    let right = unwrap_parenthesized(right);

    let mut parts = vec![];
    if let Expression::Binary(binary) = left {
        if should_flatten(operator, &binary.operator) {
            // Flatten them out by recursively calling this function.
            parts =
                print_binaryish_expressions(f, &binary.lhs, &binary.operator, &binary.rhs, is_inside_parenthesis, true);
        } else {
            parts.push(left.format(f));
        }
    } else {
        parts.push(left.format(f));
    }

    let should_inline = should_inline_binary_rhs_expression(right, operator);

    let has_space_around = match operator {
        BinaryOperator::And(_)
        | BinaryOperator::Or(_)
        | BinaryOperator::LowAnd(_)
        | BinaryOperator::LowOr(_)
        | BinaryOperator::LowXor(_) => f.settings.space_around_logical_binary_operators,
        BinaryOperator::Equal(_)
        | BinaryOperator::NotEqual(_)
        | BinaryOperator::Identical(_)
        | BinaryOperator::NotIdentical(_)
        | BinaryOperator::AngledNotEqual(_)
        | BinaryOperator::Spaceship(_) => f.settings.space_around_equality_binary_operators,
        BinaryOperator::LessThan(_)
        | BinaryOperator::LessThanOrEqual(_)
        | BinaryOperator::GreaterThan(_)
        | BinaryOperator::GreaterThanOrEqual(_) => f.settings.space_around_comparison_binary_operators,
        BinaryOperator::BitwiseAnd(_) | BinaryOperator::BitwiseOr(_) | BinaryOperator::BitwiseXor(_) => {
            f.settings.space_around_bitwise_binary_operators
        }
        BinaryOperator::Multiplication(_) | BinaryOperator::Division(_) | BinaryOperator::Modulo(_) => {
            f.settings.space_around_multiplicative_binary_operators
        }
        BinaryOperator::Exponentiation(_) => f.settings.space_around_exponentiation_binary_operators,
        BinaryOperator::Addition(_) | BinaryOperator::Subtraction(_) => {
            f.settings.space_around_additive_binary_operators
        }
        BinaryOperator::LeftShift(_) | BinaryOperator::RightShift(_) => f.settings.space_around_shift_binary_operators,
        BinaryOperator::StringConcat(_) => f.settings.space_around_concatenation_binary_operator,
        BinaryOperator::Elvis(_) => f.settings.space_around_elvis_binary_operator,
        BinaryOperator::NullCoalesce(_) => f.settings.space_around_null_coalescing_binary_operator,
        BinaryOperator::Instanceof(_) => true,
    };

    let line_before_operator = f.settings.line_before_binary_operator && !f.has_leading_own_line_comment(right.span());

    let right_document = vec![
        if line_before_operator && !should_inline {
            Document::Line(if has_space_around { Line::default() } else { Line::soft() })
        } else {
            Document::String(if has_space_around { " " } else { "" })
        },
        Document::String(operator.as_str(f.interner)),
        if line_before_operator || should_inline {
            Document::String(if has_space_around { " " } else { "" })
        } else {
            Document::Line(if has_space_around { Line::default() } else { Line::soft() })
        },
        if should_inline { Document::Group(Group::new(vec![right.format(f)])) } else { right.format(f) },
    ];

    // If there's only a single binary expression, we want to create a group
    // in order to avoid having a small right part like -1 be on its own line.
    let parent = f.parent_node();
    let should_break = f.has_comment(left.span(), CommentFlags::Trailing | CommentFlags::Line);
    let should_group = !is_nested
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

pub(super) fn should_inline_binary_expression(expression: &Expression) -> bool {
    match unwrap_parenthesized(expression) {
        Expression::Binary(operation) => {
            if should_inline_binary_rhs_expression(&operation.rhs, &operation.operator) {
                return true;
            }

            match operation.lhs.as_ref() {
                Expression::Binary(_) => should_inline_binary_expression(&operation.lhs),
                left => should_inline_binary_rhs_expression(left, &operation.operator),
            }
        }
        _ => false,
    }
}

pub(super) fn should_inline_binary_rhs_expression(rhs: &Expression, operator: &BinaryOperator) -> bool {
    match unwrap_parenthesized(rhs) {
        Expression::Array(Array { elements, .. })
        | Expression::List(List { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. }) => {
            !elements.is_empty() && (operator.is_logical() || operator.is_null_coalesce())
        }
        Expression::Match(_) => operator.is_elvis() || operator.is_null_coalesce() || operator.is_concatenation(),
        Expression::Instantiation(_) | Expression::Closure(_) | Expression::Call(_) => {
            operator.is_elvis() || operator.is_null_coalesce()
        }
        _ => false,
    }
}
