use crate::ast::*;

/// Get the assignment operation from an expression.
///
/// This function will recursively search through the expression and its children to find
///  the first assignment operation.
///
/// If no assignment operation is found, it will return `None`.
#[inline]
pub fn get_assignment_from_expression<'a, 'arena>(
    expression: &'a Expression<'arena>,
) -> Option<&'a Assignment<'arena>> {
    match &expression {
        Expression::Assignment(assignment_operation) => Some(assignment_operation),
        Expression::Parenthesized(parenthesized) => get_assignment_from_expression(parenthesized.expression),
        Expression::Binary(operation) => {
            get_assignment_from_expression(operation.lhs).or_else(|| get_assignment_from_expression(operation.rhs))
        }
        Expression::UnaryPrefix(operation) => get_assignment_from_expression(operation.operand),
        Expression::UnaryPostfix(operation) => get_assignment_from_expression(operation.operand),
        Expression::Conditional(conditional) => get_assignment_from_expression(conditional.condition)
            .or_else(|| conditional.then.as_ref().and_then(|then| get_assignment_from_expression(then)))
            .or_else(|| get_assignment_from_expression(conditional.r#else)),
        Expression::Array(array) => array.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::LegacyArray(legacy_array) => legacy_array.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::List(list) => list.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::ArrayAccess(array_access) => get_assignment_from_expression(array_access.array)
            .or_else(|| get_assignment_from_expression(array_access.index)),
        Expression::ArrayAppend(array_append) => get_assignment_from_expression(array_append.array),
        Expression::Match(r#match) => get_assignment_from_expression(r#match.expression).or_else(|| {
            r#match.arms.iter().find_map(|arm| match arm {
                MatchArm::Expression(match_expression_arm) => match_expression_arm
                    .conditions
                    .iter()
                    .find_map(|condition| get_assignment_from_expression(condition))
                    .or_else(|| get_assignment_from_expression(match_expression_arm.expression)),
                MatchArm::Default(match_default_arm) => get_assignment_from_expression(match_default_arm.expression),
            })
        }),
        Expression::Yield(r#yield) => match r#yield {
            Yield::Value(yield_value) => {
                yield_value.value.as_ref().and_then(|value| get_assignment_from_expression(value))
            }
            Yield::Pair(yield_pair) => get_assignment_from_expression(yield_pair.key)
                .or_else(|| get_assignment_from_expression(yield_pair.value)),
            Yield::From(yield_from) => get_assignment_from_expression(yield_from.iterator),
        },
        Expression::Construct(construct) => match construct {
            Construct::Isset(isset_construct) => {
                isset_construct.values.iter().find_map(|v| get_assignment_from_expression(v))
            }
            Construct::Empty(empty_construct) => get_assignment_from_expression(empty_construct.value),
            Construct::Eval(eval_construct) => get_assignment_from_expression(eval_construct.value),
            Construct::Include(include_construct) => get_assignment_from_expression(include_construct.value),
            Construct::IncludeOnce(include_once_construct) => {
                get_assignment_from_expression(include_once_construct.value)
            }
            Construct::Require(require_construct) => get_assignment_from_expression(require_construct.value),
            Construct::RequireOnce(require_once_construct) => {
                get_assignment_from_expression(require_once_construct.value)
            }
            Construct::Print(print_construct) => get_assignment_from_expression(print_construct.value),
            Construct::Exit(exit_construct) => exit_construct.arguments.as_ref().and_then(|arguments| {
                arguments.arguments.iter().find_map(|argument| {
                    get_assignment_from_expression(match &argument {
                        Argument::Positional(positional_argument) => &positional_argument.value,
                        Argument::Named(named_argument) => &named_argument.value,
                    })
                })
            }),
            Construct::Die(die_construct) => die_construct.arguments.as_ref().and_then(|arguments| {
                arguments.arguments.iter().find_map(|argument| {
                    get_assignment_from_expression(match &argument {
                        Argument::Positional(positional_argument) => &positional_argument.value,
                        Argument::Named(named_argument) => &named_argument.value,
                    })
                })
            }),
        },
        Expression::Throw(throw) => get_assignment_from_expression(throw.exception),
        Expression::Clone(clone) => get_assignment_from_expression(clone.object),
        Expression::Call(call) => match &call {
            Call::Function(function_call) => get_assignment_from_expression(function_call.function).or_else(|| {
                function_call.argument_list.arguments.iter().find_map(|argument| match &argument {
                    Argument::Positional(positional_argument) => {
                        get_assignment_from_expression(&positional_argument.value)
                    }
                    Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                })
            }),
            Call::Method(method_call) => get_assignment_from_expression(method_call.object)
                .or_else(|| match &method_call.method {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
                .or_else(|| {
                    method_call.argument_list.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                }),
            Call::NullSafeMethod(null_safe_method_call) => get_assignment_from_expression(null_safe_method_call.object)
                .or_else(|| match &null_safe_method_call.method {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
                .or_else(|| {
                    null_safe_method_call.argument_list.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                }),
            Call::StaticMethod(static_method_call) => get_assignment_from_expression(static_method_call.class)
                .or_else(|| match &static_method_call.method {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
                .or_else(|| {
                    static_method_call.argument_list.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                }),
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                get_assignment_from_expression(property_access.object).or_else(|| match &property_access.property {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                get_assignment_from_expression(null_safe_property_access.object).or_else(|| {
                    match &null_safe_property_access.property {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
            Access::StaticProperty(static_property_access) => {
                get_assignment_from_expression(static_property_access.class)
            }
            Access::ClassConstant(class_constant_access) => get_assignment_from_expression(class_constant_access.class)
                .or_else(|| match &class_constant_access.constant {
                    ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(class_like_member_expression_selector.expression)
                    }
                    _ => None,
                }),
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation {
            ClosureCreation::Function(function_closure_creation) => {
                get_assignment_from_expression(function_closure_creation.function)
            }
            ClosureCreation::Method(method_closure_creation) => get_assignment_from_expression(
                method_closure_creation.object,
            )
            .or_else(|| match &method_closure_creation.method {
                ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                    get_assignment_from_expression(class_like_member_expression_selector.expression)
                }
                _ => None,
            }),
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                get_assignment_from_expression(static_method_closure_creation.class).or_else(|| {
                    match &static_method_closure_creation.method {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
        },
        Expression::Instantiation(instantiation) => get_assignment_from_expression(instantiation.class).or_else(|| {
            instantiation.argument_list.as_ref().and_then(|arguments| {
                arguments.arguments.iter().find_map(|argument| match &argument {
                    Argument::Positional(positional_argument) => {
                        get_assignment_from_expression(&positional_argument.value)
                    }
                    Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                })
            })
        }),
        _ => None,
    }
}
