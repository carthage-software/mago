use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_names::ResolvedNames;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Block;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::Variable;

use crate::assertion::Assertion;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;

type AssertionMap = BTreeMap<Atom, Vec<Assertion>>;

/// Infers assertions for a function-like whose body is a single expression
/// (arrow function).
pub(super) fn infer_assertions_from_expression_body<'arena>(
    expression: &'arena Expression<'arena>,
    metadata: &mut FunctionLikeMetadata,
    resolved_names: &ResolvedNames<'arena>,
) {
    if has_explicit_assertions(metadata) {
        return;
    }

    let parameter_names = collect_parameter_names(metadata);
    if parameter_names.is_empty() {
        return;
    }

    let (if_true, if_false) = infer_from_expression(expression, &parameter_names, resolved_names, false);

    apply_assertions(metadata, if_true, if_false);
}

/// Infers assertions for a function-like whose body is a block. Only single
/// `return expr;` bodies are recognized to keep inference sound: we cannot
/// reason about side effects or multi-path returns at scan time.
pub(super) fn infer_assertions_from_block_body<'arena>(
    body: &'arena Block<'arena>,
    metadata: &mut FunctionLikeMetadata,
    resolved_names: &ResolvedNames<'arena>,
) {
    if has_explicit_assertions(metadata) {
        return;
    }

    let parameter_names = collect_parameter_names(metadata);
    if parameter_names.is_empty() {
        return;
    }

    let Some(return_expression) = single_return_expression(body) else {
        return;
    };

    let (if_true, if_false) = infer_from_expression(return_expression, &parameter_names, resolved_names, false);

    apply_assertions(metadata, if_true, if_false);
}

fn has_explicit_assertions(metadata: &FunctionLikeMetadata) -> bool {
    !metadata.assertions.is_empty()
        || !metadata.if_true_assertions.is_empty()
        || !metadata.if_false_assertions.is_empty()
}

fn collect_parameter_names(metadata: &FunctionLikeMetadata) -> AtomSet {
    metadata.parameters.iter().map(|p| p.get_name().0).collect()
}

fn single_return_expression<'arena>(body: &'arena Block<'arena>) -> Option<&'arena Expression<'arena>> {
    if body.statements.len() != 1 {
        return None;
    }

    let Statement::Return(ret) = &body.statements.as_slice()[0] else {
        return None;
    };

    ret.value
}

fn apply_assertions(metadata: &mut FunctionLikeMetadata, if_true: AssertionMap, if_false: AssertionMap) {
    if if_true.is_empty() && if_false.is_empty() {
        return;
    }

    for (var, assertions) in if_true {
        metadata.if_true_assertions.entry(var).or_default().extend(assertions);
    }
    for (var, assertions) in if_false {
        metadata.if_false_assertions.entry(var).or_default().extend(assertions);
    }

    metadata.assertions_inferred = true;
}

fn infer_from_expression<'arena>(
    expression: &'arena Expression<'arena>,
    parameter_names: &AtomSet,
    resolved_names: &ResolvedNames<'arena>,
    negated: bool,
) -> (AssertionMap, AssertionMap) {
    let expression = unwrap_parens(expression);

    match expression {
        Expression::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not(_)) => {
            infer_from_expression(unary.operand, parameter_names, resolved_names, !negated)
        }
        Expression::Binary(binary) => match &binary.operator {
            BinaryOperator::Instanceof(_) => parse_instanceof(binary.lhs, binary.rhs, parameter_names, resolved_names)
                .map(|(var, atomic)| build_assertions(var, atomic, negated))
                .unwrap_or_default(),
            BinaryOperator::Identical(_) | BinaryOperator::Equal(_) => {
                parse_null_compare(binary.lhs, binary.rhs, parameter_names)
                    .map(|var| build_assertions(var, TAtomic::Null, negated))
                    .unwrap_or_default()
            }
            BinaryOperator::NotIdentical(_) | BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_) => {
                parse_null_compare(binary.lhs, binary.rhs, parameter_names)
                    .map(|var| build_assertions(var, TAtomic::Null, !negated))
                    .unwrap_or_default()
            }
            _ => Default::default(),
        },
        Expression::Call(Call::Function(call)) => parse_type_check_function(call, parameter_names, resolved_names)
            .map(|(var, atomic)| build_assertions(var, atomic, negated))
            .unwrap_or_default(),
        _ => Default::default(),
    }
}

fn build_assertions(var: Atom, atomic: TAtomic, negated: bool) -> (AssertionMap, AssertionMap) {
    let mut if_true = AssertionMap::new();
    let mut if_false = AssertionMap::new();

    if negated {
        if_true.insert(var, vec![Assertion::IsNotType(atomic.clone())]);
        if_false.insert(var, vec![Assertion::IsType(atomic)]);
    } else {
        if_true.insert(var, vec![Assertion::IsType(atomic.clone())]);
        if_false.insert(var, vec![Assertion::IsNotType(atomic)]);
    }

    (if_true, if_false)
}

fn unwrap_parens<'expr, 'arena>(mut expression: &'expr Expression<'arena>) -> &'expr Expression<'arena> {
    while let Expression::Parenthesized(p) = expression {
        expression = p.expression;
    }
    expression
}

fn parameter_var(expression: &Expression<'_>, parameter_names: &AtomSet) -> Option<Atom> {
    let Expression::Variable(Variable::Direct(direct)) = unwrap_parens(expression) else {
        return None;
    };

    let candidate = atom(direct.name);
    if parameter_names.contains(&candidate) { Some(candidate) } else { None }
}

fn parse_instanceof<'arena>(
    lhs: &Expression<'arena>,
    rhs: &Expression<'arena>,
    parameter_names: &AtomSet,
    resolved_names: &ResolvedNames<'arena>,
) -> Option<(Atom, TAtomic)> {
    let var = parameter_var(lhs, parameter_names)?;

    let Expression::Identifier(identifier) = unwrap_parens(rhs) else {
        return None;
    };

    let class_name = atom(resolved_names.get(identifier));

    Some((var, TAtomic::Object(TObject::Named(TNamedObject::new(class_name)))))
}

fn parse_null_compare<'arena>(
    lhs: &Expression<'arena>,
    rhs: &Expression<'arena>,
    parameter_names: &AtomSet,
) -> Option<Atom> {
    if let Some(var) = parameter_var(lhs, parameter_names)
        && is_null_literal(rhs)
    {
        return Some(var);
    }

    if let Some(var) = parameter_var(rhs, parameter_names)
        && is_null_literal(lhs)
    {
        return Some(var);
    }

    None
}

fn is_null_literal(expression: &Expression<'_>) -> bool {
    matches!(unwrap_parens(expression), Expression::Literal(Literal::Null(_)))
}

fn parse_type_check_function<'arena>(
    call: &FunctionCall<'arena>,
    parameter_names: &AtomSet,
    resolved_names: &ResolvedNames<'arena>,
) -> Option<(Atom, TAtomic)> {
    let Expression::Identifier(function_id) = call.function else {
        return None;
    };

    let resolved = resolved_names.get(function_id);
    let atomic = type_for_check_function(resolved)?;

    if call.argument_list.arguments.len() != 1 {
        return None;
    }

    let arg = call.argument_list.arguments.iter().next()?;

    parameter_var(arg.value(), parameter_names).map(|var| (var, atomic))
}

fn type_for_check_function(name: &str) -> Option<TAtomic> {
    match name.to_ascii_lowercase().as_str() {
        "is_int" | "is_integer" | "is_long" => Some(TAtomic::Scalar(TScalar::int())),
        "is_string" => Some(TAtomic::Scalar(TScalar::string())),
        "is_float" | "is_double" | "is_real" => Some(TAtomic::Scalar(TScalar::float())),
        "is_bool" => Some(TAtomic::Scalar(TScalar::bool())),
        "is_null" => Some(TAtomic::Null),
        "is_object" => Some(TAtomic::Object(TObject::Any)),
        "is_numeric" => Some(TAtomic::Scalar(TScalar::numeric())),
        _ => None,
    }
}
