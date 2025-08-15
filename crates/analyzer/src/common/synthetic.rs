use mago_interner::ThreadedInterner;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::sequence::TokenSeparatedSequence;
use mago_syntax::ast::*;

pub fn new_synthetic_call(interner: &ThreadedInterner, f: &str, expression: Expression) -> Expression {
    let str_id = interner.intern(format!("'{}'", f));

    Expression::Call(Call::Function(FunctionCall {
        function: Box::new(Expression::Literal(Literal::String(LiteralString {
            kind: Some(LiteralStringKind::SingleQuoted),
            span: Span::dummy(0, 1),
            raw: str_id,
            value: Some(f.to_string()),
        }))),
        argument_list: ArgumentList {
            left_parenthesis: Span::dummy(0, 1),
            arguments: TokenSeparatedSequence::new(
                vec![Argument::Positional(PositionalArgument { ellipsis: None, value: expression })],
                vec![],
            ),
            right_parenthesis: Span::dummy(0, 1),
        },
    }))
}

pub fn new_synthetic_disjunctive_equality(
    subject: &Expression,
    left: &Expression,
    right: Vec<&Expression>,
) -> Expression {
    let mut expr = new_synthetic_equals(subject, left);
    for r in right {
        expr = new_synthetic_or(&expr, &new_synthetic_equals(subject, r));
    }

    expr
}

pub fn new_synthetic_negation(expression: &Expression) -> Expression {
    if let Expression::Binary(Binary { lhs, operator: BinaryOperator::And(_), rhs }) = expression {
        return new_synthetic_or(&new_synthetic_negation(lhs), &new_synthetic_negation(rhs));
    }

    Expression::UnaryPrefix(UnaryPrefix {
        operator: UnaryPrefixOperator::Not(expression.span()),
        operand: Box::new(expression.clone()),
    })
}

pub fn new_synthetic_variable(interner: &ThreadedInterner, name: &str) -> Expression {
    Expression::Variable(Variable::Direct(DirectVariable { span: Span::dummy(0, 1), name: interner.intern(name) }))
}

pub fn new_synthetic_equals(left: &Expression, right: &Expression) -> Expression {
    new_synthetic_binary(left, BinaryOperator::Equal(Span::dummy(0, 1)), right)
}

pub fn new_synthetic_or(left: &Expression, right: &Expression) -> Expression {
    new_synthetic_binary(left, BinaryOperator::Or(Span::dummy(0, 1)), right)
}

pub fn new_synthetic_binary(left: &Expression, operator: BinaryOperator, right: &Expression) -> Expression {
    Expression::Binary(Binary { lhs: Box::new(left.clone()), operator, rhs: Box::new(right.clone()) })
}
