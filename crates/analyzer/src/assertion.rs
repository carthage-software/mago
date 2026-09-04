use mago_allocator::Arena;
use std::cmp::Ordering;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::assertion_set::negate_assertion_set;
use mago_codex::assertion::Assertion;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringCasing;
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::get_array_value_parameter;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_iterable_value_parameter;
use mago_codex::ttype::get_numeric_string;
use mago_codex::ttype::get_true;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Access;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Call;
use mago_syntax::cst::ClassConstantAccess;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Expression;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Literal;
use mago_syntax::cst::LocalIdentifier;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::i64_word;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::context::assertion::AssertionContext;
use crate::resolver::class_name::get_class_name_from_atomic;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_index_id;
use crate::utils::expression::get_non_nullsafe_expression_id;
use crate::utils::misc::unwrap_expression;

#[derive(Debug, Clone, Copy)]
pub enum OtherValuePosition {
    Left,
    Right,
}

pub fn scrape_assertions<A>(
    mut expression: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    expression = unwrap_expression(expression);

    let mut if_types = WordMap::default();

    if let Some(var_name) = assertion_context.get_expression_id(expression) {
        if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
    }

    match expression {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Not(_), operand }) => {
            let assertions = scrape_assertions(operand, artifacts, assertion_context);
            let mut negated_assertions = WordMap::default();
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
                _ => {}
            }

            if let Call::Function(FunctionCall { function: _, argument_list }) = call
                && 1 == argument_list.arguments.len()
                && let Some(first_argument) = argument_list.arguments.first()
                && let Some(first_argument_expression_id) = assertion_context.get_expression_id(first_argument.value())
            {
                if is_count_or_size_of_call(expression, assertion_context) {
                    if_types.insert(first_argument_expression_id, vec![vec![Assertion::NonEmptyCountable(true)]]);
                } else if is_function_call_to(expression, assertion_context, b"function_exists") {
                    if_types.insert(
                        first_argument_expression_id,
                        vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::callable())))]],
                    );
                } else if is_function_call_to(expression, assertion_context, b"is_callable")
                    && let Some(first_argument_type) = artifacts.get_expression_type(first_argument.value())
                {
                    let mut callables = vec![];

                    let mut add_callable_assertion = |atomic: &TAtomic| {
                        if let Some(callable) = cast_atomic_to_callable(atomic, assertion_context.codebase, None) {
                            callables.push(Assertion::IsType(TAtomic::Callable(callable.into_owned())));
                        } else if let TAtomic::Scalar(TScalar::String(string)) = atomic {
                            callables.push(Assertion::IsType(TAtomic::Scalar(TScalar::String(string.as_callable()))));
                        }
                    };

                    for argument_type_atomic in first_argument_type.types.as_ref() {
                        if let TAtomic::GenericParameter(generic_parameter) = argument_type_atomic {
                            for constrained_atomic in generic_parameter.constraint.types.as_ref() {
                                add_callable_assertion(constrained_atomic);
                            }

                            continue;
                        }

                        add_callable_assertion(argument_type_atomic);
                    }

                    if !callables.is_empty() {
                        if_types.insert(first_argument_expression_id, vec![callables]);
                    }
                }
            }
        }
        Expression::Construct(construct) => match construct {
            Construct::Empty(empty_construct) => {
                let Some(value_id) = assertion_context.get_expression_id(empty_construct.value) else {
                    return vec![];
                };

                if let Expression::Variable(variable) = empty_construct.value
                    && let Some(expression_type) = artifacts.get_expression_type(variable)
                    && !expression_type.is_mixed()
                    && !expression_type.possibly_undefined()
                {
                    if_types.insert(value_id, vec![vec![Assertion::Falsy]]);
                } else {
                    if_types.insert(value_id, vec![vec![Assertion::Empty]]);
                }
            }
            Construct::Isset(isset_construct) => {
                for value in &isset_construct.values {
                    if let Some(value_id) = assertion_context.get_expression_id(value) {
                        if let Expression::Variable(variable) = value
                            && let Some(expression_type) = artifacts.get_expression_type(variable)
                            && !expression_type.is_mixed()
                            && !expression_type.possibly_undefined()
                            && !expression_type.possibly_undefined_from_try()
                        {
                            if_types.entry(value_id).or_insert_with(|| vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
                        } else {
                            if_types.entry(value_id).or_insert_with(|| vec![vec![Assertion::IsIsset]]);
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
                    let var_name = assertion_context.get_expression_id(binary.lhs);

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
        _ => {}
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn process_custom_assertions(expression_span: Span, artifacts: &AnalysisArtifacts) -> WordMap<AssertionSet> {
    let range = (expression_span.start.offset, expression_span.end.offset);
    let mut assertions = artifacts.if_true_assertions.get(&range).cloned().unwrap_or_default();

    assertions.retain(|_, assertion_set| {
        assertion_set.retain(|assertions| !assertions.iter().any(Assertion::has_equality));
        !assertion_set.is_empty()
    });

    if let Some(if_false_assertions) = artifacts.if_false_assertions.get(&range) {
        for (variable, assertion_set) in if_false_assertions {
            for assertion_group in assertion_set {
                if assertion_group.iter().any(Assertion::has_equality) {
                    continue;
                }

                assertions.entry(*variable).or_default().extend(negate_assertion_set(vec![assertion_group.clone()]));
            }
        }
    }

    assertions
}

fn scrape_special_function_call_assertions<A>(
    assertion_context: AssertionContext<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    function_call: &FunctionCall,
) -> WordMap<AssertionSet>
where
    A: Arena,
{
    let mut if_types = WordMap::default();

    let Expression::Identifier(function_identifier) = function_call.function else {
        return if_types;
    };

    let function_name = ascii_lowercase_word(function_identifier.value());
    let resolved_function_name = ascii_lowercase_word(assertion_context.resolved_names.get(function_identifier));
    let should_check_against_unresolved = { function_identifier.is_local() };

    let is_array_key_exists = |name: &[u8]| matches!(name, b"array_key_exists" | b"key_exists");
    if (should_check_against_unresolved && is_array_key_exists(function_name.as_bytes()))
        || is_array_key_exists(resolved_function_name.as_bytes())
    {
        let key_argument = function_call.argument_list.arguments.first().map(mago_syntax::cst::Argument::value);
        let array_argument = function_call.argument_list.arguments.get(1).map(mago_syntax::cst::Argument::value);

        if let (Some(key_argument), Some(array_argument)) = (key_argument, array_argument)
            && get_expression_array_key(artifacts, key_argument).is_none()
        {
            if let Some(array_id) = assertion_context.get_expression_id(array_argument)
                && let Some(index_id) = get_index_id(
                    key_argument,
                    assertion_context.this_class_name,
                    assertion_context.resolved_names,
                    Some(assertion_context.codebase),
                )
            {
                let access_id = concat_word!(array_id.as_bytes(), b"[", index_id.as_bytes(), b"]");
                if_types.insert(access_id, vec![vec![Assertion::ArrayKeyExists]]);
            }

            if let Some(key_id) = assertion_context.get_expression_id(key_argument)
                && let Some(array_type) = artifacts.get_expression_type(array_argument)
                && let Some(input_key_type) = artifacts.get_expression_type(key_argument)
                && let Some(key_type) =
                    get_possible_array_key_argument_type(array_type, input_key_type, assertion_context.codebase)
            {
                if_types.insert(key_id, vec![vec![Assertion::InArray(key_type)]]);
            }

            if !if_types.is_empty() {
                return if_types;
            }
        }
    }

    let (argument_variable_id_position, function_assertions) = if resolved_function_name
        .as_bytes()
        .starts_with(b"psl\\")
    {
        match resolved_function_name.as_bytes() {
            b"psl\\iter\\contains_key" => {
                if let Some(array_key) = function_call
                    .argument_list
                    .arguments
                    .get(1)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|array_key| get_expression_array_key(artifacts, array_key))
                {
                    (0, vec![Assertion::HasArrayKey(array_key)])
                } else {
                    return if_types;
                }
            }
            b"psl\\iter\\contains" => {
                let Some(checked_iterable) = function_call
                    .argument_list
                    .arguments
                    .get(0)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))
                else {
                    return if_types;
                };

                let mut iterable_values = None;
                for atomic in checked_iterable.types.as_ref() {
                    let Some(value_type) = get_iterable_value_parameter(atomic, assertion_context.codebase) else {
                        return if_types;
                    };

                    iterable_values =
                        Some(add_optional_union_type(value_type, iterable_values.as_ref(), assertion_context.codebase));
                }

                let Some(value_type) = iterable_values else {
                    return if_types;
                };

                let mut value_assertions = vec![];
                for atomic in value_type.types.into_owned() {
                    value_assertions.push(Assertion::IsEqual(atomic));
                }

                (1, value_assertions)
            }
            _ => return if_types,
        }
    } else {
        let get_builtin_assertion = |name: &[u8]| match name {
            b"array_key_exists" | b"key_exists" => function_call
                .argument_list
                .arguments
                .first()
                .map(mago_syntax::cst::Argument::value)
                .and_then(|array_key| get_expression_array_key(artifacts, array_key))
                .map(|key| (1, vec![Assertion::HasArrayKey(key)])),
            b"is_countable" => Some((0, vec![Assertion::Countable])),
            b"ctype_digit" => Some((
                0,
                vec![Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                    true,
                    false,
                    false,
                    false,
                    TStringCasing::Unspecified,
                ))))],
            )),
            b"ctype_lower" => Some((
                0,
                vec![Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                    false,
                    false,
                    true,
                    false,
                    TStringCasing::Lowercase,
                ))))],
            )),
            b"ctype_upper" => Some((
                0,
                vec![Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                    false,
                    false,
                    true,
                    false,
                    TStringCasing::Uppercase,
                ))))],
            )),
            b"count" => Some((0, vec![Assertion::HasAtLeastCount(1)])),
            b"function_exists" => {
                Some((0, vec![Assertion::IsType(TAtomic::Scalar(TScalar::String(TString::callable())))]))
            }
            b"method_exists" if assertion_context.trust_existence_checks => {
                let method_name = function_call
                    .argument_list
                    .arguments
                    .get(1)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))
                    .and_then(|ty| ty.get_single_literal_string_value())?;

                Some((0, vec![Assertion::IsType(TAtomic::Object(TObject::new_has_method(word(method_name))))]))
            }
            b"property_exists" if assertion_context.trust_existence_checks => {
                let property_name = function_call
                    .argument_list
                    .arguments
                    .get(1)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))
                    .and_then(|ty| ty.get_single_literal_string_value())?;

                Some((0, vec![Assertion::IsType(TAtomic::Object(TObject::new_has_property(word(property_name))))]))
            }
            b"is_a" | b"is_subclass_of" => {
                let class_name_type = function_call
                    .argument_list
                    .arguments
                    .get(1)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))?;

                let is_subclass_of_call = name == b"is_subclass_of";
                let allow_string = function_call
                    .argument_list
                    .arguments
                    .get(2)
                    .and_then(|argument| artifacts.get_expression_type(argument.value()))
                    .map_or(is_subclass_of_call, |t| !t.is_false());

                let subject_is_string_only = function_call
                    .argument_list
                    .arguments
                    .first()
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))
                    .is_some_and(|subject| {
                        !subject.types.is_empty()
                            && subject
                                .types
                                .iter()
                                .all(|atomic| matches!(atomic, TAtomic::Scalar(scalar) if scalar.is_any_string()))
                    });

                let mut assertions = vec![];
                for atomic in class_name_type.types.as_ref() {
                    let Some(resolved) = get_class_name_from_atomic(assertion_context.codebase, atomic) else {
                        continue;
                    };

                    let object_type = resolved.get_object_type(assertion_context.codebase);

                    if allow_string {
                        assertions.push(Assertion::IsType(TAtomic::Scalar(TScalar::class_string_of_type(
                            object_type.clone(),
                        ))));
                    }

                    if !(allow_string && subject_is_string_only) {
                        assertions.push(Assertion::IsType(object_type));
                    }
                }

                if assertions.is_empty() {
                    return None;
                }

                Some((0, assertions))
            }
            b"in_array" => {
                let should_strict_check = function_call
                    .argument_list
                    .arguments
                    .get(2)
                    .and_then(|argument| artifacts.get_expression_type(argument.value()))
                    .is_some_and(mago_codex::ttype::union::TUnion::is_true);

                if !should_strict_check {
                    return None;
                }

                let checked_array = function_call
                    .argument_list
                    .arguments
                    .get(1)
                    .map(mago_syntax::cst::Argument::value)
                    .and_then(|expr| artifacts.get_expression_type(expr))?;

                let mut value_types = None;
                for atomic in checked_array.types.as_ref() {
                    let TAtomic::Array(array) = atomic else {
                        return None;
                    };

                    value_types = Some(add_optional_union_type(
                        get_array_value_parameter(array, assertion_context.codebase),
                        value_types.as_ref(),
                        assertion_context.codebase,
                    ));
                }

                let mut value_assertions = vec![];
                for atomic in value_types?.types.into_owned() {
                    value_assertions.push(Assertion::IsIdentical(atomic));
                }

                Some((0, value_assertions))
            }
            _ => None,
        };

        let mut result = None;
        if should_check_against_unresolved {
            result = get_builtin_assertion(function_name.as_bytes());
        }

        if result.is_none() {
            result = get_builtin_assertion(resolved_function_name.as_bytes());
        }

        let Some(found_assertions) = result else {
            return if_types;
        };

        found_assertions
    };

    let extract_expression_id = |argument_expression: &Expression| {
        if let Some(id) = assertion_context.get_expression_id(argument_expression) {
            return Some((id, false));
        }

        if let Expression::Binary(binary) = unwrap_expression(argument_expression)
            && matches!(binary.operator, BinaryOperator::NullCoalesce(_))
            && matches!(unwrap_expression(binary.rhs), Expression::Literal(Literal::Null(_)))
            && let Some(lhs_id) = assertion_context.get_expression_id(binary.lhs)
        {
            return Some((lhs_id, true));
        }

        None
    };

    if let Some(argument) =
        function_call.argument_list.arguments.get(argument_variable_id_position).map(mago_syntax::cst::Argument::value)
        && let Some((variable_id, needs_isset)) = extract_expression_id(argument)
    {
        let mut assertions = vec![function_assertions];
        if needs_isset {
            assertions.push(vec![Assertion::IsIsset]);
        }

        if_types.insert(variable_id, assertions);
    }

    if_types
}

fn get_possible_array_key_argument_type(
    array_type: &TUnion,
    input_key_type: &TUnion,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let mut key_type = None;

    for atomic in array_type.types.as_ref() {
        let TAtomic::Array(array) = atomic else {
            continue;
        };

        match array {
            TArray::Keyed(keyed) => {
                if let Some(known_items) = keyed.get_known_items() {
                    for key in known_items.keys() {
                        add_possible_key_argument_type(&key.to_atomic(), input_key_type, &mut key_type, codebase);
                    }
                }

                if let Some((parameter, _)) = keyed.get_generic_parameters() {
                    for atomic in parameter.types.as_ref() {
                        add_possible_key_argument_type(atomic, input_key_type, &mut key_type, codebase);
                    }
                }
            }
            TArray::List(list) => {
                if !list.get_element_type().is_never() {
                    add_possible_key_argument_type(
                        &TAtomic::Scalar(TScalar::Integer(TInteger::unspecified())),
                        input_key_type,
                        &mut key_type,
                        codebase,
                    );

                    continue;
                }

                if let Some(known_elements) = list.get_known_elements() {
                    for index in known_elements.keys() {
                        let Ok(index) = i64::try_from(*index) else {
                            add_possible_key_argument_type(
                                &TAtomic::Scalar(TScalar::Integer(TInteger::unspecified())),
                                input_key_type,
                                &mut key_type,
                                codebase,
                            );

                            break;
                        };

                        add_possible_key_argument_type(
                            &ArrayKey::Integer(index).to_atomic(),
                            input_key_type,
                            &mut key_type,
                            codebase,
                        );
                    }
                }
            }
        }
    }

    key_type
}

fn add_possible_key_argument_type(
    key: &TAtomic,
    input_key_type: &TUnion,
    possible_type: &mut Option<TUnion>,
    codebase: &CodebaseMetadata,
) {
    let add = |candidate, possible_type: &mut Option<TUnion>| {
        *possible_type = Some(add_optional_union_type(candidate, possible_type.as_ref(), codebase));
    };

    let accepts_all_scalars = input_key_type.is_mixed() || input_key_type.has_scalar();

    match key {
        TAtomic::Scalar(TScalar::Integer(integer)) => {
            if accepts_all_scalars || input_key_type.has_int() {
                add(TUnion::from_atomic(key.clone()), possible_type);
            }

            if accepts_all_scalars || input_key_type.has_float() {
                add(get_float(), possible_type);
            }

            if let Some(value) = integer.get_literal_value() {
                if accepts_all_scalars || input_key_type.has_string() {
                    add(TUnion::from_atomic(TAtomic::Scalar(TScalar::literal_string(i64_word(value)))), possible_type);
                }

                if accepts_all_scalars || input_key_type.has_bool() {
                    if value == 0 {
                        add(get_false(), possible_type);
                    } else if value == 1 {
                        add(get_true(), possible_type);
                    }
                }
            } else {
                if accepts_all_scalars || input_key_type.has_string() {
                    add(get_numeric_string(), possible_type);
                }

                if accepts_all_scalars || input_key_type.has_bool() {
                    add(get_bool(), possible_type);
                }
            }
        }
        TAtomic::Scalar(TScalar::ArrayKey) => {
            add(get_arraykey(), possible_type);
            if accepts_all_scalars || input_key_type.has_float() {
                add(get_float(), possible_type);
            }

            if accepts_all_scalars || input_key_type.has_bool() {
                add(get_bool(), possible_type);
            }
        }
        _ => add(TUnion::from_atomic(key.clone()), possible_type),
    }
}

pub(super) fn scrape_equality_assertions<A>(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
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
            let mut if_types = WordMap::default();

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
            let mut if_types = WordMap::default();

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
    }

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

fn scrape_inequality_assertions<A>(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
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
            let mut if_types = WordMap::default();

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
            let mut if_types = WordMap::default();

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
    }

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
fn scrape_class_constant_equality_assertions<A>(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
    negated: bool,
) -> Option<Vec<WordMap<AssertionSet>>>
where
    A: Arena,
{
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

    let variable_id = assertion_context.get_expression_id(variable_expr)?;

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

    let mut if_types = WordMap::default();
    if_types.insert(variable_id, vec![assertions]);
    Some(vec![if_types])
}

/// Helper to check if an expression is a `::class` constant access.
/// Returns the expression for the class part (e.g., `$foo` in `$foo::class`).
#[inline]
fn is_class_constant_access<'arena>(expr: &'arena Expression<'arena>) -> Option<&'arena Expression<'arena>> {
    if let Expression::Access(Access::ClassConstant(ClassConstantAccess {
        class,
        constant: ClassLikeConstantSelector::Identifier(LocalIdentifier { value: b"class", .. }),
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

fn get_empty_array_equality_assertions<A>(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    null_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

    if let Some(var_name) = var_name {
        if is_identity {
            if_types.insert(var_name, vec![vec![Assertion::EmptyCountable]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Falsy]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_empty_array_inequality_assertions<A>(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    null_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

    if let Some(var_name) = var_name {
        if operator.is_identity() {
            if_types.insert(var_name, vec![vec![Assertion::NonEmptyCountable(true)]]);
        } else {
            if_types.insert(var_name, vec![vec![Assertion::Truthy]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_enum_case_equality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let (variable_expression, Some(enum_case_type)) = (match enum_case_position {
        OtherValuePosition::Left => (right, artifacts.get_expression_type(left)),
        OtherValuePosition::Right => (left, artifacts.get_expression_type(right)),
    }) else {
        return vec![];
    };

    let mut if_types = WordMap::default();

    let var_name = assertion_context.get_expression_id(variable_expression);

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsType(enum_case_type.get_single().clone())]]);
    }

    vec![if_types]
}

fn get_enum_case_inequality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    enum_case_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let (variable_expression, Some(enum_case_type)) = (match enum_case_position {
        OtherValuePosition::Left => (right, artifacts.get_expression_type(left)),
        OtherValuePosition::Right => (left, artifacts.get_expression_type(right)),
    }) else {
        return vec![];
    };

    let mut if_types = WordMap::default();

    let var_name = assertion_context.get_expression_id(variable_expression);

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsNotType(enum_case_type.get_single().clone())]]);
    }

    vec![if_types]
}

fn get_null_equality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    null_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    if let Expression::Binary(binary) = unwrap_expression(base_conditional)
        && let BinaryOperator::NullCoalesce(_) = binary.operator
        && let Expression::Literal(Literal::Null(_)) = unwrap_expression(binary.rhs)
    {
        let coalesce_lhs = binary.lhs;

        if let Some(var_name) = assertion_context.get_expression_id(coalesce_lhs) {
            if_types.insert(var_name, vec![vec![Assertion::IsNotIsset]]);
        }
    } else {
        let var_name = assertion_context.get_expression_id(base_conditional);

        if let Some(var_name) = var_name {
            if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Null)]]);
        }
    }

    vec![if_types]
}

fn get_null_inequality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    null_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match null_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    if let Expression::Binary(binary) = unwrap_expression(base_conditional)
        && matches!(binary.operator, BinaryOperator::NullCoalesce(_))
        && matches!(unwrap_expression(binary.rhs), Expression::Literal(Literal::Null(_)))
    {
        let coalesce_lhs = binary.lhs;

        if let Some(var_name) = assertion_context.get_expression_id(coalesce_lhs) {
            if_types.insert(var_name, vec![vec![Assertion::IsIsset]]);
        } else if let Expression::ArrayAccess(array_access) = coalesce_lhs
            && let Some(root_array_id) = assertion_context.get_expression_id(array_access.array)
        {
            if_types.insert(root_array_id, vec![vec![Assertion::IsEqualIsset], vec![Assertion::Truthy]]);
        }
    } else {
        let var_name = assertion_context.get_expression_id(base_conditional);

        if let Some(var_name) = var_name {
            if_types.insert(var_name, vec![vec![Assertion::IsNotType(TAtomic::Null)]]);
        }
    }

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_false_inquality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    false_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsNotType(TAtomic::Scalar(TScalar::r#false()))]]);
    }

    vec![if_types]
}

fn get_true_inquality_assertions<A>(
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    true_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

    if let Some(var_name) = var_name {
        if_types.insert(var_name, vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::r#true()))]]);
    }

    vec![if_types]
}

fn scrape_lesser_than_assertions<A>(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = WordMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, left) {
                let maximum_count = if matches!(operator, BinaryOperator::LessThan(_)) {
                    number_on_right.saturating_sub(1)
                } else {
                    number_on_right
                };

                match maximum_count.cmp(&0) {
                    Ordering::Less => {
                        // This branch is logically unreachable, e.g. `count($arr) < 0`.
                    }
                    Ordering::Equal => {
                        if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                    }
                    Ordering::Greater => {
                        if_types.insert(
                            array_variable_id,
                            vec![vec![Assertion::DoesNotHasAtLeastCount(maximum_count as usize)]],
                        );
                    }
                }
            }

            return if if_types.is_empty() { vec![] } else { vec![if_types] };
        }
        (Some(number_on_left), None) => {
            let mut if_types = WordMap::default();

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

    let mut if_types: WordMap<AssertionSet> = WordMap::default();

    let left_id = assertion_context.get_expression_id(left);

    let right_id = assertion_context.get_expression_id(right);

    if let (Some(left_var_id), Some(right_var_id)) = (left_id, right_id) {
        let relation = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            Assertion::IsLessThanOrEqualVariable(right_var_id)
        } else {
            Assertion::IsLessThanVariable(right_var_id)
        };

        if_types.entry(left_var_id).or_default().push(vec![relation]);
    }

    // Generate assertions for the left variable based on the right variable's type.
    // For an expression `$a < $b`, this asserts `$a` is less than the upper bound of `$b`.
    // A non-literal bound is only a one-way implication: the assertion narrows
    // the true branch but must not narrow the negated branch.
    if let (Some(left_var_id), Some(right_int)) = (left_id, &right_integer) {
        let use_range_bounds = right_id.is_some();

        let assertion_result = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThanOrEqual(count), count)),
                TInteger::To(upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanOrEqualFromBound(upper_bound), upper_bound))
                }
                TInteger::Range(_, upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanOrEqualFromBound(upper_bound), upper_bound))
                }
                _ => None,
            }
        } else {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThan(count), count)),
                TInteger::To(upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanFromBound(upper_bound), upper_bound))
                }
                TInteger::Range(_, upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanFromBound(upper_bound), upper_bound))
                }
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
                if_types.entry(left_var_id).or_default().push(vec![assertion]);
            }
        }
    }

    // Generate assertions for the right variable based on the left variable's type.
    // For an expression `$a < $b`, this asserts `$b` is greater than the lower bound of `$a`.
    // Range bounds are NOT used here: using `$a`'s lower bound to assert `$b > min($a)` is
    // correct for the true branch, but when negated gives `$b <= min($a)` which is wrong
    // (the false branch only implies `$b <= $a`, not `$b <= min($a)`).
    if let (Some(right_var_id), Some(left_int)) = (right_id, &left_integer) {
        let use_range_bounds = false;

        let assertion_result = if matches!(operator, BinaryOperator::LessThanOrEqual(_)) {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThanOrEqual(count), count)),
                TInteger::From(lower_bound) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound))
                }
                TInteger::Range(lower_bound, _) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanOrEqual(lower_bound), lower_bound))
                }
                _ => None,
            }
        } else {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThan(count), count)),
                TInteger::From(lower_bound) if use_range_bounds => {
                    Some((Assertion::IsGreaterThan(lower_bound), lower_bound))
                }
                TInteger::Range(lower_bound, _) if use_range_bounds => {
                    Some((Assertion::IsGreaterThan(lower_bound), lower_bound))
                }
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

fn scrape_greater_than_assertions<A>(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    match resolve_count_comparison(left, right, artifacts, assertion_context) {
        (None, Some(number_on_right)) => {
            let mut if_types = WordMap::default();

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
            let mut if_types = WordMap::default();

            if let Some(array_variable_id) = get_first_argument_expression_id(assertion_context, right) {
                let maximum_count = if matches!(operator, BinaryOperator::GreaterThan(_)) {
                    number_on_left.saturating_sub(1)
                } else {
                    number_on_left
                };

                match maximum_count.cmp(&0) {
                    Ordering::Less => {
                        // This branch is logically unreachable, e.g. `-1 > count($arr)`.
                    }
                    Ordering::Equal => {
                        if_types.insert(array_variable_id, vec![vec![Assertion::EmptyCountable]]);
                    }
                    Ordering::Greater => {
                        if_types.insert(
                            array_variable_id,
                            vec![vec![Assertion::DoesNotHasAtLeastCount(maximum_count as usize)]],
                        );
                    }
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

    let mut if_types: WordMap<AssertionSet> = WordMap::default();

    let left_id = assertion_context.get_expression_id(left);
    let right_id = assertion_context.get_expression_id(right);

    if let (Some(left_var_id), Some(right_var_id)) = (left_id, right_id) {
        let relation = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            Assertion::IsGreaterThanOrEqualVariable(right_var_id)
        } else {
            Assertion::IsGreaterThanVariable(right_var_id)
        };

        if_types.entry(left_var_id).or_default().push(vec![relation]);
    }

    // Generate assertions for the left variable based on the right variable's type.
    // For an expression `$a > $b`, this asserts `$a` is greater than the lower bound of `$b`.
    // A non-literal bound is only a one-way implication: the assertion narrows
    // the true branch but must not narrow the negated branch.
    if let Some(right_int) = &right_integer
        && let Some(left_var_id) = assertion_context.get_expression_id(left)
    {
        let use_range_bounds = assertion_context.get_expression_id(right).is_some();

        let assertion_result = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThanOrEqual(count), count)),
                TInteger::From(lower_bound) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanOrEqualFromBound(lower_bound), lower_bound))
                }
                TInteger::Range(lower_bound, _) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanOrEqualFromBound(lower_bound), lower_bound))
                }
                _ => None,
            }
        } else {
            match *right_int {
                TInteger::Literal(count) => Some((Assertion::IsGreaterThan(count), count)),
                TInteger::From(lower_bound) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanFromBound(lower_bound), lower_bound))
                }
                TInteger::Range(lower_bound, _) if use_range_bounds => {
                    Some((Assertion::IsGreaterThanFromBound(lower_bound), lower_bound))
                }
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
                if_types.entry(left_var_id).or_default().push(vec![assertion]);
            }
        }
    }

    // Generate assertions for the right variable based on the left variable's type.
    // For an expression `$a > $b`, this asserts `$b` is less than the upper bound of `$a`.
    // As above, range-derived facts must not be treated as the negation of the
    // full comparison in the false branch.
    if let Some(left_int) = &left_integer
        && let Some(right_var_id) = assertion_context.get_expression_id(right)
    {
        let use_range_bounds = assertion_context.get_expression_id(left).is_some();

        let assertion_result = if matches!(operator, BinaryOperator::GreaterThanOrEqual(_)) {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThanOrEqual(count), count)),
                TInteger::To(upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanOrEqualFromBound(upper_bound), upper_bound))
                }
                TInteger::Range(_, upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanOrEqualFromBound(upper_bound), upper_bound))
                }
                _ => None,
            }
        } else {
            match *left_int {
                TInteger::Literal(count) => Some((Assertion::IsLessThan(count), count)),
                TInteger::To(upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanFromBound(upper_bound), upper_bound))
                }
                TInteger::Range(_, upper_bound) if use_range_bounds => {
                    Some((Assertion::IsLessThanFromBound(upper_bound), upper_bound))
                }
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

fn scrape_instanceof_assertions<A>(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    context: AssertionContext<'_, '_, A>,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();

    let left = unwrap_expression(left);
    let assertion_target = match left {
        Expression::Binary(binary)
            if matches!(binary.operator, BinaryOperator::NullCoalesce(_))
                && matches!(unwrap_expression(binary.rhs), Expression::Literal(Literal::Null(_))) =>
        {
            binary.lhs
        }
        _ => left,
    };

    let variable_id =
        get_expression_id(assertion_target, context.this_class_name, context.resolved_names, Some(context.codebase))
            .map(|id| get_non_nullsafe_expression_id(id).unwrap_or(id));

    if let Some(counter_variable_id) = variable_id {
        match right {
            Expression::Identifier(identifier) => {
                let resolved_name = context.resolved_names.get(identifier);

                if_types.insert(
                    counter_variable_id,
                    vec![vec![Assertion::IsType(TAtomic::Object(TObject::Named(TNamedObject::new(word(
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
                    && let Some(self_meta) = context.codebase.get_class_like(self_class.as_bytes())
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
        }
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
fn resolve_count_comparison<A>(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> (Option<i64>, Option<i64>)
where
    A: Arena,
{
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
    if let Some(int) = artifacts
        .get_expression_type(expression)
        .and_then(mago_codex::ttype::union::TUnion::get_single_int)
        .filter(|integer| !integer.is_unspecified())
    {
        return Some(int);
    }

    match expression {
        Expression::Literal(Literal::Integer(lit)) => lit.value.map(|v| TInteger::Literal(v as i64)),
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Negation(_), operand }) => {
            if let Expression::Literal(Literal::Integer(lit)) = operand {
                lit.value.map(|v| TInteger::Literal(-(v as i64)))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_expression_array_key(artifacts: &AnalysisArtifacts, expression: &Expression) -> Option<ArrayKey> {
    artifacts.get_expression_type(expression).and_then(mago_codex::ttype::union::TUnion::get_single_array_key)
}

fn is_count_or_size_of_call<A>(expression: &Expression, assertion_context: AssertionContext<'_, '_, A>) -> bool
where
    A: Arena,
{
    is_function_call_to_one_of(expression, assertion_context, &[b"count", b"sizeof", b"Psl\\Iter\\count"])
}

fn is_function_call_to<A>(
    expression: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    function_name: &[u8],
) -> bool
where
    A: Arena,
{
    is_function_call_to_one_of(expression, assertion_context, &[function_name])
}

#[inline]
fn is_function_call_to_one_of<A>(
    expression: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    functions: &[&[u8]],
) -> bool
where
    A: Arena,
{
    let Expression::Call(Call::Function(FunctionCall { function, argument_list })) = expression else {
        return false;
    };

    if argument_list.arguments.len() != 1 {
        return false;
    }

    let Expression::Identifier(function_identifier) = function else {
        return false;
    };

    if functions.iter().any(|name| function_identifier.value().eq_ignore_ascii_case(name)) {
        return true;
    }

    let resolved_function_name = assertion_context.resolved_names.get(function_identifier);

    functions.iter().any(|name| resolved_function_name.eq_ignore_ascii_case(name))
}

fn get_true_equality_assertions<A>(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
    true_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match true_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

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

pub fn has_typed_value_comparison<A>(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
) -> Option<OtherValuePosition>
where
    A: Arena,
{
    let left_var_id = assertion_context.get_expression_id(left);

    let right_var_id = assertion_context.get_expression_id(right);

    let left_is_class_constant =
        left_var_id.as_ref().is_some_and(|id| memchr::memmem::find(id.as_bytes(), b"::").is_some());
    let right_is_simple_var = right_var_id.as_ref().is_some_and(|id| id.as_bytes().starts_with(b"$"));

    if left_is_class_constant
        && right_is_simple_var
        && let Some(left_type) = artifacts.get_expression_type(&left.span())
        && left_type.is_single()
        && !left_type.is_mixed()
    {
        return Some(OtherValuePosition::Left);
    }

    if let Some(right_type) = artifacts.get_expression_type(&right.span())
        && (left_var_id.is_some() || right_var_id.is_none())
        && right_type.is_single()
        && !right_type.is_mixed()
    {
        return Some(OtherValuePosition::Right);
    }

    if let Some(left_type) = artifacts.get_expression_type(&left.span())
        && left_type.is_single()
        && !left_type.is_mixed()
    {
        return Some(OtherValuePosition::Left);
    }

    None
}

fn get_false_equality_assertions<A>(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_, A>,
    false_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();
    let base_conditional = match false_position {
        OtherValuePosition::Left => right,
        OtherValuePosition::Right => left,
    };

    let var_name = assertion_context.get_expression_id(base_conditional);

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

fn get_typed_value_equality_assertions<A>(
    left: &Expression,
    is_identity: bool,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
    typed_value_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();

    let (var_name, other_value_var_name, var_type, other_value_type) = match typed_value_position {
        OtherValuePosition::Right => (
            assertion_context.get_expression_id(left),
            assertion_context.get_expression_id(right),
            artifacts.get_expression_type(&left.span()),
            artifacts.get_expression_type(&right.span()),
        ),
        OtherValuePosition::Left => (
            assertion_context.get_expression_id(right),
            assertion_context.get_expression_id(left),
            artifacts.get_expression_type(&right.span()),
            artifacts.get_expression_type(&left.span()),
        ),
    };

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

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

fn get_typed_value_inequality_assertions<A>(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    assertion_context: AssertionContext<'_, '_, A>,
    typed_value_position: OtherValuePosition,
) -> Vec<WordMap<AssertionSet>>
where
    A: Arena,
{
    let mut if_types = WordMap::default();

    let (var_name, other_value_var_name, var_type, other_value_type) = match typed_value_position {
        OtherValuePosition::Right => (
            assertion_context.get_expression_id(left),
            assertion_context.get_expression_id(right),
            artifacts.get_expression_type(&left.span()),
            artifacts.get_expression_type(&right.span()),
        ),
        OtherValuePosition::Left => (
            assertion_context.get_expression_id(right),
            assertion_context.get_expression_id(left),
            artifacts.get_expression_type(&right.span()),
            artifacts.get_expression_type(&left.span()),
        ),
    };

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

    if if_types.is_empty() { vec![] } else { vec![if_types] }
}

#[inline]
fn get_first_argument_expression_id<A>(
    assertion_context: AssertionContext<'_, '_, A>,
    expression: &Expression,
) -> Option<Word>
where
    A: Arena,
{
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
            .is_some_and(mago_codex::ttype::union::TUnion::is_single_enum_case)
    {
        return Some(OtherValuePosition::Right);
    }

    if let Expression::Access(Access::ClassConstant(class_constant_access)) = unwrap_expression(left)
        && artifacts
            .get_expression_type(class_constant_access)
            .is_some_and(mago_codex::ttype::union::TUnion::is_single_enum_case)
    {
        return Some(OtherValuePosition::Left);
    }

    None
}

fn has_literal_operand(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
    is_literal: fn(&Expression) -> bool,
    is_type: fn(&TUnion) -> bool,
) -> Option<OtherValuePosition> {
    for (operand, position) in [(right, OtherValuePosition::Right), (left, OtherValuePosition::Left)] {
        if is_literal(unwrap_expression(operand)) {
            return Some(position);
        }

        if artifacts.get_expression_type(operand).is_some_and(is_type) {
            return Some(position);
        }
    }

    None
}

#[inline]
pub fn has_null_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    has_literal_operand(
        left,
        right,
        artifacts,
        |expression| matches!(expression, Expression::Literal(Literal::Null(_))),
        TUnion::is_null,
    )
}

#[inline]
pub fn has_false_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    has_literal_operand(
        left,
        right,
        artifacts,
        |expression| matches!(expression, Expression::Literal(Literal::False(_))),
        TUnion::is_false,
    )
}

#[inline]
pub fn has_true_variable(
    left: &Expression,
    right: &Expression,
    artifacts: &AnalysisArtifacts,
) -> Option<OtherValuePosition> {
    has_literal_operand(
        left,
        right,
        artifacts,
        |expression| matches!(expression, Expression::Literal(Literal::True(_))),
        TUnion::is_true,
    )
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
