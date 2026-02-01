use bumpalo::Bump;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::DirectVariable;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Literal;
use mago_syntax::ast::LiteralString;
use mago_syntax::ast::LiteralStringKind;
use mago_syntax::ast::PositionalArgument;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::Variable;
use mago_syntax::ast::sequence::TokenSeparatedSequence;

pub fn new_synthetic_call<'arena>(arena: &'arena Bump, f: &str, expression: Expression<'arena>) -> Expression<'arena> {
    Expression::Call(Call::Function(FunctionCall {
        function: arena.alloc(Expression::Literal(Literal::String(LiteralString {
            kind: Some(LiteralStringKind::SingleQuoted),
            span: Span::zero(),
            raw: arena.alloc_str(&format!("'{f}'")),
            value: Some(arena.alloc_str(f)),
        }))),
        argument_list: ArgumentList {
            left_parenthesis: Span::zero(),
            arguments: TokenSeparatedSequence::new(
                vec![in arena; Argument::Positional(PositionalArgument { ellipsis: None, value: arena.alloc(expression) })],
                vec![in arena],
            ),
            right_parenthesis: Span::zero(),
        },
    }))
}

pub fn new_synthetic_disjunctive_equality<'ast, 'arena>(
    arena: &'arena Bump,
    subject: &'ast Expression<'arena>,
    left: &'ast Expression<'arena>,
    right: Vec<&'ast Expression<'arena>>,
) -> Expression<'arena> {
    let mut expr = new_synthetic_equals(arena, subject, left);
    for r in right {
        expr = new_synthetic_or(arena, &expr, &new_synthetic_equals(arena, subject, r));
    }

    expr
}

pub fn new_synthetic_disjunctive_identity<'ast, 'arena>(
    arena: &'arena Bump,
    subject: &'ast Expression<'arena>,
    left: &'ast Expression<'arena>,
    right: Vec<&'ast Expression<'arena>>,
) -> Expression<'arena> {
    let mut expr = match subject {
        Expression::Literal(Literal::False(_)) => new_synthetic_negation(arena, left),
        Expression::Literal(Literal::True(_)) => left.clone(),
        _ => new_synthetic_identical(arena, subject, left),
    };

    for r in right {
        expr = new_synthetic_or(arena, &expr, &new_synthetic_identical(arena, subject, r));
    }

    expr
}

pub fn new_synthetic_negation<'arena>(arena: &'arena Bump, expression: &Expression<'arena>) -> Expression<'arena> {
    if let Expression::Binary(Binary { lhs, operator: BinaryOperator::And(_), rhs }) = expression {
        return new_synthetic_or(arena, &new_synthetic_negation(arena, lhs), &new_synthetic_negation(arena, rhs));
    }

    Expression::UnaryPrefix(UnaryPrefix {
        operator: UnaryPrefixOperator::Not(expression.span()),
        operand: arena.alloc(expression.clone()),
    })
}

pub fn new_synthetic_variable<'arena>(arena: &'arena Bump, name: &str, span: Span) -> Expression<'arena> {
    Expression::Variable(Variable::Direct(DirectVariable { span, name: arena.alloc_str(name) }))
}

pub fn new_synthetic_identical<'ast, 'arena>(
    arena: &'arena Bump,
    left: &'ast Expression<'arena>,
    right: &'ast Expression<'arena>,
) -> Expression<'arena> {
    new_synthetic_binary(arena, left, BinaryOperator::Identical(Span::zero()), right)
}

pub fn new_synthetic_equals<'ast, 'arena>(
    arena: &'arena Bump,
    left: &'ast Expression<'arena>,
    right: &'ast Expression<'arena>,
) -> Expression<'arena> {
    new_synthetic_binary(arena, left, BinaryOperator::Equal(Span::zero()), right)
}

pub fn new_synthetic_or<'ast, 'arena>(
    arena: &'arena Bump,
    left: &'ast Expression<'arena>,
    right: &'ast Expression<'arena>,
) -> Expression<'arena> {
    new_synthetic_binary(arena, left, BinaryOperator::Or(Span::zero()), right)
}

pub fn new_synthetic_binary<'ast, 'arena>(
    arena: &'arena Bump,
    left: &'ast Expression<'arena>,
    operator: BinaryOperator<'arena>,
    right: &'ast Expression<'arena>,
) -> Expression<'arena> {
    Expression::Binary(Binary { lhs: arena.alloc(left.clone()), operator, rhs: arena.alloc(right.clone()) })
}
