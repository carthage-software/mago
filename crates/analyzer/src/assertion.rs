use ahash::HashMap;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::assertion_set::negate_assertion_set;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::assertion::Assertion;
use mago_codex::get_class_like;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

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
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();

    if let Some(var_name) = get_expression_id(
        expression,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    ) {
        if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
    }

    match unwrap_expression(expression) {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Not(_), operand }) => {
            let assertions = scrape_assertions(operand, artifacts, assertion_context);
            let mut negated_assertions = HashMap::default();
            for assertion in assertions {
                for (var_name, assertion_set) in assertion {
                    negated_assertions
                        .entry(var_name)
                        .or_insert_with(Vec::new)
                        .extend(negate_assertion_set(assertion_set));
                }
            }

            return if negated_assertions.is_empty() { vec![] } else { vec![negated_assertions] };
        }
        Expression::Call(call) => {
            // Collect `@assert` assertions.
            if_types.extend(process_custom_assertions(call.span(), artifacts));

            match call {
                // If the function does not have any, try collecting
                // assertions for special functions.
                Call::Function(function_call) if if_types.is_empty() => {
                    if_types.extend(scrape_special_function_call_assertions(
                        assertion_context,
                        artifacts,
                        function_call,
                    ));
                }
                // If its a null-safe method call, assert that
                // the lhs is non-null.
                Call::NullSafeMethod(null_safe_method_call) => {
                    let object_var_id = get_expression_id(
                        null_safe_method_call.object,
                        assertion_context.this_class_name,
                        assertion_context.resolved_names,
                        Some(assertion_context.codebase),
                    );

                    if let Some(object_var_id) = object_var_id {
                        if_types.insert(object_var_id, vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
                    }
                }
                _ => {}
            }
        }
        Expression::Construct(construct) => match construct {
            Construct::Empty(empty_construct) => {
                let Some(value_id) = get_expression_id(
                    empty_construct.value,
                    assertion_context.this_class_name,
                    assertion_context.resolved_names,
                    Some(assertion_context.codebase),
                ) else {
                    return vec![];
                };

                if let Expression::Variable(variable) = empty_construct.value
                    && let Some(expression_type) = artifacts.get_expression_type(variable)
                    && !expression_type.is_mixed()
                    && !expression_type.possibly_undefined
                {
                    if_types.insert(value_id, vec![vec![Assertion::Falsy]]);
                } else {
                    if_types.insert(value_id, vec![vec![Assertion::Empty]]);
                }
            }
            Construct::Isset(isset_construct) => {
                for value in isset_construct.values.iter() {
                    if let Some(value_id) = get_expression_id(
                        value,
                        assertion_context.this_class_name,
                        assertion_context.resolved_names,
                        Some(assertion_context.codebase),
                    ) {
                        if let Expression::Variable(variable) = value
                            && let Some(expression_type) = artifacts.get_expression_type(variable)
                            && !expression_type.is_mixed()
                            && !expression_type.possibly_undefined
                            && !expression_type.possibly_undefined_from_try
                        {
                            if_types.insert(value_id, vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
                        } else {
                            if_types.insert(value_id, vec![vec![Assertion::IsIsset]]);
                        }
                    } else {
                        let mut root_array_id = None;
                        let mut root_array: &Expression = value;
                        while let (None, Expression::ArrayAccess(array_access)) = (root_array_id.as_ref(), root_array) {
                            root_array = array_access.array;

                            root_array_id = get_expression_id(
                                root_array,
                                assertion_context.this_class_name,
                                assertion_context.resolved_names,
                                Some(assertion_context.codebase),
                            );
                        }

                        if let Some(root_array_id) = root_array_id {
                            if_types.insert(root_array_id, vec![vec![Assertion::IsEqualIsset]]);
                        }
                    }
                }
            }
            _ => {}
        },
        Expression::Binary(binary) => match binary.operator {
            BinaryOperator::Equal(_) | BinaryOperator::Identical(_) => {
                return scrape_equality_assertions(
                    binary.lhs,
                    binary.operator.is_identity(),
                    binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::NotEqual(_) | BinaryOperator::NotIdentical(_) | BinaryOperator::AngledNotEqual(_) => {
                return scrape_inequality_assertions(
                    binary.lhs,
                    &binary.operator,
                    binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::NullCoalesce(_) => {
                let rhs = unwrap_expression(binary.rhs);
                if matches!(rhs, Expression::Literal(Literal::Null(_))) {
                    let var_name = get_expression_id(
                        binary.lhs,
                        assertion_context.this_class_name,
                        assertion_context.resolved_names,
                        Some(assertion_context.codebase),
                    );

                    if let Some(var_name) = var_name {
                        if_types.insert(var_name, vec![vec![Assertion::IsIsset]]);
                    }
                }
            }
            BinaryOperator::GreaterThan(_) | BinaryOperator::GreaterThanOrEqual(_) => {
                return scrape_greater_than_assertions(
                    binary.lhs,
                    &binary.operator,
                    binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::LessThan(_) | BinaryOperator::LessThanOrEqual(_) => {
                return scrape_lesser_than_assertions(
                    binary.lhs,
                    &binary.operator,
                    binary.rhs,
                    artifacts,
                    assertion_context,
                );
            }
            BinaryOperator::Instanceof(_) => {
                return scrape_instanceof_assertions(binary.lhs, binary.rhs, artifacts, assertion_context);
            }
            _ => {}
        },
        Expression::Access(Access::NullSafeProperty(null_safe_property_access)) => {
            let object_var_id = get_expression_id(
                null_safe_property_access.object,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
                Some(assertion_context.codebase),
            );

            if let Some(object_var_id) = object_var_id {
                if_types.insert(object_var_id, vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
            }
        }
        _ => {}
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn process_custom_assertions(expression_span: Span, artifacts: &AnalysisArtifacts) -> HashMap<String, AssertionSet> {
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
            .extend(negate_assertion_set(if_false_assertion.1));
    }

    if_true_assertions
}

fn scrape_special_function_call_assertions(
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &AnalysisArtifacts,
    function_call: &FunctionCall,
) -> HashMap<String, AssertionSet> {
    let mut if_types = HashMap::default();

    let Expression::Identifier(function_identifier) = function_call.function else {
        return if_types;
    };

    let resolved_function_name = assertion_context.resolved_names.get(function_identifier);

    let (argument_variable_id_position, function_assertion) = match ascii_lowercase_atom(resolved_function_name)
        .as_str()
    {
        "psl\\iter\\contains_key" => {
            if let Some(array_key) = function_call
                .argument_list
                .arguments
                .get(1)
                .map(|argument| argument.value())
                .and_then(|array_key| get_expression_array_key(artifacts, array_key))
            {
                (0, Assertion::HasArrayKey(array_key))
            } else {
                return if_types;
            }
        }
        "array_key_exists" | "key_exists" => {
            if let Some(array_key) = function_call
                .argument_list
                .arguments
                .first()
                .map(|argument| argument.value())
                .and_then(|array_key| get_expression_array_key(artifacts, array_key))
            {
                (1, Assertion::HasArrayKey(array_key))
            } else {
                return if_types;
            }
        }
        "is_countable" => (0, Assertion::Countable),
        "ctype_digit" => (
            0,
            Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(true, false, false, false)))),
        ),
        "ctype_lower" => (
            0,
            Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(false, false, true, true)))),
        ),
        _ => return if_types,
    };

    let extract_expression_id = |argument_expression| {
        get_expression_id(
            argument_expression,
            assertion_context.this_class_name,
            assertion_context.resolved_names,
            Some(assertion_context.codebase),
        )
    };

    if let Some(first_argument_variable_id) = function_call
        .argument_list
        .arguments
        .get(argument_variable_id_position)
        .map(|argument| argument.value())
        .and_then(extract_expression_id)
    {
        if_types.insert(first_argument_variable_id, vec![vec![function_assertion]]);
    }

    if_types
}

pub(super) fn scrape_equality_assertions(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
    if let Some(assertions) = scrape_class_constant_equality_assertions(
        left,
        right,
        artifacts,
        assertion_context,
        false, // negated = false
    ) {
        return assertions;
    }

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

    if let Some(null_position) = has_null_variable(left, right, artifacts) {
        return get_null_equality_assertions(left, right, assertion_context, null_position);
    }

    if let Some(true_position) = has_true_variable(left, right, artifacts) {
        return get_true_equality_assertions(left, is_identity, right, artifacts, assertion_context, true_position);
    }

    if let Some(false_position) = has_false_variable(left, right, artifacts) {
        return get_false_equality_assertions(left, is_identity, right, assertion_context, false_position);
    }

    if let Some(empty_array_position) = has_empty_array_variable(left, right) {
        return get_empty_array_equality_assertions(left, is_identity, right, assertion_context, empty_array_position);
    }

    if let Some(enum_case_position) = has_enum_case_comparison(left, right, artifacts) {
        return get_enum_case_equality_assertions(left, right, assertion_context, artifacts, enum_case_position);
    }

    if let Some(typed_value_position) = has_typed_value_comparison(left, right, artifacts, assertion_context) {
        return get_typed_value_equality_assertions(
            left,
            is_identity,
            right,
            artifacts,
            assertion_context,
            typed_value_position,
        );
    }

    vec![]
}

fn scrape_inequality_assertions(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
    if let Some(assertions) = scrape_class_constant_equality_assertions(
        left,
        right,
        artifacts,
        assertion_context,
        true, // negated = true
    ) {
        return assertions;
    }

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

    if let Some(null_position) = has_null_variable(left, right, artifacts) {
        return get_null_inequality_assertions(left, right, assertion_context, null_position);
    }

    if let Some(false_position) = has_false_variable(left, right, artifacts) {
        return get_false_inquality_assertions(left, right, assertion_context, false_position);
    }

    if let Some(true_position) = has_true_variable(left, right, artifacts) {
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

    vec![]
}

/// Scrapes assertions for comparisons like `$foo::class === Bar::class`.
/// This is treated as equivalent to an `instanceof` check.
fn scrape_class_constant_equality_assertions(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
    negated: bool,
) -> Option<Vec<HashMap<String, AssertionSet>>> {
    let left_class_part = is_class_constant_access(left);
    let right_class_part = is_class_constant_access(right);

    let (variable_expr, class_name_expr) = match (left_class_part, right_class_part) {
        // Case 1: Both sides are `::class` expressions (e.g., `$var::class === Foo::class`)
        (Some(left_part), Some(right_part)) => {
            let left_is_static = is_static_class_reference(left_part);
            let right_is_static = is_static_class_reference(right_part);

            if !left_is_static && right_is_static {
                // $var::class === Foo::class  =>  $var is the variable, Foo::class is the type
                (left_part, right)
            } else if left_is_static && !right_is_static {
                // Foo::class === $var::class  =>  $var is the variable, Foo::class is the type
                (right_part, left)
            } else {
                // Both are dynamic ($a::class === $b::class) or both static (A::class === B::class).
                // Let the standard reconciler handle these comparisons.
                return None;
            }
        }
        // Case 2: Only the left side is `::class`
        (Some(part), None) => (part, right),
        // Case 3: Only the right side is `::class`
        (None, Some(part)) => (part, left),
        // Case 4: Neither side is `::class`
        (None, None) => return None,
    };

    let variable_id = get_expression_id(
        variable_expr,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    )?;

    let class_name_type = artifacts.get_expression_type(class_name_expr)?;

    let mut assertions = vec![];
    for atomic in class_name_type.types.iter() {
        if let Some(resolved_class) = get_class_name_from_atomic(assertion_context.codebase, atomic) {
            let object_type = resolved_class.get_object_type(assertion_context.codebase);

            assertions.push(if negated {
                if resolved_class.is_final {
                    Assertion::IsNotType(object_type)
                } else {
                    Assertion::IsNotIdentical(object_type)
                }
            } else if resolved_class.is_final {
                Assertion::IsType(object_type)
            } else {
                Assertion::IsIdentical(object_type)
            });
        }
    }

    if assertions.is_empty() {
        return None;
    }

    let mut if_types = HashMap::default();
    if_types.insert(variable_id, vec![assertions]);
    Some(vec![if_types])
}

/// Helper to check if an expression is a `::class` constant access.
/// Returns the expression for the class part (e.g., `$foo` in `$foo::class`).
#[inline]
fn is_class_constant_access<'arena>(expr: &'arena Expression<'arena>) -> Option<&'arena Expression<'arena>> {
    if let Expression::Access(Access::ClassConstant(ClassConstantAccess {
        class,
        constant: ClassLikeConstantSelector::Identifier(LocalIdentifier { value: "class", .. }),
        ..
    })) = unwrap_expression(expr)
    {
        Some(class)
    } else {
        None
    }
}

/// Helper to determine if the class part of a `::class` expression is a static reference.
#[inline]
fn is_static_class_reference(expr: &Expression) -> bool {
    matches!(
        unwrap_expression(expr),
        Expression::Identifier(_) | Expression::Self_(_) | Expression::Static(_) | Expression::Parent(_)
    )
}

fn get_empty_array_equality_assertions(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if is_identity {
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
    assertion_context: AssertionContext<'_, '_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
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
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
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
    assertion_context: AssertionContext<'_, '_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    assertion_context: AssertionContext<'_, '_>,
    null_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    assertion_context: AssertionContext<'_, '_>,
    false_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    assertion_context: AssertionContext<'_, '_>,
    true_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
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
                    if_types.insert(
                        array_variable_id,
                        vec![vec![Assertion::DoesNotHasAtLeastCount(maximum_count as usize)]],
                    );
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
        return vec![];
    }

    let mut if_types = HashMap::default();

    let left_id = get_expression_id(
        left,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    );

    let right_id = get_expression_id(
        right,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
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
                    if_types.insert(
                        array_variable_id,
                        vec![vec![Assertion::DoesNotHasAtLeastCount(maximum_count as usize)]],
                    );
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
        return vec![];
    }

    let mut if_types = HashMap::default();

    // Generate assertions for the left variable based on the right variable's type.
    // For an expression `$a > $b`, this asserts `$a` is greater than the lower bound of `$b`.
    if let Some(right_int) = &right_integer
        && let Some(left_var_id) = get_expression_id(
            left,
            assertion_context.this_class_name,
            assertion_context.resolved_names,
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
    artifacts: &AnalysisArtifacts,
    context: AssertionContext<'_, '_>,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();

    let variable_id = get_expression_id(left, context.this_class_name, context.resolved_names, Some(context.codebase));

    if let Some(counter_variable_id) = variable_id {
        match right {
            Expression::Identifier(identifier) => {
                let resolved_name = context.resolved_names.get(identifier);

                if_types.insert(
                    counter_variable_id,
                    vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
                        resolved_name,
                    )))))]],
                );
            }
            Expression::Self_(_) => {
                if let Some(self_class) = context.this_class_name {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new_this(
                            self_class,
                        ))))]],
                    );
                }
            }
            Expression::Static(_) => {
                if let Some(self_class) = context.this_class_name {
                    if_types.insert(
                        counter_variable_id,
                        vec![vec![Assertion::IsIdentical(TAtomic::Object(TObject::Named(TNamedObject::new_this(
                            self_class,
                        ))))]],
                    );
                }
            }
            Expression::Parent(_) => {
                if let Some(self_class) = context.this_class_name
                    && let Some(self_meta) = get_class_like(context.codebase, &self_class)
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
                    for atomic in expression_type.types.as_ref() {
                        let Some(name) = get_class_name_from_atomic(context.codebase, atomic) else {
                            continue;
                        };

                        assertions.push(Assertion::IsType(name.get_object_type(context.codebase)));
                    }

                    // If we failed to resolve the class-name on the rhs of
                    // `instanceof`, assert that the lhs is a generic `object`.
                    if assertions.is_empty() && !expression_type.is_objecty() {
                        assertions.push(Assertion::IsType(TAtomic::Object(TObject::Any)));
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
    assertion_context: AssertionContext<'_, '_>,
) -> (Option<i64>, Option<i64>) {
    if is_count_or_size_of_call(left, assertion_context) {
        (None, get_expression_integer_value(artifacts, right).and_then(|integer| integer.get_literal_value()))
    } else if is_count_or_size_of_call(right, assertion_context) {
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

fn get_expression_array_key(artifacts: &AnalysisArtifacts, expression: &Expression) -> Option<ArrayKey> {
    artifacts.get_expression_type(expression).and_then(|t| t.get_single_array_key())
}

fn is_count_or_size_of_call(expression: &Expression, assertion_context: AssertionContext<'_, '_>) -> bool {
    let Expression::Call(Call::Function(FunctionCall { function, argument_list })) = expression else {
        return false;
    };

    if argument_list.arguments.len() != 1 {
        return false;
    }

    let Expression::Identifier(function_identifier) = function else {
        return false;
    };

    let resolved_function_name = assertion_context.resolved_names.get(function_identifier);

    resolved_function_name.eq_ignore_ascii_case("count") || resolved_function_name.eq_ignore_ascii_case("sizeof")
}

fn get_true_equality_assertions(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
    true_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if is_identity {
            if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::r#true()))]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
        }

        vec![if_types]
    } else {
        // If we can't get an expression ID, we can still assert that the expression is truthy.
        scrape_assertions(base_conditional, artifacts, assertion_context)
    }
}

pub fn has_typed_value_comparison(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
) -> Option<OtherValuePosition> {
    let left_var_id = get_expression_id(
        left,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    );

    let right_var_id = get_expression_id(
        right,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
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
    is_identity: bool,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    false_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
    let mut if_types = HashMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = get_expression_id(
        base_conditional,
        assertion_context.this_class_name,
        assertion_context.resolved_names,
        Some(assertion_context.codebase),
    );

    if let Some(var_name) = var_name {
        if is_identity {
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
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_>,
    typed_value_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
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
                Some(assertion_context.codebase),
            );

            other_value_var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
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
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
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

        let orred_types = if is_identity {
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
        let orred_types = if is_identity {
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
    assertion_context: AssertionContext<'_, '_>,
    typed_value_position: OtherValuePosition,
) -> Vec<HashMap<String, AssertionSet>> {
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
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                right,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
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
                Some(assertion_context.codebase),
            );
            other_value_var_name = get_expression_id(
                left,
                assertion_context.this_class_name,
                assertion_context.resolved_names,
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
    assertion_context: AssertionContext<'_, '_>,
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
pub fn has_null_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::Null(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Some(right_type) = artifacts.get_expression_type(right)
        && right_type.is_null()
    {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::Null(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    if let Some(left_type) = artifacts.get_expression_type(left)
        && left_type.is_null()
    {
        return Some(OtherValuePosition::Left);
    }

    None
}

#[inline]
pub fn has_false_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::False(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Some(right_type) = artifacts.get_expression_type(right)
        && right_type.is_false()
    {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::False(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    if let Some(left_type) = artifacts.get_expression_type(left)
        && left_type.is_false()
    {
        return Some(OtherValuePosition::Left);
    }

    None
}

#[inline]
pub fn has_true_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    if let Expression::Literal(Literal::True(_)) = unwrap_expression(right) {
        return Some(OtherValuePosition::Right);
    }

    if let Some(right_type) = artifacts.get_expression_type(right)
        && right_type.is_true()
    {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Literal(Literal::True(_)) = unwrap_expression(left) {
        return Some(OtherValuePosition::Left);
    }

    if let Some(left_type) = artifacts.get_expression_type(left)
        && left_type.is_true()
    {
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
