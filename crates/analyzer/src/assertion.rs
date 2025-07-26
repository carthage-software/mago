use ahash::HashMap;

use mago_codex::get_class_like;
use mago_codex::ttype::atomic::scalar::int::TInteger;
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
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, left) {
                if number_on_right == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                } else {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasExactCount(number_on_right as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (Some(number_on_left), None) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, right) {
                if number_on_left == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                } else {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasExactCount(number_on_left as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        _ => {
            // Continue to check for other conditions
        }
    };

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

    if let Some(enum_case_position) = has_enum_case_comparison(left, right, artifacts) {
        return get_enum_case_equality_assertions(left, right, assertion_context, artifacts, enum_case_position);
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
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, left) {
                if number_on_right == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                } else {
                    if_types.insert(
                        array_variable_id,
                        vec![vec![Assertion::DoesNotHaveExactCount(number_on_right as usize)]],
                    );
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (Some(number_on_left), None) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, right) {
                if number_on_left == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                } else {
                    if_types.insert(
                        array_variable_id,
                        vec![vec![Assertion::DoesNotHaveExactCount(number_on_left as usize)]],
                    );
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        _ => {
            // Continue to check for other conditions
        }
    };

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

    if let Some(enum_case_position) = has_enum_case_comparison(left, right, artifacts) {
        return get_enum_case_inequality_assertions(left, right, assertion_context, artifacts, enum_case_position);
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

fn get_enum_case_equality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let (variable_expression, Some(enum_case_type)) = (match enum_case_position {
        OtherValuePosition::Left => (right, artifacts.get_expression_type(left)),
        OtherValuePosition::Right => (left, artifacts.get_expression_type(right)),
    }) else {
        return vec![];
    };

    let mut if_types = HashMap::default();

    let var_name = get_expression_id(
        variable_expression,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsType(enum_case_type.clone().get_single_owned())]]);
    }

    vec![if_types]
}

fn get_enum_case_inequality_assertions(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<HashMap<String, Vec<Vec<Assertion>>>> {
    let (variable_expression, Some(enum_case_type)) = (match enum_case_position {
        OtherValuePosition::Left => (right, artifacts.get_expression_type(left)),
        OtherValuePosition::Right => (left, artifacts.get_expression_type(right)),
    }) else {
        return vec![];
    };

    let mut if_types = HashMap::default();

    let var_name = get_expression_id(
        variable_expression,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsNotType(enum_case_type.clone().get_single_owned())]]);
    }

    vec![if_types]
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
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, left) {
                let maximum_count = if matches!(operator, BinaryOperator::LessThan(_)) {
                    number_on_right.saturating_sub(1)
                } else {
                    number_on_right
                };

                if maximum_count < 0 {
                    // This branch is logically unreachable, e.g. `count($arr) < 0`.
                } else if maximum_count == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                } else {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasAtMostCount(maximum_count as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (Some(number_on_left), None) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, right) {
                let minimum_count = if matches!(operator, BinaryOperator::LessThan(_)) {
                    number_on_left.saturating_add(1)
                } else {
                    number_on_left
                };

                if minimum_count == 1 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                } else if minimum_count > 1 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasAtLeastCount(minimum_count as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        _ => {
            // Not a count comparison, so we proceed to the main logic.
        }
    }

    let (left_integer, right_integer) = get_comparison_literal_operand(artifacts, left, right);

    if left_integer.is_none() && right_integer.is_none() {
        return Vec::new();
    }

    let mut if_types = HashMap::default();

    let left_id = get_expression_id(
        left,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    let right_id = get_expression_id(
        right,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    );

    // Generate assertions for the left variable based on the right variable's type.
    // For an expression `$a < $b`, this asserts `$a` is less than the upper bound of `$b`.
    if let (Some(left_var_id), Some(right_int)) = (left_id, &right_integer) {
        let assertion_result = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThanOrEqual(count), count)),
                TInteger::To(upper_bound) => Some((Assertion::IsLessThanOrEqual(upper_bound), upper_bound)),
                TInteger::Range(_, upper_bound) => Some((Assertion::IsLessThanOrEqual(upper_bound), upper_bound)),
                _ => None,
            }
        } else {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThan(count), count)),
                TInteger::To(upper_bound) => Some((Assertion::IsLessThan(upper_bound), upper_bound)),
                TInteger::Range(_, upper_bound) => Some((Assertion::IsLessThan(upper_bound), upper_bound)),
                _ => None,
            }
        };

        if let Some((assertion, bound)) = assertion_result {
            let mut is_redundant = false;
            if !right_int.is_literal()
                && let Some(left_int) = &left_integer
                && let Some(max_val) = left_int.get_maximum_value()
            {
                is_redundant = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
                    max_val <= bound
                } else {
                    max_val < bound
                };
            }

            if !is_redundant {
                if_types.insert(left_var_id, vec![vec![assertion]]);
            }
        }
    }

    // Generate assertions for the right variable based on the left variable's type.
    // For an expression `$a < $b`, this asserts `$b` is greater than the lower bound of `$a`.
    if let (Some(right_var_id), Some(left_int)) = (right_id, &left_integer) {
        let assertion_result = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThanOrEqual(count), count)),
                TInteger::From(lower_bound) => Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound)),
                TInteger::Range(lower_bound, _) => Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound)),
                _ => None,
            }
        } else {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThan(count), count)),
                TInteger::From(lower_bound) => Some((Assertion::IsGreaterThan(lower_bound), lower_bound)),
                TInteger::Range(lower_bound, _) => Some((Assertion::IsGreaterThan(lower_bound), lower_bound)),
                _ => None,
            }
        };

        if let Some((assertion, bound)) = assertion_result {
            let mut is_redundant = false;
            if !left_int.is_literal()
                && let Some(right_int) = &right_integer
                && let Some(min_val) = right_int.get_minimum_value()
            {
                is_redundant = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
                    min_val >= bound
                } else {
                    min_val > bound
                };
            }

            if !is_redundant {
                if_types.insert(right_var_id, vec![vec![assertion]]);
            }
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
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, left) {
                let minimum_count = if matches!(operator, BinaryOperator::GreaterThan(_)) {
                    number_on_right.saturating_add(1)
                } else {
                    number_on_right
                };

                if minimum_count == 1 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::NonEmptyCountable(false)]]);
                } else if minimum_count > 1 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasAtLeastCount(minimum_count as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (Some(number_on_left), None) => {
            let mut if_types = HashMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, right) {
                let maximum_count = if matches!(operator, BinaryOperator::GreaterThan(_)) {
                    number_on_left.saturating_sub(1)
                } else {
                    number_on_left
                };

                if maximum_count < 0 {
                    // This branch is logically unreachable, e.g. `-1 > count($arr)`.
                } else if maximum_count == 0 {
                    if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                } else {
                    if_types.insert(array_variable_id, vec![vec![Assertion::HasAtMostCount(maximum_count as usize)]]);
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        _ => {
            // Not a count comparison, so we proceed to the main logic.
        }
    }

    let (left_integer, right_integer) = get_comparison_literal_operand(artifacts, left, right);

    if left_integer.is_none() && right_integer.is_none() {
        return Vec::new();
    }

    let mut if_types = HashMap::default();

    // Generate assertions for the left variable based on the right variable's type.
    // For an expression `$a > $b`, this asserts `$a` is greater than the lower bound of `$b`.
    if let Some(right_int) = &right_integer
        && let Some(left_var_id) = get_expression_id(
            left,
            assertion_context.this_class_name,
            assertion_context.resolved_names,
            assertion_context.interner,
            Some(assertion_context.codebase),
        )
    {
        let assertion_result = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThanOrEqual(count), count)),
                TInteger::From(lower_bound) => Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound)),
                TInteger::Range(lower_bound, _) => Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound)),
                _ => None,
            }
        } else {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThan(count), count)),
                TInteger::From(lower_bound) => Some((Assertion::IsGreaterThan(lower_bound), lower_bound)),
                TInteger::Range(lower_bound, _) => Some((Assertion::IsGreaterThan(lower_bound), lower_bound)),
                _ => None,
            }
        };

        if let Some((assertion, bound)) = assertion_result {
            let mut is_redundant = false;
            if !right_int.is_literal()
                && let Some(left_int) = &left_integer
                && let Some(min_val) = left_int.get_minimum_value()
            {
                is_redundant = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
                    min_val >= bound
                } else {
                    min_val > bound
                };
            }

            if !is_redundant {
                if_types.insert(left_var_id, vec![vec![assertion]]);
            }
        }
    }

    // Generate assertions for the right variable based on the left variable's type.
    // For an expression `$a > $b`, this asserts `$b` is less than the upper bound of `$a`.
    if let Some(left_int) = &left_integer
        && let Some(right_var_id) = get_expression_id(
            right,
            assertion_context.this_class_name,
            assertion_context.resolved_names,
            assertion_context.interner,
            Some(assertion_context.codebase),
        )
    {
        let assertion_result = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThanOrEqual(count), count)),
                TInteger::To(upper_bound) => Some((Assertion::IsLessThanOrEqual(upper_bound), upper_bound)),
                TInteger::Range(_, upper_bound) => Some((Assertion::IsLessThanOrEqual(upper_bound), upper_bound)),
                _ => None,
            }
        } else {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThan(count), count)),
                TInteger::To(upper_bound) => Some((Assertion::IsLessThan(upper_bound), upper_bound)),
                TInteger::Range(_, upper_bound) => Some((Assertion::IsLessThan(upper_bound), upper_bound)),
                _ => None,
            }
        };

        if let Some((assertion, bound)) = assertion_result {
            let mut is_redundant = false;
            if !left_int.is_literal()
                && let Some(right_int) = &right_integer
                && let Some(max_val) = right_int.get_maximum_value()
            {
                is_redundant = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
                    max_val <= bound
                } else {
                    max_val < bound
                };
            }

            if !is_redundant {
                if_types.insert(right_var_id, vec![vec![assertion]]);
            }
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
                    && let Some(parent_id_ref) = self_meta.direct_parent_class.as_ref()
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

/// Checks if a binary operation is a comparison between a `count()` or `sizeof()`
/// call and an integer literal. It looks for `<`, `<=`, `>`, and `>=` operators.
///
/// # Returns
///
/// A tuple `(Option<i64>, Option<i64>)`.
///
/// If the `count()` call is on the left, it returns `(None, Some(right_value))`.
/// If the `count()` call is on the right, it returns `(Some(left_value), None)`.
///
/// If the expression is not a size comparison, or the other operand is not an
/// integer literal, it returns `(None, None)`. The returned tuple will never
/// contain a value for both the left and right sides.
fn resolve_count_comparison(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_>,
) -> (Option<i64>, Option<i64>) {
    if is_count_or_size_of_call(assertion_context, left) {
        (None, get_expression_integer_value(artifacts, right).and_then(|integer| integer.get_literal_value()))
    } else if is_count_or_size_of_call(assertion_context, right) {
        (get_expression_integer_value(artifacts, left).and_then(|integer| integer.get_literal_value()), None)
    } else {
        (None, None)
    }
}

fn get_comparison_literal_operand(
    artifacts: &AnalysisArtifacts,
    left: &Expression,
    right: &Expression,
) -> (Option<TInteger>, Option<TInteger>) {
    (get_expression_integer_value(artifacts, left), get_expression_integer_value(artifacts, right))
}

fn get_expression_integer_value(artifacts: &AnalysisArtifacts, expression: &Expression) -> Option<TInteger> {
    artifacts
        .get_expression_type(expression)
        .and_then(|t| t.get_single_int())
        .filter(|integer| !integer.is_unspecified())
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

        let orred_types = if operator.is_identity() {
            vec![Assertion::IsIdentical(other_value_atomic)]
        } else {
            vec![Assertion::IsEqual(other_value_atomic)]
        };

        if_types.insert(var_name, vec![orred_types]);
    } else if let Some(other_value_var_name) = other_value_var_name
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
fn get_first_argument_expression_id(
    assertion_context: AssertionContext<'_>,
    expression: &Expression,
) -> Option<String> {
    let Expression::Call(Call::Function(FunctionCall { argument_list, .. })) = expression else {
        return None;
    };

    if argument_list.arguments.len() != 1 {
        return None;
    }

    get_expression_id(
        argument_list.arguments.first()?.value(),
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        assertion_context.interner,
        Some(assertion_context.codebase),
    )
}

#[inline]
pub fn has_enum_case_comparison(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    if let Expression::Access(Access::ClassConstant(class_constant_access)) = unwrap_expression(right)
        && artifacts
            .get_expression_type(class_constant_access)
            .is_some_and(|expression_type| expression_type.is_single_enum_case())
    {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Access(Access::ClassConstant(class_constant_access)) = unwrap_expression(left)
        && artifacts
            .get_expression_type(class_constant_access)
            .is_some_and(|expression_type| expression_type.is_single_enum_case())
    {
        return Some(OtherValuePosition::Left);
    }

    None
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
