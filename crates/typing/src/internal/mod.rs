use ahash::HashSet;
use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_trinary::Trinary;
use ordered_float::OrderedFloat;

use fennec_ast::*;
use fennec_reflection::r#type::kind::*;
use sequence::TokenSeparatedSequence;

#[inline]
pub fn resolve_name<'i>(
    interner: &'i ThreadedInterner,
    value_id: StringIdentifier,
) -> (StringIdentifier, StringIdentifier) {
    let value = interner.lookup(&value_id);

    if value.contains('\\') {
        // take the last part of the path
        let short_value: &'i str = unsafe { value.split('\\').last().unwrap_unchecked() };

        return (value_id, interner.intern(short_value));
    } else {
        (value_id, value_id)
    }
}

/// This function can use some improvements.
#[inline]
pub fn get_array_index_kind(array: TypeKind) -> TypeKind {
    match array {
        TypeKind::Array(array_type_kind) => match array_type_kind {
            ArrayTypeKind::Array { value, .. } => value.as_ref().clone(),
            ArrayTypeKind::List { value, .. } => value.as_ref().clone(),
            ArrayTypeKind::CallableArray => mixed_kind(false),
            ArrayTypeKind::Shape(array_shape) => {
                let mut possible_kinds = HashSet::default();
                for property in array_shape.properties.iter() {
                    possible_kinds.insert(property.kind.clone());
                }

                if let Some(additional_properties) = array_shape.additional_properties {
                    possible_kinds.insert(additional_properties.1.as_ref().clone());
                }

                if possible_kinds.len() == 1 {
                    possible_kinds.into_iter().next().unwrap()
                } else {
                    union_kind(possible_kinds.into_iter().collect())
                }
            }
        },
        _ => mixed_kind(false),
    }
}

#[inline]
pub fn get_composite_string_kind<F>(composite_string: &CompositeString, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    let parts = match composite_string {
        CompositeString::ShellExecute(_) => return union_kind(vec![string_kind(), null_kind(), false_kind()]),
        CompositeString::Interpolated(interpolated_string) => &interpolated_string.parts,
        CompositeString::Document(document_string) => &document_string.parts,
    };

    let mut contains_non_empty_part = false;
    for part in parts.iter() {
        match &part {
            StringPart::Literal(literal_string_part) => {
                if !literal_string_part.value.is_empty() {
                    contains_non_empty_part = true;

                    break;
                }
            }
            StringPart::Expression(expression) => {
                if let TypeKind::Scalar(ScalarTypeKind::NonEmptyString) = get_expression_kind(expression) {
                    contains_non_empty_part = true;

                    break;
                }
            }
            StringPart::BracedExpression(braced_expression_string_part) => {
                if let TypeKind::Scalar(ScalarTypeKind::NonEmptyString) =
                    get_expression_kind(&braced_expression_string_part.expression)
                {
                    contains_non_empty_part = true;

                    break;
                }
            }
        }
    }

    if contains_non_empty_part {
        non_empty_string_kind()
    } else {
        string_kind()
    }
}

#[inline]
pub fn get_bitwise_operation_kind<F>(
    interner: &ThreadedInterner,
    bitwise_operation: &BitwiseOperation,
    get_expression_kind: F,
) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    match bitwise_operation {
        BitwiseOperation::Prefix(bitwise_prefix_operation) => {
            match get_expression_kind(&bitwise_prefix_operation.value) {
                TypeKind::Never => never_kind(),
                TypeKind::Value(ValueTypeKind::Integer { value }) => value_integer_kind(!value),
                TypeKind::Scalar(ScalarTypeKind::Integer { .. }) => integer_kind(),
                kind if is_gmp_or_bcmath_number(interner, &kind) => kind,
                kind if kind.is_object() || kind.is_resource() || kind.is_array() => never_kind(),
                _ => integer_kind(),
            }
        }
        BitwiseOperation::Infix(bitwise_infix_operation) => {
            match (get_expression_kind(&bitwise_infix_operation.lhs), get_expression_kind(&bitwise_infix_operation.rhs))
            {
                (TypeKind::Never, _) | (_, TypeKind::Never) => never_kind(),
                (lhs_value_kind, rhs_value_kind)
                    if is_numeric_value_kind(&lhs_value_kind) && is_numeric_value_kind(&rhs_value_kind) =>
                {
                    if !can_extract_literal_value(&lhs_value_kind) || !can_extract_literal_value(&rhs_value_kind) {
                        return integer_kind();
                    }

                    let lhs_value = extract_literal_value(&lhs_value_kind);
                    let rhs_value = extract_literal_value(&rhs_value_kind);

                    let lhs_value = lhs_value.trunc() as i64;
                    let rhs_value = rhs_value.trunc() as i64;

                    let result = match &bitwise_infix_operation.operator {
                        BitwiseInfixOperator::And(_) => lhs_value & rhs_value,
                        BitwiseInfixOperator::Or(_) => lhs_value | rhs_value,
                        BitwiseInfixOperator::Xor(_) => lhs_value ^ rhs_value,
                        BitwiseInfixOperator::LeftShift(_) => {
                            if rhs_value < 0 {
                                return never_kind();
                            }

                            if rhs_value > u32::MAX as i64 {
                                0i64
                            } else {
                                lhs_value.wrapping_shl(rhs_value as u32)
                            }
                        }
                        BitwiseInfixOperator::RightShift(_) => {
                            if rhs_value < 0 {
                                return never_kind();
                            }

                            if rhs_value > u32::MAX as i64 {
                                0i64
                            } else {
                                lhs_value.wrapping_shr(rhs_value as u32)
                            }
                        }
                    };

                    value_integer_kind(result)
                }
                (kind, _) if is_gmp_or_bcmath_number(interner, &kind) => kind,
                (_, kind) if is_gmp_or_bcmath_number(interner, &kind) => kind,
                (left, right)
                    if left.is_object()
                        || left.is_resource()
                        || left.is_array()
                        || right.is_object()
                        || right.is_resource()
                        || right.is_array() =>
                {
                    never_kind()
                }
                _ => integer_kind(),
            }
        }
    }
}

#[inline]
pub fn get_logical_operation_kind<F>(logical_operation: &LogicalOperation, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    match logical_operation {
        LogicalOperation::Prefix(logical_prefix_operation) => {
            let value_kind = get_expression_kind(&logical_prefix_operation.value);

            if matches!(value_kind, TypeKind::Never) {
                return never_kind();
            }

            match value_kind.is_truthy() {
                Trinary::True => false_kind(),
                Trinary::False => true_kind(),
                _ => bool_kind(),
            }
        }
        LogicalOperation::Infix(logical_infix_operation) => {
            let lhs_kind = get_expression_kind(&logical_infix_operation.lhs);
            let rhs_kind = get_expression_kind(&logical_infix_operation.rhs);

            if matches!(lhs_kind, TypeKind::Never) || matches!(rhs_kind, TypeKind::Never) {
                return never_kind();
            }

            match (lhs_kind.is_truthy(), rhs_kind.is_truthy()) {
                (Trinary::True, Trinary::True) => match logical_infix_operation.operator {
                    LogicalInfixOperator::And(_) | LogicalInfixOperator::LowPrecedenceAnd(_) => true_kind(),
                    LogicalInfixOperator::Or(_) | LogicalInfixOperator::LowPrecedenceOr(_) => true_kind(),
                    LogicalInfixOperator::LowPrecedenceXor(_) => false_kind(),
                },
                (Trinary::False, Trinary::False) => match logical_infix_operation.operator {
                    LogicalInfixOperator::And(_) | LogicalInfixOperator::LowPrecedenceAnd(_) => false_kind(),
                    LogicalInfixOperator::Or(_) | LogicalInfixOperator::LowPrecedenceOr(_) => false_kind(),
                    LogicalInfixOperator::LowPrecedenceXor(_) => false_kind(),
                },
                (Trinary::True, Trinary::False) => match logical_infix_operation.operator {
                    LogicalInfixOperator::And(_) | LogicalInfixOperator::LowPrecedenceAnd(_) => false_kind(),
                    LogicalInfixOperator::Or(_) | LogicalInfixOperator::LowPrecedenceOr(_) => true_kind(),
                    LogicalInfixOperator::LowPrecedenceXor(_) => true_kind(),
                },
                (Trinary::False, Trinary::True) => match logical_infix_operation.operator {
                    LogicalInfixOperator::And(_) | LogicalInfixOperator::LowPrecedenceAnd(_) => false_kind(),
                    LogicalInfixOperator::Or(_) | LogicalInfixOperator::LowPrecedenceOr(_) => true_kind(),
                    LogicalInfixOperator::LowPrecedenceXor(_) => true_kind(),
                },
                (_, _) => bool_kind(),
            }
        }
    }
}

#[inline]
pub fn get_unary_prefix_operation_kind<F>(unary_operation: &UnaryPrefixOperation, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    let value_kind = get_expression_kind(&unary_operation.operand);

    match &unary_operation.operator {
        UnaryPrefixOperator::ArrayCast(_, _) => {
            if value_kind.is_array() {
                // the value is already an array, which could be more specific, so we keep it.
                value_kind
            } else {
                array_kind(array_key_kind(), mixed_kind(false), None)
            }
        }
        UnaryPrefixOperator::BoolCast(_, _) | UnaryPrefixOperator::BooleanCast(_, _) => {
            if value_kind.is_bool().is_true() {
                return value_kind;
            }

            match value_kind.is_truthy() {
                Trinary::True => true_kind(),
                Trinary::Maybe => bool_kind(),
                Trinary::False => false_kind(),
            }
        }
        UnaryPrefixOperator::DoubleCast(_, _)
        | UnaryPrefixOperator::RealCast(_, _)
        | UnaryPrefixOperator::FloatCast(_, _) => {
            if value_kind.is_float().is_true() {
                return value_kind;
            } else {
                return float_kind();
            }
        }
        UnaryPrefixOperator::IntCast(_, _) | UnaryPrefixOperator::IntegerCast(_, _) => {
            if value_kind.is_integer().is_true() {
                return value_kind;
            } else {
                return integer_kind();
            }
        }
        UnaryPrefixOperator::ObjectCast(_, _) => {
            if value_kind.is_object() {
                // the value is already an object, which could be more specific, so we keep it.
                value_kind
            } else {
                any_object_kind()
            }
        }
        UnaryPrefixOperator::UnsetCast(_, _) => void_kind(),
        UnaryPrefixOperator::StringCast(_, _) | UnaryPrefixOperator::BinaryCast(_, _) => {
            if value_kind.is_string().is_true() {
                // the value is already a string, which could be more specific, so we keep it.
                value_kind
            } else {
                string_kind()
            }
        }
        UnaryPrefixOperator::ErrorControl(_) => void_kind(),
        UnaryPrefixOperator::Reference(_) => value_kind,
        UnaryPrefixOperator::BitwiseNot(_) => {
            if value_kind.is_integer().is_true() {
                return value_kind;
            } else {
                return integer_kind();
            }
        }
        UnaryPrefixOperator::Not(_) => match value_kind.is_truthy() {
            Trinary::True => false_kind(),
            Trinary::Maybe => bool_kind(),
            Trinary::False => true_kind(),
        },
        UnaryPrefixOperator::PreIncrement(_) => {
            if value_kind.is_integer().is_true() {
                return value_kind;
            } else {
                return integer_kind();
            }
        }
        UnaryPrefixOperator::PreDecrement(_) => {
            if value_kind.is_integer().is_true() {
                return value_kind;
            } else {
                return integer_kind();
            }
        }
        UnaryPrefixOperator::Plus(_) | UnaryPrefixOperator::Negation(_) => {
            if value_kind.is_integer().is_true() || value_kind.is_float().is_true() {
                return value_kind;
            } else {
                return union_kind(vec![integer_kind(), float_kind()]);
            }
        }
    }
}

#[inline]
pub fn get_ternary_operation_kind<F>(ternary_operation: &TernaryOperation, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    match ternary_operation {
        TernaryOperation::Conditional(conditional_ternary_operation) => {
            let condition_kind = get_expression_kind(&conditional_ternary_operation.condition);

            match &conditional_ternary_operation.then {
                Some(then) => {
                    let then_kind = get_expression_kind(then);
                    let else_kind = get_expression_kind(&conditional_ternary_operation.r#else);

                    match condition_kind.is_truthy() {
                        Trinary::True => then_kind,
                        Trinary::Maybe => union_kind(vec![then_kind, else_kind]),
                        Trinary::False => else_kind,
                    }
                }
                None => {
                    let else_kind = get_expression_kind(&conditional_ternary_operation.r#else);

                    match condition_kind.is_truthy() {
                        Trinary::True => condition_kind,
                        Trinary::Maybe => union_kind(vec![condition_kind, else_kind]),
                        Trinary::False => else_kind,
                    }
                }
            }
        }
        TernaryOperation::Elvis(elvis_ternary_operation) => {
            let condition_kind = get_expression_kind(&elvis_ternary_operation.condition);
            let else_kind = get_expression_kind(&elvis_ternary_operation.r#else);

            match condition_kind.is_truthy() {
                Trinary::True => condition_kind,
                Trinary::Maybe => union_kind(vec![condition_kind, else_kind]),
                Trinary::False => else_kind,
            }
        }
    }
}

#[inline]
pub fn get_binary_operation_kind<F>(binary_operation: &BinaryOperation, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    let left_kind = get_expression_kind(&binary_operation.lhs);
    let right_kind = get_expression_kind(&binary_operation.rhs);

    if matches!(left_kind, TypeKind::Never) || matches!(right_kind, TypeKind::Never) {
        return never_kind();
    }

    match &binary_operation.operator {
        BinaryOperator::StringConcat(_) => {
            if left_kind.is_non_empty_string().or(right_kind.is_non_empty_string()).is_true() {
                return non_empty_string_kind();
            }

            if left_kind
                .is_integer()
                .or(left_kind.is_float())
                .and(right_kind.is_integer().or(right_kind.is_float()))
                .is_true()
            {
                return TypeKind::Scalar(ScalarTypeKind::NumericString);
            }

            string_kind()
        }
        BinaryOperator::Equal(_) | BinaryOperator::Identical(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind == right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_) | BinaryOperator::NotIdentical(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind != right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::LessThan(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind < right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::GreaterThan(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind > right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::LessThanOrEqual(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind <= right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::GreaterThanOrEqual(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                if left_kind >= right_kind {
                    return true_kind();
                } else {
                    return false_kind();
                }
            } else {
                bool_kind()
            }
        }
        BinaryOperator::Spaceship(_) => {
            if can_extract_literal_value(&left_kind) && can_extract_literal_value(&right_kind) {
                let left_kind = extract_literal_value(&left_kind);
                let right_kind = extract_literal_value(&right_kind);

                let cmp_result = left_kind.partial_cmp(&right_kind).unwrap_or(std::cmp::Ordering::Equal);

                return value_integer_kind(match cmp_result {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                });
            } else {
                integer_range_kind(-1, 1)
            }
        }
        BinaryOperator::Instanceof(_) => bool_kind(),
        BinaryOperator::NullCoalesce(_) => {
            if left_kind == right_kind {
                return left_kind;
            }

            match left_kind.is_nullable() {
                Trinary::False => left_kind,
                Trinary::True => right_kind,
                Trinary::Maybe => union_kind(vec![left_kind, right_kind]),
            }
        }
        _ => mixed_kind(false),
    }
}

#[inline]
pub fn get_array_kind<F>(elements: &TokenSeparatedSequence<ArrayElement>, get_expression_kind: F) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    let mut known_size = 0;
    let mut properties = Vec::new();
    let mut key_kinds = Vec::new();
    let mut value_kinds = Vec::new();
    let mut has_key_value_pairs = false;

    for element in elements.iter() {
        match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                has_key_value_pairs = true;

                let mut key_kind = get_expression_kind(&key_value_array_element.key);
                let value_kind = get_expression_kind(&key_value_array_element.value);

                if matches!(key_kind, TypeKind::Never) || matches!(value_kind, TypeKind::Never) {
                    return never_kind();
                }

                match key_kind {
                    TypeKind::Never => {
                        return never_kind();
                    }
                    TypeKind::Value(ValueTypeKind::Integer { value }) => {
                        properties.push(integer_shape_property(value as isize, value_kind.clone(), false));
                    }
                    TypeKind::Value(ValueTypeKind::String { value, .. }) => {
                        properties.push(string_shape_property(value, value_kind.clone(), false));
                    }
                    k if !k.is_string().or(k.is_integer()).is_true() => {
                        key_kind = array_key_kind();
                    }
                    _ => {}
                };

                if matches!(value_kind, TypeKind::Never) {
                    return never_kind();
                }

                if !key_kinds.contains(&key_kind) {
                    key_kinds.push(key_kind);
                }

                if !value_kinds.contains(&value_kind) {
                    value_kinds.push(value_kind);
                }

                known_size += 1;
            }
            ArrayElement::Value(value_array_element) => {
                let value_kind = get_expression_kind(&value_array_element.value);

                if matches!(value_kind, TypeKind::Never) {
                    return never_kind();
                }

                properties.push(indexed_shape_property(value_kind.clone(), false));

                if !value_kinds.contains(&value_kind) {
                    value_kinds.push(value_kind);
                }

                known_size += 1;
            }
            _ => {
                // regardless of what we know, we can't be sure of the kind
                // return array<array-key, mixed>
                return array_kind(array_key_kind(), mixed_kind(false), None);
            }
        }
    }

    if known_size == 0 {
        return array_kind(array_key_kind(), mixed_kind(false), Some(0));
    }

    let value_kind = if value_kinds.len() == 1 { value_kinds.swap_remove(0) } else { union_kind(value_kinds) };

    if properties.len() == known_size {
        // yay! we know all the elements of the array! we can return a shape!
        return array_shape_kind(properties, None);
    }

    if !has_key_value_pairs {
        return non_empty_list_kind(value_kind, Some(known_size));
    } else {
        let key_kind = if key_kinds.len() == 1 { key_kinds.swap_remove(0) } else { union_kind(key_kinds) };

        return non_empty_array_kind(key_kind, value_kind, Some(known_size));
    }
}

#[inline]
pub fn get_arithmetic_operation_kind<F>(
    interner: &ThreadedInterner,
    arithmetic_operation: &ArithmeticOperation,
    get_expression_kind: F,
) -> TypeKind
where
    F: Fn(&Expression) -> TypeKind,
{
    match arithmetic_operation {
        ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
            match get_expression_kind(&arithmetic_prefix_operation.value) {
                TypeKind::Value(ValueTypeKind::Integer { value }) => match &arithmetic_prefix_operation.operator {
                    ArithmeticPrefixOperator::Increment(_) => value_integer_kind(value.wrapping_add(1)),
                    ArithmeticPrefixOperator::Decrement(_) => value_integer_kind(value.wrapping_sub(1)),
                    ArithmeticPrefixOperator::Plus(_) => value_integer_kind(value),
                    ArithmeticPrefixOperator::Minus(_) => value_integer_kind(-value),
                },
                TypeKind::Value(ValueTypeKind::Float { value }) => match &arithmetic_prefix_operation.operator {
                    ArithmeticPrefixOperator::Increment(_) => value_float_kind(value + 1.0),
                    ArithmeticPrefixOperator::Decrement(_) => value_float_kind(value - 1.0),
                    ArithmeticPrefixOperator::Plus(_) => value_float_kind(value),
                    ArithmeticPrefixOperator::Minus(_) => value_float_kind(-value),
                },
                TypeKind::Scalar(ScalarTypeKind::Integer { .. }) => match &arithmetic_prefix_operation.operator {
                    ArithmeticPrefixOperator::Increment(_) | ArithmeticPrefixOperator::Decrement(_) => integer_kind(),
                    ArithmeticPrefixOperator::Plus(_) | ArithmeticPrefixOperator::Minus(_) => integer_kind(),
                },
                TypeKind::Scalar(ScalarTypeKind::Float) => float_kind(),
                TypeKind::Scalar(ScalarTypeKind::NumericString) => {
                    // If the operand is a non-empty string, the result is an integer
                    integer_kind()
                }
                kind if is_gmp_or_bcmath_number(interner, &kind) => kind,
                _ => never_kind(),
            }
        }
        ArithmeticOperation::Infix(arithmetic_infix_operation) => {
            let lhs_kind = get_expression_kind(&arithmetic_infix_operation.lhs);
            let rhs_kind = get_expression_kind(&arithmetic_infix_operation.rhs);

            match (&lhs_kind, &rhs_kind) {
                (TypeKind::Never, _) | (_, TypeKind::Never) => {
                    // If either operand is Never, the result is Never
                    never_kind()
                }
                (
                    TypeKind::Value(ValueTypeKind::Integer { value: lhs }),
                    TypeKind::Value(ValueTypeKind::Integer { value: rhs_value }),
                ) => {
                    match &arithmetic_infix_operation.operator {
                        ArithmeticInfixOperator::Addition(_) => value_integer_kind(lhs.wrapping_add(*rhs_value)),
                        ArithmeticInfixOperator::Subtraction(_) => value_integer_kind(lhs.wrapping_sub(*rhs_value)),
                        ArithmeticInfixOperator::Multiplication(_) => value_integer_kind(lhs.wrapping_mul(*rhs_value)),
                        ArithmeticInfixOperator::Division(_) => {
                            if *rhs_value != 0 {
                                if lhs % rhs_value == 0 {
                                    // Division is exact, result is integer
                                    value_integer_kind(lhs / rhs_value)
                                } else {
                                    // Division results in float
                                    value_float_kind(OrderedFloat((*lhs as f64) / (*rhs_value as f64)))
                                }
                            } else {
                                // Division by zero; in PHP, this throws, resulting in `never`
                                never_kind()
                            }
                        }
                        ArithmeticInfixOperator::Modulo(_) => {
                            if *rhs_value != 0 {
                                value_integer_kind(lhs % rhs_value)
                            } else {
                                // Modulo by zero; in PHP, this throws, resulting in `never`
                                never_kind()
                            }
                        }
                        ArithmeticInfixOperator::Exponentiation(_) => {
                            // Exponentiation of integers
                            let base = *lhs as f64;
                            let exponent = *rhs_value as f64;
                            let result = base.powf(exponent);

                            if result.fract() == 0.0 && result >= i64::MIN as f64 && result <= i64::MAX as f64 {
                                // Result is an integer
                                value_integer_kind(result as i64)
                            } else {
                                // Result is a float
                                value_float_kind(OrderedFloat(result))
                            }
                        }
                    }
                }
                // Both operands are numeric literals (integer or float)
                (lhs_value_kind, rhs_value_kind)
                    if is_numeric_value_kind(lhs_value_kind) && is_numeric_value_kind(rhs_value_kind) =>
                {
                    let lhs_value = extract_numeric_value(lhs_value_kind);
                    let rhs_value = extract_numeric_value(rhs_value_kind);

                    match (lhs_value, rhs_value) {
                        (Some(lhs_num), Some(rhs_num)) => {
                            let result = match &arithmetic_infix_operation.operator {
                                ArithmeticInfixOperator::Addition(_) => lhs_num + rhs_num,
                                ArithmeticInfixOperator::Subtraction(_) => lhs_num - rhs_num,
                                ArithmeticInfixOperator::Multiplication(_) => lhs_num * rhs_num,
                                ArithmeticInfixOperator::Division(_) => {
                                    if rhs_num != 0.0 {
                                        lhs_num / rhs_num
                                    } else {
                                        return never_kind(); // Division by zero
                                    }
                                }
                                ArithmeticInfixOperator::Modulo(_) => {
                                    if rhs_num != 0.0 {
                                        // Convert operands to integers by truncating the decimal part
                                        let lhs_int = lhs_num.0.trunc() as i64;
                                        let rhs_int = rhs_num.0.trunc() as i64;

                                        if rhs_int != 0 {
                                            let result = lhs_int % rhs_int;
                                            return value_integer_kind(result);
                                        } else {
                                            return never_kind(); // Modulo by zero
                                        }
                                    } else {
                                        return never_kind(); // Modulo by zero
                                    }
                                }
                                ArithmeticInfixOperator::Exponentiation(_) => OrderedFloat(lhs_num.powf(*rhs_num)),
                            };

                            value_float_kind(result)
                        }
                        _ => float_kind(),
                    }
                }
                // One or both operands are not literals
                _ => resolve_numeric_operation_kind(
                    interner,
                    lhs_kind.clone(),
                    rhs_kind.clone(),
                    &arithmetic_infix_operation.operator,
                ),
            }
        }
        ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
            get_expression_kind(&arithmetic_postfix_operation.value)
        }
    }
}

#[inline]
pub fn get_literal_kind(interner: &ThreadedInterner, literal: &Literal) -> TypeKind {
    match &literal {
        Literal::String(string) => get_literal_string_value_kind(interner, string.value, true),
        Literal::Integer(integer) => {
            if let Some(value) = integer.value {
                if value > i64::MAX as u64 {
                    integer_kind()
                } else {
                    // we can safely cast `value` to an `i64`
                    value_integer_kind(value as i64)
                }
            } else {
                integer_kind()
            }
        }
        Literal::Float(literal_float) => value_float_kind(literal_float.value),
        Literal::True(_) => true_kind(),
        Literal::False(_) => false_kind(),
        Literal::Null(_) => null_kind(),
    }
}

pub fn get_literal_string_value_kind(
    interner: &ThreadedInterner,
    string: StringIdentifier,
    remove_quotes: bool,
) -> TypeKind {
    if string.is_empty() {
        return value_string_kind(string, 0, Trinary::False, Trinary::False, Trinary::False, Trinary::False);
    }

    let mut value = interner.lookup(&string);
    if remove_quotes {
        value = &value[1..value.len() - 1];
    }

    if value.is_empty() {
        return value_string_kind(
            StringIdentifier::empty(),
            0,
            Trinary::False,
            Trinary::False,
            Trinary::False,
            Trinary::False,
        );
    }

    let mut length = 0;
    let mut is_uppercase = Trinary::Maybe;
    let mut is_lowercase = Trinary::Maybe;
    let mut is_ascii_uppercase = Trinary::Maybe;
    let mut is_ascii_lowercase = Trinary::Maybe;

    for c in value.chars() {
        length += 1;

        is_uppercase &= c.is_uppercase();
        is_lowercase &= c.is_lowercase();
        is_ascii_uppercase &= c.is_ascii_uppercase();
        is_ascii_lowercase &= c.is_ascii_lowercase();
    }

    value_string_kind(
        interner.intern(value),
        length,
        is_uppercase,
        is_ascii_uppercase,
        is_lowercase,
        is_ascii_lowercase,
    )
}

// Check if a TypeKind is a numeric value kind (integer or float literal)
#[inline]
pub fn is_numeric_value_kind(kind: &TypeKind) -> bool {
    matches!(kind, TypeKind::Value(ValueTypeKind::Integer { .. }) | TypeKind::Value(ValueTypeKind::Float { .. }))
}

// Extract the numeric value (as OrderedFloat<f64>) from a TypeKind
#[inline]
pub fn extract_numeric_value(kind: &TypeKind) -> Option<OrderedFloat<f64>> {
    match kind {
        TypeKind::Value(ValueTypeKind::Integer { value }) => Some(OrderedFloat(*value as f64)),
        TypeKind::Value(ValueTypeKind::Float { value }) => Some(*value),
        _ => None,
    }
}

// Infer the resulting type of a numeric operation when operands are not literals
#[inline]
pub fn resolve_numeric_operation_kind(
    interner: &ThreadedInterner,
    lhs_kind: TypeKind,
    rhs_kind: TypeKind,
    operator: &ArithmeticInfixOperator,
) -> TypeKind {
    match (lhs_kind, rhs_kind) {
        // If either operand is Never, the result is Never
        (TypeKind::Never, _) | (_, TypeKind::Never) => never_kind(),
        (TypeKind::Scalar(ScalarTypeKind::Integer { .. }), TypeKind::Scalar(ScalarTypeKind::Integer { .. })) => {
            match operator {
                ArithmeticInfixOperator::Modulo(_) => integer_kind(),
                ArithmeticInfixOperator::Division(_) => union_kind(vec![integer_kind(), float_kind()]),
                ArithmeticInfixOperator::Exponentiation(_) => union_kind(vec![integer_kind(), float_kind()]),
                _ => integer_kind(),
            }
        }
        (TypeKind::Scalar(ScalarTypeKind::Float), TypeKind::Scalar(ScalarTypeKind::Float))
        | (TypeKind::Scalar(ScalarTypeKind::Integer { .. }), TypeKind::Scalar(ScalarTypeKind::Float))
        | (TypeKind::Scalar(ScalarTypeKind::Float), TypeKind::Scalar(ScalarTypeKind::Integer { .. })) => float_kind(),
        (kind, _) if is_gmp_or_bcmath_number(interner, &kind) => kind,
        (_, kind) if is_gmp_or_bcmath_number(interner, &kind) => kind,
        (left, right)
            if left.is_object()
                || left.is_resource()
                || left.is_array()
                || right.is_object()
                || right.is_resource()
                || right.is_array() =>
        {
            never_kind()
        }
        _ => union_kind(vec![integer_kind(), float_kind()]),
    }
}

#[inline]
pub fn can_extract_literal_value(kind: &TypeKind) -> bool {
    match kind {
        TypeKind::Value(ValueTypeKind::Integer { .. })
        | TypeKind::Value(ValueTypeKind::Float { .. })
        | TypeKind::Value(ValueTypeKind::True)
        | TypeKind::Value(ValueTypeKind::False) => true,
        _ => false,
    }
}

#[inline]
pub fn extract_literal_value(kind: &TypeKind) -> OrderedFloat<f64> {
    match kind {
        TypeKind::Value(ValueTypeKind::Integer { value }) => OrderedFloat(*value as f64),
        TypeKind::Value(ValueTypeKind::Float { value }) => *value,
        TypeKind::Value(ValueTypeKind::True) => OrderedFloat(1.0),
        TypeKind::Value(ValueTypeKind::False) => OrderedFloat(0.0),
        _ => unreachable!(),
    }
}

#[inline]
fn is_gmp_or_bcmath_number(interner: &ThreadedInterner, kind: &TypeKind) -> bool {
    if let TypeKind::Object(ObjectTypeKind::NamedObject { name, .. }) = kind {
        let class = interner.lookup(&name);

        class.eq_ignore_ascii_case("gmp") || class.eq_ignore_ascii_case("bcmath\\number")
    } else {
        false
    }
}
