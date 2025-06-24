use ahash::HashMap;

use mago_codex::get_class_like;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use mago_codex::assertion::Assertion;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;

use crate::artifacts::AnalysisArtifacts;
use crate::context::assertion::AssertionContext;
use crate::resolver::class_name::get_class_name_from_atomic;
use crate::utils::expression::get_expression_id;
use crate::utils::misc::unwrap_expression;

#[derive(Debug, Clone, Copy)]
pub enum OtherValuePosition {
    Left,
    Right,
}

pub fn scrape_assertions(
    expression: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let expression = unwrap_expression(expression);
    if let Expression::Call(call) = expression {
        let mut assertions = scrape_function_assertions(&call.span(), artifacts);
        if assertions.is_empty()
            && let Call::Function(function_call) = call
        {
            assertions = scrape_special_function_call_assertions(assertion_context, function_call);
        }

        return assertions;
    }

    let mut if_types = HashMap::default();

    if let Some(var_name) = get_expression_id(
        expression,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    ) {
        if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
    }

    if let Expression::UnaryPrefix(unary_prefix) = &expression
        && let UnaryPrefixOperator::Not(_) = unary_prefix.operator
    {
        return Vec::new();
    }

    if let Expression::Binary(binary) = &expression {
        match binary.operator {
            BinaryOperator::Equal(_) | BinaryOperator::Identical(_) => {
                return scrape_equality_assertions(
                    &binary.lhs,
                    &binary.operator,
                    &binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::NotEqual(_) | BinaryOperator::NotIdentical(_) | BinaryOperator::AngledNotEqual(_) => {
                return scrape_inequality_assertions(
                    &binary.lhs,
                    &binary.operator,
                    &binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::NullCoalesce(_) => {
                let rhs = unwrap_expression(&binary.rhs);
                if matches!(rhs, Expression::Literal(Literal::Null(_))) {
                    let var_name = get_expression_id(
                        &binary.lhs,
                        assertion_context.this_class_name,
                        assertion_context.resolved_names,
                        assertion_context.interner,
                        Some(assertion_context.codebase),
                    );

                    if let Some(var_name) = var_name {
                        if_types.insert(var_name, vec![vec![Assertion::IsIsset]]);
                    }
                }
            }
            BinaryOperator::GreaterThan(_) | BinaryOperator::GreaterThanOrEqual(_) => {
                return scrape_greater_than_assertions(
                    &binary.lhs,
                    &binary.operator,
                    &binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::LessThan(_) | BinaryOperator::LessThanOrEqual(_) => {
                return scrape_lesser_than_assertions(
                    &binary.lhs,
                    &binary.operator,
                    &binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::Instanceof(_) => {
                return scrape_instanceof_assertions(&binary.lhs, &binary.rhs, artifacts, assertion_context);
            }
            _ => {}
        }
    }

    vec![if_types]
}

fn process_custom_assertions(
    expression_span: &Span,
    artifacts: &mut AnalysisArtifacts,
) -> HashMap<String, Vec<Vec<Assertion>>> {
    let mut if_true_assertions = artifacts
        .if_true_assertions
        .get(&(expression_span.start.offset, expression_span.end.offset))
        .cloned()
        .unwrap_or(HashMap::default());

    let if_false_assertions = artifacts
        .if_false_assertions
        .get(&(expression_span.start.offset, expression_span.end.offset))
        .cloned()
        .unwrap_or(HashMap::default());

    if if_true_assertions.is_empty() && if_false_assertions.is_empty() {
        return HashMap::default();
    }

    for if_false_assertion in if_false_assertions {
        if_true_assertions
            .entry(if_false_assertion.0)
            .or_insert_with(Vec::new)
            .extend(if_false_assertion.1.into_iter().map(|a| a.get_negation()).collect::<Vec<_>>());
    }

    if_true_assertions.into_iter().map(|(k, v)| (k, v.into_iter().map(|v| vec![v]).collect())).collect()
}

fn scrape_special_function_call_assertions(
    assertion_context: AssertionContext<'_>,
    function_call: &FunctionCall,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let Expression::Identifier(function_identifier) = function_call.function.as_ref() else {
        return Vec::new();
    };

    let resolved_function_name_id = assertion_context.resolved_names.get(function_identifier);
    let resolved_function_name = assertion_context.interner.lookup(resolved_function_name_id);
    let function_name = if resolved_function_name.starts_with("is_") || resolved_function_name.starts_with("ctype_") {
        resolved_function_name
    } else if function_identifier.is_local() {
        assertion_context.interner.lookup(function_identifier.value())
    } else {
        return Vec::new();
    };

    let function_assertion = match function_name {
        "is_countable" => Assertion::Countable,
        "ctype_digit" => {
            Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(true, false, false))))
        }
        "ctype_lower" => {
            Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(false, false, true))))
        }
        _ => return Vec::new(),
    };

    let Some(first_argument_variable_id) =
        function_call.argument_list.arguments.get(0).map(|argument| argument.value()).and_then(|argument_expression| {
            get_expression_id(
                argument_expression,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            )
        })
    else {
        return Vec::new();
    };

    let mut if_types = HashMap::default();
    if_types.insert(first_argument_variable_id, vec![vec![function_assertion]]);

    vec![if_types]
}

fn scrape_equality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    if let Some(null_position) = has_null_variable(left, right) {
        return get_null_equality_assertions(left, right, assertion_context, null_position);
    }

    if let Some(true_position) = has_true_variable(left, right) {
        return get_true_equality_assertions(left, operator, right, assertion_context, true_position);
    }

    if let Some(false_position) = has_false_variable(left, right) {
        return get_false_equality_assertions(left, operator, right, assertion_context, false_position);
    }

    if let Some(empty_array_position) = has_empty_array_variable(left, right) {
        return get_empty_array_equality_assertions(left, operator, right, assertion_context, empty_array_position);
    }

    if let Some(typed_value_position) = has_typed_value_comparison(left, right, artifacts, assertion_context) {
        return get_typed_value_equality_assertions(
            left,
            operator,
            right,
            artifacts,
            assertion_context,
            typed_value_position,
        );
    }

    Vec::new()
}

fn scrape_inequality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    if let Some(null_position) = has_null_variable(left, right) {
        return get_null_inequality_assertions(left, right, assertion_context, null_position);
    }

    if let Some(false_position) = has_false_variable(left, right) {
        return get_false_inquality_assertions(left, right, assertion_context, false_position);
    }

    if let Some(true_position) = has_true_variable(left, right) {
        return get_true_inquality_assertions(left, right, assertion_context, true_position);
    }

    if let Some(empty_array_position) = has_empty_array_variable(left, right) {
        return get_empty_array_inequality_assertions(left, operator, right, assertion_context, empty_array_position);
    }

    if let Some(typed_value_position) = has_typed_value_comparison(left, right, artifacts, assertion_context) {
        return get_typed_value_inequality_assertions(
            left,
            operator,
            right,
            artifacts,
            assertion_context,
            typed_value_position,
        );
    }

    Vec::new()
}

fn scrape_function_assertions(
    span: &Span,
    artifacts: &mut AnalysisArtifacts,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let if_types = process_custom_assertions(span, artifacts);

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_empty_array_equality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if operator.is_identity() {
            if_types.insert(var_name, vec![vec![Assertion::EmptyCountable]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Falsy]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_empty_array_inequality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if operator.is_identity() {
            if_types.insert(var_name, vec![vec![Assertion::NonEmptyCountable(true)]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_null_equality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Null)]]);
    }

    vec![if_types]
}

fn get_null_inequality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
    }

    vec![if_types]
}

fn get_false_inquality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    false_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsNotType(TAtomic::Scalar(TScalar::r#false()))]]);
    }

    vec![if_types]
}

fn get_true_inquality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    true_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::r#true()))]]);
    }

    vec![if_types]
}

fn scrape_lesser_than_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    match has_non_empty_count_equality_check(left, operator, right, artifacts, assertion_context) {
        (Some(minimum_count), None) => {
            let mut if_types = HashMap::default();

            let counter_variable_id = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            if let Some(counter_variable_id) = counter_variable_id {
                if minimum_count == 1 {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::NonEmptyCountable(true)]]);
                } else if minimum_count > 1 {
                    if_types
                        .insert(counter_variable_id, vec![vec![Assertion::HasAtLeastCount(minimum_count as usize)]]);
                } else {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (None, None) => {
            // Continue to check for other conditions
        }
        _ => {
            unreachable!("unexpected non-empty count equality check")
        }
    }

    match has_less_than_count_equality_check(left, operator, right, artifacts, assertion_context) {
        (None, Some(maximum_count)) => {
            let mut if_types = HashMap::default();

            let counter_variable_id = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            if let Some(counter_variable_id) = counter_variable_id {
                if maximum_count > 0 {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::DoesNotHaveAtLeastCount(maximum_count as usize + 1)]],
                    );
                } else {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::EmptyCountable]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (None, None) => {
            // Continue to check for other conditions
        }
        _ => {
            unreachable!("unexpected less than count equality check")
        }
    };

    let (count, variable, is_left) = match get_comparison_literal_operand(artifacts, left, right) {
        (Some(count), None) => (count, right, true),
        (None, Some(count)) => (count, left, false),
        _ => return Vec::new(),
    };

    let mut if_types = HashMap::default();

    let variable_id = get_expression_id(
        variable,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(counter_variable_id) = variable_id {
        if is_left {
            if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
                if_types.insert(counter_variable_id, vec![vec![Assertion::IsGreaterThanOrEqual(count)]]);
            } else {
                if_types.insert(counter_variable_id, vec![vec![Assertion::IsGreaterThan(count)]]);
            }
        } else if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            if_types.insert(counter_variable_id, vec![vec![Assertion::IsLessThanOrEqual(count)]]);
        } else {
            if_types.insert(counter_variable_id, vec![vec![Assertion::IsLessThan(count)]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn scrape_greater_than_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    match has_non_empty_count_equality_check(left, operator, right, artifacts, assertion_context) {
        (None, Some(minimum_count)) => {
            let mut if_types = HashMap::default();

            let counter_variable_id = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            if let Some(counter_variable_id) = counter_variable_id {
                if minimum_count == 1 {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::NonEmptyCountable(true)]]);
                } else if minimum_count > 1 {
                    if_types
                        .insert(counter_variable_id, vec![vec![Assertion::HasAtLeastCount(minimum_count as usize)]]);
                } else {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (None, None) => {
            // Continue to check for other conditions
        }
        _ => {
            unreachable!("unexpected non-empty count equality check")
        }
    }

    match has_less_than_count_equality_check(left, operator, right, artifacts, assertion_context) {
        (Some(maximum_count), None) => {
            let mut if_types = HashMap::default();

            let counter_variable_id = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            if let Some(counter_variable_id) = counter_variable_id {
                if maximum_count > 0 {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::DoesNotHaveAtLeastCount(maximum_count as usize + 1)]],
                    );
                } else {
                    if_types.insert(counter_variable_id, vec![vec![Assertion::EmptyCountable]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (None, None) => {
            // Continue to check for other conditions
        }
        _ => {
            unreachable!("unexpected less than count equality check")
        }
    }

    let (count, variable, is_left) = match get_comparison_literal_operand(artifacts, left, right) {
        (Some(count), None) => (count, right, true),
        (None, Some(count)) => (count, left, false),
        _ => return Vec::new(),
    };

    let mut if_types = HashMap::default();

    let variable_id = get_expression_id(
        variable,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(counter_variable_id) = variable_id {
        if is_left {
            if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
                if_types.insert(counter_variable_id, vec![vec![Assertion::IsLessThanOrEqual(count)]]);
            } else {
                if_types.insert(counter_variable_id, vec![vec![Assertion::IsLessThan(count)]]);
            }
        } else if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            if_types.insert(counter_variable_id, vec![vec![Assertion::IsGreaterThanOrEqual(count)]]);
        } else {
            if_types.insert(counter_variable_id, vec![vec![Assertion::IsGreaterThan(count)]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn scrape_instanceof_assertions(
    left: &Expression,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    context: AssertionContext<'_>,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();

    let variable_id = get_expression_id(
        left,
        context.this_class_name,
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    if let Some(counter_variable_id) = variable_id {
        match right {
            Expression::Identifier(identifier) => {
                let resolved_name = context.resolved_names.get(identifier);

                if_types.insert(
                    counter_variable_id,
                    vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new(*resolved_name))))]],
                );
            }
            Expression::Self_(_) => {
                if let Some(self_class) = context.this_class_name {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new_this(
                            *self_class,
                        ))))]],
                    );
                }
            }
            Expression::Static(_) => {
                if let Some(self_class) = context.this_class_name {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::IsIdentical(TAtomic::Object(TObject::Named(TNamedObject::new_this(
                            *self_class,
                        ))))]],
                    );
                }
            }
            Expression::Parent(_) => {
                if let Some(self_class) = context.this_class_name
                    && let Some(self_meta) = get_class_like(context.codebase, context.interner, self_class)
                    && let Some(parent_id_ref) = self_meta.get_direct_parent_class_ref()
                {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new(
                            *parent_id_ref,
                        ))))]],
                    );
                }
            }
            expression => {
                if let Some(expression_type) = artifacts.get_expression_type(expression) {
                    let mut assertions = vec![];
                    for atomic in &expression_type.types {
                        let Some(classname) = get_class_name_from_atomic(context.interner, atomic) else {
                            continue;
                        };

                        if let Some(fq_id) = classname.fq_class_id {
                            assertions
                                .push(Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new(fq_id)))));
                        }
                    }

                    if !assertions.is_empty() {
                        if_types.insert(counter_variable_id, vec![assertions]);
                    }
                }
            }
        };
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn has_non_empty_count_equality_check(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> (Option<i64>, Option<i64>) {
    let is_greater_than = matches!(operator, BinaryOperator::GreaterThan(_));
    let is_greater_than_or_equal = matches!(operator, BinaryOperator::GreaterThanOrEqual(_));

    if !is_greater_than && !is_greater_than_or_equal {
        return (None, None);
    }

    if is_count_or_size_of_call(assertion_context, left) {
        let right_value =
            get_expression_integer_value(artifacts, right).filter(|v| if is_greater_than { *v > 0 } else { *v > 1 });

        if is_greater_than {
            return (None, right_value.map(|v| v.wrapping_add(1)));
        } else {
            return (None, right_value);
        }
    }

    if is_count_or_size_of_call(assertion_context, right) {
        let left_value =
            get_expression_integer_value(artifacts, left).filter(|v| if is_greater_than { *v > 0 } else { *v > 1 });

        if is_greater_than {
            return (left_value.map(|v| v.wrapping_add(1)), None);
        } else {
            return (left_value, None);
        }
    }

    (None, None)
}

fn has_less_than_count_equality_check(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &mut AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> (Option<i64>, Option<i64>) {
    let is_less_than = matches!(operator, BinaryOperator::LessThan(_));
    let is_less_than_or_equal = matches!(operator, BinaryOperator::LessThanOrEqual(_));

    if (is_less_than_or_equal || is_less_than) && is_count_or_size_of_call(assertion_context, left) {
        let right_value = get_expression_integer_value(artifacts, right);

        if is_less_than {
            return (None, right_value);
        } else {
            return (None, right_value.map(|v| v - 1));
        }
    }

    let is_greater_than = matches!(operator, BinaryOperator::GreaterThan(_));
    let is_greater_than_or_equal = matches!(operator, BinaryOperator::GreaterThanOrEqual(_));

    if (is_greater_than_or_equal || is_greater_than) && is_count_or_size_of_call(assertion_context, right) {
        let left_value = get_expression_integer_value(artifacts, left);

        if is_greater_than {
            return (left_value.map(|v| v - 1), None);
        } else {
            return (left_value, None);
        }
    }

    (None, None)
}

fn get_comparison_literal_operand(
    artifacts: &mut AnalysisArtifacts,
    left: &Expression,
    right: &Expression,
) -> (Option<i64>, Option<i64>) {
    if let Some(value) = get_expression_integer_value(artifacts, left) {
        return (Some(value), None);
    }

    if let Some(value) = get_expression_integer_value(artifacts, right) {
        return (None, Some(value));
    }

    (None, None)
}

fn get_expression_integer_value(artifacts: &mut AnalysisArtifacts, expression: &Expression) -> Option<i64> {
    if let Some(value) = artifacts.get_expression_type(expression).and_then(|t| t.get_single_literal_int_value()) {
        return Some(value);
    }

    if let Expression::Literal(Literal::Integer(integer)) = expression {
        return Some(integer.value as i64);
    }

    if let Expression::UnaryPrefix(UnaryPrefix { operator, operand }) = expression
        && let Expression::Literal(Literal::Integer(integer)) = operand.as_ref()
    {
        let value = integer.value as i64;

        match operator {
            UnaryPrefixOperator::Plus(_) => return Some(value),
            UnaryPrefixOperator::Negation(_) => return Some(-value),
            _ => {}
        }
    }

    None
}

fn is_count_or_size_of_call(assertion_context: AssertionContext<'_>, expression: &Expression) -> bool {
    let Expression::Call(Call::Function(FunctionCall { function, argument_list })) = expression else {
        return false;
    };

    if argument_list.arguments.len() != 1 {
        return false;
    }

    let Expression::Identifier(function_identifier) = function.as_ref() else {
        return false;
    };

    let func_name = assertion_context.interner.lookup(function_identifier.value());

    func_name.eq_ignore_ascii_case("count") || func_name.eq_ignore_ascii_case("sizeof")
}

fn get_true_equality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    true_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if operator.is_identity() {
            if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::r#true()))]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
        }

        return vec![if_types];
    }

    Vec::new()
}

pub fn has_typed_value_comparison(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> Option<OtherValuePosition> {
    let left_var_id = get_expression_id(
        left,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    let right_var_id = get_expression_id(
        right,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(right_type) = artifacts.get_expression_type(&right.span())
        && (left_var_id.is_some() || right_var_id.is_none())
        && right_type.is_single()
        && !right_type.is_mixed()
    {
        return Some(OtherValuePosition::Right);
    }

    if let Some(left_type) = artifacts.get_expression_type(&left.span())
        && left_var_id.is_none()
        && left_type.is_single()
        && !left_type.is_mixed()
    {
        return Some(OtherValuePosition::Left);
    }
    None
}

fn get_false_equality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    false_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if operator.is_identity() {
            if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::r#false()))]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Falsy]]);
        }

        return vec![if_types];
    }

    vec![]
}

fn get_typed_value_equality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
    typed_value_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();

    let var_name;
    let other_value_var_name;
    let var_type;
    let other_value_type;

    match typed_value_position {
        OtherValuePosition::Right => {
            var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            other_value_var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            var_type = artifacts.get_expression_type(&left.span());
            other_value_type = artifacts.get_expression_type(&right.span());
        }
        OtherValuePosition::Left => {
            var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            var_type = artifacts.get_expression_type(&right.span());
            other_value_type = artifacts.get_expression_type(&left.span());
        }
    }

    let Some(var_name) = var_name else {
        return vec![];
    };

    let Some(other_value_type) = other_value_type else {
        return vec![];
    };

    if other_value_type.is_single() {
        let other_value_atomic = other_value_type.get_single().clone();

        let orred_types = if other_value_atomic.is_enum_case() {
            vec![Assertion::IsType(other_value_atomic)]
        } else if operator.is_identity() {
            vec![Assertion::IsIdentical(other_value_atomic)]
        } else {
            vec![Assertion::IsEqual(other_value_atomic)]
        };

        if_types.insert(var_name, vec![orred_types]);
    }

    if let Some(other_value_var_name) = other_value_var_name
        && let Some(var_type) = var_type
        && !var_type.is_mixed()
        && var_type.is_single()
    {
        let orred_types = if operator.is_identity() {
            vec![Assertion::IsIdentical(var_type.get_single().clone())]
        } else {
            vec![Assertion::IsEqual(var_type.get_single().clone())]
        };

        if_types.insert(other_value_var_name, vec![orred_types]);
    }

    if !if_types.is_empty() { vec![if_types] } else { vec![] }
}

fn get_typed_value_inequality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
    typed_value_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let mut if_types = HashMap::default();

    let var_name;
    let other_value_var_name;
    let other_value_type;
    let var_type;

    match typed_value_position {
        OtherValuePosition::Right => {
            var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            var_type = artifacts.get_expression_type(&left.span());
            other_value_type = artifacts.get_expression_type(&right.span());
        }
        OtherValuePosition::Left => {
            var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                assertion_context.interner,
                Some(assertion_context.codebase),
            );

            var_type = artifacts.get_expression_type(&right.span());
            other_value_type = artifacts.get_expression_type(&left.span());
        }
    }

    if let Some(var_name) = var_name
        && let Some(other_value_type) = other_value_type
    {
        if other_value_type.is_single() {
            let orred_types = if operator.is_identity() {
                vec![Assertion::IsNotIdentical(other_value_type.get_single().clone())]
            } else {
                vec![Assertion::IsNotEqual(other_value_type.get_single().clone())]
            };

            if_types.insert(var_name, vec![orred_types]);
        }

        if let Some(other_value_var_name) = other_value_var_name
            && let Some(var_type) = var_type
            && !var_type.is_mixed()
            && var_type.is_single()
        {
            let orred_types = if operator.is_identity() {
                vec![Assertion::IsNotIdentical(var_type.get_single().clone())]
            } else {
                vec![Assertion::IsNotEqual(var_type.get_single().clone())]
            };

            if_types.insert(other_value_var_name, vec![orred_types]);
        }
    }

    if !if_types.is_empty() { vec![if_types] } else { vec![] }
}

#[inline]
pub const fn has_null_variable(left: &Expression, right: &Expression) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::Null(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::Null(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    None
}

#[inline]
pub const fn has_false_variable(left: &Expression, right: &Expression) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::False(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::False(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    None
}

#[inline]
pub const fn has_true_variable(left: &Expression, right: &Expression) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::True(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::True(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    None
}

#[inline]
pub fn has_empty_array_variable(left: &Expression, right: &Expression) -> Option<OtherValuePosition> {
    match unwrap_expression(right) {
        Expression::Array(array) if array.elements.is_empty() => {
            return Some(OtherValuePosition::Right);
        }
        Expression::LegacyArray(legacy_array) if legacy_array.elements.is_empty() => {
            return Some(OtherValuePosition::Right);
        }
        _ => {}
    }

    match unwrap_expression(left) {
        Expression::Array(array) if array.elements.is_empty() => {
            return Some(OtherValuePosition::Left);
        }
        Expression::LegacyArray(legacy_array) if legacy_array.elements.is_empty() => {
            return Some(OtherValuePosition::Left);
        }
        _ => {}
    }

    None
}
