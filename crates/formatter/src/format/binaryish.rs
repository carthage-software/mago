use fennec_ast::*;
use fennec_span::HasSpan;

use crate::binaryish::BinaryishOperator;
use crate::document::Document;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::format::Format;
use crate::format::Group;
use crate::token;
use crate::Formatter;

use super::IfBreak;

pub(super) fn print_binaryish_expression<'a>(
    f: &mut Formatter<'a>,
    left: &'a Expression,
    operator: BinaryishOperator,
    right: &'a Expression,
) -> Document<'a> {
    let spaced = match operator {
        BinaryishOperator::Concat(_) => f.settings.space_concatenation,
        _ => true,
    };

    let parent_node = f.nth_parent_kind(match operator {
        BinaryishOperator::Logical(_) | BinaryishOperator::Bitwise(_) | BinaryishOperator::Arithmetic(_) => 3,
        BinaryishOperator::Comparison(_) | BinaryishOperator::Concat(_) | BinaryishOperator::Coalesce(_) => 2,
    });

    let is_rhs_of_binaryish = match parent_node {
        Some(Node::LogicalInfixOperation(o)) => o.operator.span().end.offset < operator.span().start.offset,
        Some(Node::ComparisonOperation(o)) => o.operator.span().end.offset < operator.span().start.offset,
        Some(Node::BitwiseInfixOperation(o)) => o.operator.span().end.offset < operator.span().start.offset,
        Some(Node::ArithmeticInfixOperation(o)) => o.operator.span().end.offset < operator.span().start.offset,
        Some(Node::ConcatOperation(o)) => o.dot.end.offset < operator.span().start.offset,
        Some(Node::CoalesceOperation(o)) => o.double_question_mark.end.offset < operator.span().start.offset,
        _ => false,
    };

    let lhs = left.format(f);
    let operator = token!(f, operator.span(), operator.as_str());
    let rhs = right.format(f);

    let must_break = f.settings.preserve_multiline_binary_operations
        && f.source.line_number(left.span().end.offset) != f.source.line_number(right.span().start.offset);

    let spaces = if spaced {
        Document::String(f.as_str(" ".repeat(f.settings.binary_op_spacing.max(1))))
    } else {
        Document::empty()
    };

    if must_break {
        if is_rhs_of_binaryish {
            Document::Group(Group::new(vec![
                lhs,
                spaces,
                operator,
                Document::Line(Line::hardline()),
                rhs,
                Document::BreakParent,
            ]))
        } else {
            Document::Group(Group::new(vec![
                lhs,
                spaces,
                operator,
                Document::Indent(vec![Document::Line(Line::hardline()), rhs]),
            ]))
        }
    } else {
        if is_rhs_of_binaryish {
            Document::Group(Group::new(vec![
                lhs,
                spaces.clone(),
                operator,
                Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), spaces)),
                rhs,
            ]))
        } else {
            Document::Group(Group::new(vec![
                lhs,
                spaces.clone(),
                operator,
                Document::IndentIfBreak(IndentIfBreak::new(vec![
                    Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), spaces)),
                    rhs,
                ])),
            ]))
        }
    }
}

pub(super) fn should_inline_logical_or_coalesce_expression<'a>(expression: &'a Expression) -> bool {
    let rhs = match expression {
        Expression::LogicalOperation(logical_operation) => {
            if let LogicalOperation::Infix(infix_logical_operation) = logical_operation.as_ref() {
                &infix_logical_operation.rhs
            } else {
                return false;
            }
        }
        Expression::CoalesceOperation(coalesce_operation) => &coalesce_operation.rhs,
        _ => return false,
    };

    if let Expression::Array(array) = rhs {
        if array.elements.len() > 0 {
            return true;
        }

        return false;
    }

    if let Expression::List(list) = rhs {
        if list.elements.len() > 0 {
            return true;
        }

        return false;
    }

    if let Expression::LegacyArray(legacy_array) = rhs {
        if legacy_array.elements.len() > 0 {
            return true;
        }

        return false;
    }

    false
}
