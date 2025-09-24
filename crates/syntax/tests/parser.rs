mod runner {
    use std::borrow::Cow;

    use bumpalo::Bump;

    use mago_database::file::File;

    use mago_syntax::ast::*;
    use mago_syntax::parser::parse_file;

    pub fn smoke_test(name: &'static str, code: &'static str) {
        let arena = Bump::new();
        let file = File::ephemeral(Cow::Borrowed(name), Cow::Borrowed(code));

        let (_program, error) = parse_file(&arena, &file);
        if let Some(parse_error) = error {
            panic!("Test case '{}' failed to parse. Error: {}", name, parse_error);
        }
    }

    pub fn run_expression_test(name: &'static str, expression: &'static str, expected: &'static str) {
        fn format_variable(var: &Variable<'_>) -> String {
            match var {
                Variable::Direct(direct_variable) => direct_variable.name.to_string(),
                Variable::Indirect(indirect_variable) => {
                    format!("${{{}}}", format_expression(indirect_variable.expression))
                }
                Variable::Nested(nested_variable) => {
                    format!("${}", format_variable(nested_variable.variable))
                }
            }
        }

        fn format_member_selector(selector: &ClassLikeMemberSelector) -> String {
            match selector {
                ClassLikeMemberSelector::Identifier(identifier) => identifier.value.to_string(),
                ClassLikeMemberSelector::Variable(variable) => format_variable(variable),
                ClassLikeMemberSelector::Expression(s) => {
                    format!("{{{}}}", format_expression(s.expression))
                }
            }
        }

        fn format_constant_selector(selector: &ClassLikeConstantSelector) -> String {
            match selector {
                ClassLikeConstantSelector::Identifier(local_identifier) => local_identifier.value.to_string(),
                ClassLikeConstantSelector::Expression(s) => {
                    format!("{{{}}}", format_expression(s.expression))
                }
            }
        }

        fn format_expression(expr: &Expression<'_>) -> String {
            match expr {
                Expression::Parenthesized(parenthesized) => format_expression(parenthesized.expression),
                Expression::Variable(variable) => format_variable(variable),
                Expression::Binary(binary) => {
                    format!(
                        "({} {} {})",
                        format_expression(binary.lhs),
                        binary.operator.as_str(),
                        format_expression(binary.rhs)
                    )
                }
                Expression::UnaryPrefix(unary_prefix) => {
                    format!("({} {})", unary_prefix.operator.as_str(), format_expression(unary_prefix.operand))
                }
                Expression::UnaryPostfix(unary_postfix) => {
                    format!("({} {})", format_expression(unary_postfix.operand), unary_postfix.operator.as_str())
                }
                Expression::Literal(literal) => match literal {
                    Literal::String(s) => s.raw.to_string(),
                    Literal::Integer(i) => i.raw.to_string(),
                    Literal::Float(f) => f.raw.to_string(),
                    Literal::True(_) => "true".to_string(),
                    Literal::False(_) => "false".to_string(),
                    Literal::Null(_) => "null".to_string(),
                },
                Expression::Assignment(assignment) => {
                    format!(
                        "({} {} {})",
                        format_expression(assignment.lhs),
                        assignment.operator.as_str(),
                        format_expression(assignment.rhs)
                    )
                }
                Expression::Conditional(conditional) => match conditional.then {
                    Some(then) => format!(
                        "( {} ? {} : {} )",
                        format_expression(conditional.condition),
                        format_expression(then),
                        format_expression(conditional.r#else)
                    ),
                    None => format!(
                        "({} ?: {})",
                        format_expression(conditional.condition),
                        format_expression(conditional.r#else)
                    ),
                },
                Expression::ConstantAccess(ConstantAccess { name }) => name.value().to_string(),
                Expression::Identifier(identifier) => identifier.value().to_string(),
                Expression::Construct(Construct::Print(construct)) => {
                    format!("(print {})", format_expression(construct.value))
                }
                Expression::Yield(yield_expr) => match yield_expr {
                    Yield::Value(yield_value) => match yield_value.value {
                        Some(value) => format!("(yield {})", format_expression(value)),
                        None => "yield".to_string(),
                    },
                    Yield::Pair(yield_pair) => format!(
                        "(yield {} => {})",
                        format_expression(yield_pair.key),
                        format_expression(yield_pair.value)
                    ),
                    Yield::From(yield_from) => format!("(yield from {})", format_expression(yield_from.iterator)),
                },
                Expression::Instantiation(instantiation) => {
                    format!("(new {})", format_expression(instantiation.class))
                }
                Expression::Clone(clone) => {
                    format!("(clone {})", format_expression(clone.object))
                }
                Expression::Throw(throw) => {
                    format!("(throw {})", format_expression(throw.exception))
                }
                Expression::Construct(Construct::Require(require_construct)) => {
                    format!("(require {})", format_expression(require_construct.value))
                }
                Expression::Construct(Construct::RequireOnce(require_once_construct)) => {
                    format!("(require_once {})", format_expression(require_once_construct.value))
                }
                Expression::Construct(Construct::Include(include_construct)) => {
                    format!("(include {})", format_expression(include_construct.value))
                }
                Expression::Construct(Construct::IncludeOnce(include_once_construct)) => {
                    format!("(include_once {})", format_expression(include_once_construct.value))
                }
                Expression::Call(call) => match call {
                    Call::Function(function_call) => format!("({}())", format_expression(function_call.function)),
                    Call::Method(method_call) => {
                        format!(
                            "({}->{}())",
                            format_expression(method_call.object),
                            format_member_selector(&method_call.method),
                        )
                    }
                    Call::NullSafeMethod(null_safe_method_call) => {
                        format!(
                            "({}?->{}())",
                            format_expression(null_safe_method_call.object),
                            format_member_selector(&null_safe_method_call.method),
                        )
                    }
                    Call::StaticMethod(static_method_call) => {
                        format!(
                            "({}::{}())",
                            format_expression(static_method_call.class),
                            format_member_selector(&static_method_call.method),
                        )
                    }
                },
                Expression::Access(access) => match access {
                    Access::Property(property_access) => {
                        format!(
                            "({}->{})",
                            format_expression(property_access.object),
                            format_member_selector(&property_access.property),
                        )
                    }
                    Access::NullSafeProperty(null_safe_property_access) => {
                        format!(
                            "({}?->{})",
                            format_expression(null_safe_property_access.object),
                            format_member_selector(&null_safe_property_access.property),
                        )
                    }
                    Access::StaticProperty(static_property_access) => {
                        format!(
                            "({}::{})",
                            format_expression(static_property_access.class),
                            format_variable(&static_property_access.property),
                        )
                    }
                    Access::ClassConstant(class_constant_access) => {
                        format!(
                            "({}::{})",
                            format_expression(class_constant_access.class),
                            format_constant_selector(&class_constant_access.constant),
                        )
                    }
                },
                _ => {
                    let expression_kind = Node::Expression(expr)
                        .children()
                        .first()
                        .map(|t| t.kind().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());

                    panic!("unsupported expression kind for formatting: {}", expression_kind);
                }
            }
        }

        let code = format!("<?php {};", expression);
        let arena = Bump::new();
        let file = File::ephemeral(Cow::Borrowed(name), Cow::Owned(code));

        let (program, error) = parse_file(&arena, &file);
        if let Some(parse_error) = error {
            panic!("Test case '{}' failed to parse. Error: {}", name, parse_error);
        }

        let statement = program.statements.get(1).expect("Expected an expression statement here");
        let Statement::Expression(expression) = statement else {
            panic!("Expected an expression statement, found `{:#?}`", statement);
        };

        let formatted_ast = format_expression(expression.expression);

        assert_eq!(formatted_ast, expected, "Test case '{}' failed. Expression does not match expected output.", name);
    }
}

mod parser {
    macro_rules! test_expression {
        ($name:ident, $expression:expr, $expected:expr) => {
            #[test]
            fn $name() {
                crate::runner::run_expression_test(stringify!($name), $expression, $expected);
            }
        };
    }

    macro_rules! smoke_test {
        ($name:ident, $expression:expr) => {
            #[test]
            fn $name() {
                crate::runner::smoke_test(stringify!($name), $expression);
            }
        };
    }

    test_expression!(assign_ref_static_call, "$a = &B::c()", "($a = (& (B::c())))");
    test_expression!(assign_ref_func_call, "$a = &b()", "($a = (& (b())))");
    test_expression!(assign_ref_method_call, "$a = &$b->c()", "($a = (& ($b->c())))");
    test_expression!(assign_ref_null_method_call, "$a = &$b?->c()", "($a = (& ($b?->c())))");
    test_expression!(unary_minus_vs_mul, "$a = -$b * $c", "($a = ((- $b) * $c))");
    test_expression!(unary_minus_vs_add, "$a = -$b + $c", "($a = ((- $b) + $c))");
    test_expression!(unary_minus_vs_div, "$a = -$b / $c", "($a = ((- $b) / $c))");
    test_expression!(unary_minus_vs_sub, "$a = -$b - $c", "($a = ((- $b) - $c))");
    test_expression!(unary_minus_vs_mod, "$a = -$b % $c", "($a = ((- $b) % $c))");
    test_expression!(unary_minus_vs_pow, "$a = -$b ** $c", "($a = (- ($b ** $c)))");
    test_expression!(unary_minus_vs_shift_left, "$a = -$b << $c", "($a = ((- $b) << $c))");
    test_expression!(unary_minus_vs_shift_right, "$a = -$b >> $c", "($a = ((- $b) >> $c))");
    test_expression!(unary_minus_vs_bitwise_and, "$a = -$b & $c", "($a = ((- $b) & $c))");
    test_expression!(unary_minus_vs_bitwise_or, "$a = -$b | $c", "($a = ((- $b) | $c))");
    test_expression!(unary_minus_vs_bitwise_xor, "$a = -$b ^ $c", "($a = ((- $b) ^ $c))");
    test_expression!(unary_minus_vs_less_than, "$a = -$b < $c", "($a = ((- $b) < $c))");
    test_expression!(unary_minus_vs_less_than_equal, "$a = -$b <= $c", "($a = ((- $b) <= $c))");
    test_expression!(unary_minus_vs_greater_than, "$a = -$b > $c", "($a = ((- $b) > $c))");
    test_expression!(unary_minus_vs_greater_than_equal, "$a = -$b >= $c", "($a = ((- $b) >= $c))");
    test_expression!(unary_minus_vs_equal, "$a = -$b == $c", "($a = ((- $b) == $c))");
    test_expression!(unary_minus_vs_identical, "$a = -$b === $c", "($a = ((- $b) === $c))");
    test_expression!(unary_minus_vs_not_equal, "$a = -$b != $c", "($a = ((- $b) != $c))");
    test_expression!(unary_minus_vs_not_identical, "$a = -$b !== $c", "($a = ((- $b) !== $c))");
    test_expression!(unary_minus_vs_spaceship, "$a = -$b <=> $c", "($a = ((- $b) <=> $c))");
    test_expression!(unary_minus_vs_coalesce, "$a = -$b ?? $c", "($a = ((- $b) ?? $c))");
    test_expression!(unary_minus_vs_logical_and_word, "$a = -$b and $c", "(($a = (- $b)) and $c)");
    test_expression!(unary_minus_vs_logical_or_word, "$a = -$b or $c", "(($a = (- $b)) or $c)");
    test_expression!(unary_minus_vs_logical_xor_word, "$a = -$b xor $c", "(($a = (- $b)) xor $c)");
    test_expression!(unary_minus_vs_logical_and_op, "$a = -$b && $c", "($a = ((- $b) && $c))");
    test_expression!(unary_minus_vs_logical_or_op, "$a = -$b || $c", "($a = ((- $b) || $c))");
    test_expression!(unary_minus_vs_ternary, "$a = -$b ? $c : $d", "($a = ( (- $b) ? $c : $d ))");
    test_expression!(error_control_vs_mul, "$a = @$b * $c", "($a = ((@ $b) * $c))");
    test_expression!(error_control_vs_add, "$a = @$b + $c", "($a = ((@ $b) + $c))");
    test_expression!(error_control_vs_div, "$a = @$b / $c", "($a = ((@ $b) / $c))");
    test_expression!(error_control_vs_sub, "$a = @$b - $c", "($a = ((@ $b) - $c))");
    test_expression!(error_control_vs_mod, "$a = @$b % $c", "($a = ((@ $b) % $c))");
    test_expression!(error_control_vs_pow, "$a = @$b ** $c", "($a = (@ ($b ** $c)))");
    test_expression!(error_control_vs_shift_left, "$a = @$b << $c", "($a = ((@ $b) << $c))");
    test_expression!(error_control_vs_shift_right, "$a = @$b >> $c", "($a = ((@ $b) >> $c))");
    test_expression!(error_control_vs_bitwise_and, "$a = @$b & $c", "($a = ((@ $b) & $c))");
    test_expression!(error_control_vs_bitwise_or, "$a = @$b | $c", "($a = ((@ $b) | $c))");
    test_expression!(error_control_vs_bitwise_xor, "$a = @$b ^ $c", "($a = ((@ $b) ^ $c))");
    test_expression!(error_control_vs_less_than, "$a = @$b < $c", "($a = ((@ $b) < $c))");
    test_expression!(error_control_vs_less_than_equal, "$a = @$b <= $c", "($a = ((@ $b) <= $c))");
    test_expression!(error_control_vs_greater_than, "$a = @$b > $c", "($a = ((@ $b) > $c))");
    test_expression!(error_control_vs_greater_than_equal, "$a = @$b >= $c", "($a = ((@ $b) >= $c))");
    test_expression!(error_control_vs_equal, "$a = @$b == $c", "($a = ((@ $b) == $c))");
    test_expression!(error_control_vs_identical, "$a = @$b === $c", "($a = ((@ $b) === $c))");
    test_expression!(error_control_vs_not_equal, "$a = @$b != $c", "($a = ((@ $b) != $c))");
    test_expression!(error_control_vs_not_identical, "$a = @$b !== $c", "($a = ((@ $b) !== $c))");
    test_expression!(error_control_vs_spaceship, "$a = @$b <=> $c", "($a = ((@ $b) <=> $c))");
    test_expression!(error_control_vs_coalesce, "$a = @$b ?? $c", "($a = ((@ $b) ?? $c))");
    test_expression!(error_control_vs_logical_and_word, "$a = @$b and $c", "(($a = (@ $b)) and $c)");
    test_expression!(error_control_vs_logical_or_word, "$a = @$b or $c", "(($a = (@ $b)) or $c)");
    test_expression!(error_control_vs_logical_xor_word, "$a = @$b xor $c", "(($a = (@ $b)) xor $c)");
    test_expression!(error_control_vs_logical_and_op, "$a = @$b && $c", "($a = ((@ $b) && $c))");
    test_expression!(error_control_vs_logical_or_op, "$a = @$b || $c", "($a = ((@ $b) || $c))");
    test_expression!(error_control_vs_ternary, "$a = @$b ? $c : $d", "($a = ( (@ $b) ? $c : $d ))");
    test_expression!(by_ref_vs_mul, "$a = &$b * $c", "(($a = (& $b)) * $c)");
    test_expression!(by_ref_vs_add, "$a = &$b + $c", "(($a = (& $b)) + $c)");
    test_expression!(by_ref_vs_div, "$a = &$b / $c", "(($a = (& $b)) / $c)");
    test_expression!(by_ref_vs_sub, "$a = &$b - $c", "(($a = (& $b)) - $c)");
    test_expression!(by_ref_vs_mod, "$a = &$b % $c", "(($a = (& $b)) % $c)");
    test_expression!(by_ref_vs_pow, "$a = &$b ** $c", "(($a = (& $b)) ** $c)");
    test_expression!(by_ref_vs_shift_left, "$a = &$b << $c", "(($a = (& $b)) << $c)");
    test_expression!(by_ref_vs_shift_right, "$a = &$b >> $c", "(($a = (& $b)) >> $c)");
    test_expression!(by_ref_vs_bitwise_and, "$a = &$b & $c", "(($a = (& $b)) & $c)");
    test_expression!(by_ref_vs_bitwise_or, "$a = &$b | $c", "(($a = (& $b)) | $c)");
    test_expression!(by_ref_vs_bitwise_xor, "$a = &$b ^ $c", "(($a = (& $b)) ^ $c)");
    test_expression!(by_ref_vs_less_than, "$a = &$b < $c", "(($a = (& $b)) < $c)");
    test_expression!(by_ref_vs_less_than_equal, "$a = &$b <= $c", "(($a = (& $b)) <= $c)");
    test_expression!(by_ref_vs_greater_than, "$a = &$b > $c", "(($a = (& $b)) > $c)");
    test_expression!(by_ref_vs_greater_than_equal, "$a = &$b >= $c", "(($a = (& $b)) >= $c)");
    test_expression!(by_ref_vs_equal, "$a = &$b == $c", "(($a = (& $b)) == $c)");
    test_expression!(by_ref_vs_identical, "$a = &$b === $c", "(($a = (& $b)) === $c)");
    test_expression!(by_ref_vs_not_equal, "$a = &$b != $c", "(($a = (& $b)) != $c)");
    test_expression!(by_ref_vs_not_identical, "$a = &$b !== $c", "(($a = (& $b)) !== $c)");
    test_expression!(by_ref_vs_spaceship, "$a = &$b <=> $c", "(($a = (& $b)) <=> $c)");
    test_expression!(by_ref_vs_coalesce, "$a = &$b ?? $c", "(($a = (& $b)) ?? $c)");
    test_expression!(by_ref_vs_logical_and_word, "$a = &$b and $c", "(($a = (& $b)) and $c)");
    test_expression!(by_ref_vs_logical_or_word, "$a = &$b or $c", "(($a = (& $b)) or $c)");
    test_expression!(by_ref_vs_logical_xor_word, "$a = &$b xor $c", "(($a = (& $b)) xor $c)");
    test_expression!(by_ref_vs_logical_and_op, "$a = &$b && $c", "(($a = (& $b)) && $c)");
    test_expression!(by_ref_vs_logical_or_op, "$a = &$b || $c", "(($a = (& $b)) || $c)");
    test_expression!(by_ref_vs_ternary, "$a = &$b ? $c : $d", "( ($a = (& $b)) ? $c : $d )");
    test_expression!(pre_inc_vs_mul, "$a = ++$b * $c", "($a = ((++ $b) * $c))");
    test_expression!(pre_inc_vs_add, "$a = ++$b + $c", "($a = ((++ $b) + $c))");
    test_expression!(pre_inc_vs_div, "$a = ++$b / $c", "($a = ((++ $b) / $c))");
    test_expression!(pre_inc_vs_sub, "$a = ++$b - $c", "($a = ((++ $b) - $c))");
    test_expression!(pre_inc_vs_mod, "$a = ++$b % $c", "($a = ((++ $b) % $c))");
    test_expression!(pre_inc_vs_pow, "$a = ++$b ** $c", "($a = ((++ $b) ** $c))");
    test_expression!(pre_inc_vs_shift_left, "$a = ++$b << $c", "($a = ((++ $b) << $c))");
    test_expression!(pre_inc_vs_shift_right, "$a = ++$b >> $c", "($a = ((++ $b) >> $c))");
    test_expression!(pre_inc_vs_bitwise_and, "$a = ++$b & $c", "($a = ((++ $b) & $c))");
    test_expression!(pre_inc_vs_bitwise_or, "$a = ++$b | $c", "($a = ((++ $b) | $c))");
    test_expression!(pre_inc_vs_bitwise_xor, "$a = ++$b ^ $c", "($a = ((++ $b) ^ $c))");
    test_expression!(pre_inc_vs_less_than, "$a = ++$b < $c", "($a = ((++ $b) < $c))");
    test_expression!(pre_inc_vs_less_than_equal, "$a = ++$b <= $c", "($a = ((++ $b) <= $c))");
    test_expression!(pre_inc_vs_greater_than, "$a = ++$b > $c", "($a = ((++ $b) > $c))");
    test_expression!(pre_inc_vs_greater_than_equal, "$a = ++$b >= $c", "($a = ((++ $b) >= $c))");
    test_expression!(pre_inc_vs_equal, "$a = ++$b == $c", "($a = ((++ $b) == $c))");
    test_expression!(pre_inc_vs_identical, "$a = ++$b === $c", "($a = ((++ $b) === $c))");
    test_expression!(pre_inc_vs_not_equal, "$a = ++$b != $c", "($a = ((++ $b) != $c))");
    test_expression!(pre_inc_vs_not_identical, "$a = ++$b !== $c", "($a = ((++ $b) !== $c))");
    test_expression!(pre_inc_vs_spaceship, "$a = ++$b <=> $c", "($a = ((++ $b) <=> $c))");
    test_expression!(pre_inc_vs_coalesce, "$a = ++$b ?? $c", "($a = ((++ $b) ?? $c))");
    test_expression!(pre_inc_vs_logical_and_word, "$a = ++$b and $c", "(($a = (++ $b)) and $c)");
    test_expression!(pre_inc_vs_logical_or_word, "$a = ++$b or $c", "(($a = (++ $b)) or $c)");
    test_expression!(pre_inc_vs_logical_xor_word, "$a = ++$b xor $c", "(($a = (++ $b)) xor $c)");
    test_expression!(pre_inc_vs_logical_and_op, "$a = ++$b && $c", "($a = ((++ $b) && $c))");
    test_expression!(pre_inc_vs_logical_or_op, "$a = ++$b || $c", "($a = ((++ $b) || $c))");
    test_expression!(pre_inc_vs_ternary, "$a = ++$b ? $c : $d", "($a = ( (++ $b) ? $c : $d ))");
    test_expression!(pre_dec_vs_mul, "$a = --$b * $c", "($a = ((-- $b) * $c))");
    test_expression!(pre_dec_vs_add, "$a = --$b + $c", "($a = ((-- $b) + $c))");
    test_expression!(pre_dec_vs_div, "$a = --$b / $c", "($a = ((-- $b) / $c))");
    test_expression!(pre_dec_vs_sub, "$a = --$b - $c", "($a = ((-- $b) - $c))");
    test_expression!(pre_dec_vs_mod, "$a = --$b % $c", "($a = ((-- $b) % $c))");
    test_expression!(pre_dec_vs_pow, "$a = --$b ** $c", "($a = ((-- $b) ** $c))");
    test_expression!(pre_dec_vs_shift_left, "$a = --$b << $c", "($a = ((-- $b) << $c))");
    test_expression!(pre_dec_vs_shift_right, "$a = --$b >> $c", "($a = ((-- $b) >> $c))");
    test_expression!(pre_dec_vs_bitwise_and, "$a = --$b & $c", "($a = ((-- $b) & $c))");
    test_expression!(pre_dec_vs_bitwise_or, "$a = --$b | $c", "($a = ((-- $b) | $c))");
    test_expression!(pre_dec_vs_bitwise_xor, "$a = --$b ^ $c", "($a = ((-- $b) ^ $c))");
    test_expression!(pre_dec_vs_less_than, "$a = --$b < $c", "($a = ((-- $b) < $c))");
    test_expression!(pre_dec_vs_less_than_equal, "$a = --$b <= $c", "($a = ((-- $b) <= $c))");
    test_expression!(pre_dec_vs_greater_than, "$a = --$b > $c", "($a = ((-- $b) > $c))");
    test_expression!(pre_dec_vs_greater_than_equal, "$a = --$b >= $c", "($a = ((-- $b) >= $c))");
    test_expression!(pre_dec_vs_equal, "$a = --$b == $c", "($a = ((-- $b) == $c))");
    test_expression!(pre_dec_vs_identical, "$a = --$b === $c", "($a = ((-- $b) === $c))");
    test_expression!(pre_dec_vs_not_equal, "$a = --$b != $c", "($a = ((-- $b) != $c))");
    test_expression!(pre_dec_vs_not_identical, "$a = --$b !== $c", "($a = ((-- $b) !== $c))");
    test_expression!(pre_dec_vs_spaceship, "$a = --$b <=> $c", "($a = ((-- $b) <=> $c))");
    test_expression!(pre_dec_vs_coalesce, "$a = --$b ?? $c", "($a = ((-- $b) ?? $c))");
    test_expression!(pre_dec_vs_logical_and_word, "$a = --$b and $c", "(($a = (-- $b)) and $c)");
    test_expression!(pre_dec_vs_logical_or_word, "$a = --$b or $c", "(($a = (-- $b)) or $c)");
    test_expression!(pre_dec_vs_logical_xor_word, "$a = --$b xor $c", "(($a = (-- $b)) xor $c)");
    test_expression!(pre_dec_vs_logical_and_op, "$a = --$b && $c", "($a = ((-- $b) && $c))");
    test_expression!(pre_dec_vs_logical_or_op, "$a = --$b || $c", "($a = ((-- $b) || $c))");
    test_expression!(pre_dec_vs_ternary, "$a = --$b ? $c : $d", "($a = ( (-- $b) ? $c : $d ))");
    test_expression!(not_vs_mul, "$a = !$b * $c", "($a = ((! $b) * $c))");
    test_expression!(not_vs_add, "$a = !$b + $c", "($a = ((! $b) + $c))");
    test_expression!(not_vs_div, "$a = !$b / $c", "($a = ((! $b) / $c))");
    test_expression!(not_vs_sub, "$a = !$b - $c", "($a = ((! $b) - $c))");
    test_expression!(not_vs_mod, "$a = !$b % $c", "($a = ((! $b) % $c))");
    test_expression!(not_vs_pow, "$a = !$b ** $c", "($a = (! ($b ** $c)))");
    test_expression!(not_vs_shift_left, "$a = !$b << $c", "($a = ((! $b) << $c))");
    test_expression!(not_vs_shift_right, "$a = !$b >> $c", "($a = ((! $b) >> $c))");
    test_expression!(not_vs_bitwise_and, "$a = !$b & $c", "($a = ((! $b) & $c))");
    test_expression!(not_vs_bitwise_or, "$a = !$b | $c", "($a = ((! $b) | $c))");
    test_expression!(not_vs_bitwise_xor, "$a = !$b ^ $c", "($a = ((! $b) ^ $c))");
    test_expression!(not_vs_less_than, "$a = !$b < $c", "($a = ((! $b) < $c))");
    test_expression!(not_vs_less_than_equal, "$a = !$b <= $c", "($a = ((! $b) <= $c))");
    test_expression!(not_vs_greater_than, "$a = !$b > $c", "($a = ((! $b) > $c))");
    test_expression!(not_vs_greater_than_equal, "$a = !$b >= $c", "($a = ((! $b) >= $c))");
    test_expression!(not_vs_equal, "$a = !$b == $c", "($a = ((! $b) == $c))");
    test_expression!(not_vs_identical, "$a = !$b === $c", "($a = ((! $b) === $c))");
    test_expression!(not_vs_not_equal, "$a = !$b != $c", "($a = ((! $b) != $c))");
    test_expression!(not_vs_not_identical, "$a = !$b !== $c", "($a = ((! $b) !== $c))");
    test_expression!(not_vs_spaceship, "$a = !$b <=> $c", "($a = ((! $b) <=> $c))");
    test_expression!(not_vs_coalesce, "$a = !$b ?? $c", "($a = ((! $b) ?? $c))");
    test_expression!(not_vs_logical_and_word, "$a = !$b and $c", "(($a = (! $b)) and $c)");
    test_expression!(not_vs_logical_or_word, "$a = !$b or $c", "(($a = (! $b)) or $c)");
    test_expression!(not_vs_logical_xor_word, "$a = !$b xor $c", "(($a = (! $b)) xor $c)");
    test_expression!(not_vs_logical_and_op, "$a = !$b && $c", "($a = ((! $b) && $c))");
    test_expression!(not_vs_logical_or_op, "$a = !$b || $c", "($a = ((! $b) || $c))");
    test_expression!(not_vs_ternary, "$a = !$b ? $c : $d", "($a = ( (! $b) ? $c : $d ))");
    test_expression!(include_equal, "$a = include $b == $c", "($a = (include ($b == $c)))");
    test_expression!(include_once_equal, "$a = include_once $b == $c", "($a = (include_once ($b == $c)))");
    test_expression!(require_equal, "$a = require $b == $c", "($a = (require ($b == $c)))");
    test_expression!(require_once_equal, "$a = require_once $b == $c", "($a = (require_once ($b == $c)))");
    test_expression!(error_control_include_equal, "$a = @include $b == $c", "($a = (@ (include ($b == $c))))");
    test_expression!(
        error_control_include_once_equal,
        "$a = @include_once $b == $c",
        "($a = (@ (include_once ($b == $c))))"
    );
    test_expression!(error_control_require_equal, "$a = @require $b == $c", "($a = (@ (require ($b == $c))))");
    test_expression!(
        error_control_require_once_equal,
        "$a = @require_once $b == $c",
        "($a = (@ (require_once ($b == $c))))"
    );
    test_expression!(paren_error_control_include_equal, "$a = (@include $b) == $c", "($a = ((@ (include $b)) == $c))");
    test_expression!(
        paren_error_control_include_once_equal,
        "$a = (@include_once $b) == $c",
        "($a = ((@ (include_once $b)) == $c))"
    );
    test_expression!(paren_error_control_require_equal, "$a = (@require $b) == $c", "($a = ((@ (require $b)) == $c))");
    test_expression!(
        paren_error_control_require_once_equal,
        "$a = (@require_once $b) == $c",
        "($a = ((@ (require_once $b)) == $c))"
    );
    test_expression!(paren_new_equal, "$a = (new C) == $b", "($a = ((new C) == $b))");
    test_expression!(no_paren_new_equal, "$a = new C == $b", "($a = ((new C) == $b))");
    test_expression!(paren_error_control_new_equal, "$a = @(new C) == $b", "($a = ((@ (new C)) == $b))");
    test_expression!(no_paren_error_control_new_equal, "$a = @new C == $b", "($a = ((@ (new C)) == $b))");
    test_expression!(
        complex_arithmetic_and_logic,
        "$a = ++$b * -$c + $d / $e ** $f && $g || $h",
        "($a = (((((++ $b) * (- $c)) + ($d / ($e ** $f))) && $g) || $h))"
    );
    test_expression!(
        complex_ternary_and_coalesce,
        "$a = $b ?? $c ? $d + $e : $f - $g",
        "($a = ( ($b ?? $c) ? ($d + $e) : ($f - $g) ))"
    );
    test_expression!(complex_assignments_and_pow, "$a = $b += $c ** $d ** $e", "($a = ($b += ($c ** ($d ** $e))))");
    test_expression!(
        complex_error_control_and_instanceof,
        "$a = @$b instanceof C + $d",
        "($a = (((@ $b) instanceof C) + $d))"
    );
    test_expression!(
        complex_logical_words_and_ops,
        "$a = $b and $c || $d xor $e && $f",
        "((($a = $b) and ($c || $d)) xor ($e && $f))"
    );
    test_expression!(
        complex_shifts_and_arithmetic,
        "$a = $b << $c + $d * $e >> $f - $g",
        "($a = (($b << ($c + ($d * $e))) >> ($f - $g)))"
    );
    test_expression!(
        complex_unary_and_binary_mix,
        "$a = !$b + ~$c * --$d / @$e",
        "($a = ((! $b) + (((~ $c) * (-- $d)) / (@ $e))))"
    );
    test_expression!(
        complex_nested_ternary,
        "$a = $b ? $c ? $d : $e : $f ? $g : $h",
        "($a = ( ( $b ? ( $c ? $d : $e ) : $f ) ? $g : $h ))"
    );
    test_expression!(
        complex_coalesce_and_ternary,
        "$a = $b ?? $c ? $d : $e ?? $f",
        "($a = ( ($b ?? $c) ? $d : ($e ?? $f) ))"
    );
    test_expression!(
        complex_arithmetic_and_spaceship,
        "$a = $b + $c * $d <=> $e / $f - $g",
        "($a = (($b + ($c * $d)) <=> (($e / $f) - $g)))"
    );
    test_expression!(
        complex_all_logical,
        "$a = $b && $c || $d and $e xor $f or $g",
        "(((($a = (($b && $c) || $d)) and $e) xor $f) or $g)"
    );
    test_expression!(complex_pre_inc_and_pow, "$a = ++$b ** $c * $d", "($a = (((++ $b) ** $c) * $d))");
    test_expression!(complex_by_ref_and_coalesce, "$a = &$b ?? $c + $d", "(($a = (& $b)) ?? ($c + $d))");
    test_expression!(complex_minus_pow_mul_add, "$a = -$b ** $c * $d + $e", "($a = (((- ($b ** $c)) * $d) + $e))");
    test_expression!(
        complex_div_mul_mod_add_sub,
        "$a = $b / $c * $d % $e + $f - $g",
        "($a = ((((($b / $c) * $d) % $e) + $f) - $g))"
    );
    test_expression!(
        complex_ternary_with_assignments,
        "$a = $b ? $c = $d : $e = $f",
        "($a = ( $b ? ($c = $d) : ($e = $f) ))"
    );
    test_expression!(
        complex_instanceof_and_logical,
        "$a = $b instanceof C && $d instanceof E",
        "($a = (($b instanceof C) && ($d instanceof E)))"
    );
    test_expression!(complex_not_and_instanceof, "$a = !$b instanceof C", "($a = (! ($b instanceof C)))");
    test_expression!(complex_clone_and_arrow, "$a = clone $b * $c", "($a = ((clone $b) * $c))");
    test_expression!(complex_error_control_on_ternary, "$a = @$b ? $c : $d - $e", "($a = ( (@ $b) ? $c : ($d - $e) ))");
    test_expression!(
        complex_long_arithmetic_chain,
        "$a = $b + $c - $d * $e / $f % $g ** $h",
        "($a = (($b + $c) - ((($d * $e) / $f) % ($g ** $h))))"
    );
    test_expression!(
        complex_right_assoc_chain,
        "$a = $b ?? $c ?? $d ? $e : $f",
        "($a = ( ($b ?? ($c ?? $d)) ? $e : $f ))"
    );
    test_expression!(complex_left_assoc_chain, "$a = $b - $c + $d - $e", "($a = ((($b - $c) + $d) - $e))");
    test_expression!(
        complex_mixed_assoc_chain,
        "$a = $b ** $c + $d - $e ** $f",
        "($a = ((($b ** $c) + $d) - ($e ** $f)))"
    );
    test_expression!(complex_unary_on_right_of_binary, "$a = $b * ++$c - ~$d", "($a = (($b * (++ $c)) - (~ $d)))");
    test_expression!(
        complex_low_precedence_words_interleaved,
        "$a = $b == $c and $d != $e or $f > $g xor $h < $i",
        "((($a = ($b == $c)) and ($d != $e)) or (($f > $g) xor ($h < $i)))"
    );
    test_expression!(
        complex_bitwise_interleaved_with_arithmetic,
        "$a = $b + $c & $d * $e | $f ^ $g - $h",
        "($a = ((($b + $c) & ($d * $e)) | ($f ^ ($g - $h))))"
    );
    test_expression!(complex_ternary_in_coalesce, "$a = $b ?? $c ? $d : $e", "($a = ( ($b ?? $c) ? $d : $e ))");
    test_expression!(complex_coalesce_in_coalesce, "$a = $b ?? $c ?? $d", "($a = ($b ?? ($c ?? $d)))");
    test_expression!(complex_assignment_in_condition, "$a = ($b = $c) ? $d : $e", "($a = ( ($b = $c) ? $d : $e ))");
    test_expression!(complex_multiple_unary, "$a = !-++$b", "($a = (! (- (++ $b))))");
    test_expression!(complex_identical_vs_coalesce, "$a = $b === $c ?? $d", "($a = (($b === $c) ?? $d))");
    test_expression!(complex_coalesce_vs_identical, "$a = $b ?? $c === $d", "($a = ($b ?? ($c === $d)))");
    test_expression!(complex_pow_is_right_associative, "$a = $b ** $c ** $d", "($a = ($b ** ($c ** $d)))");
    test_expression!(complex_concat_is_left_associative, "$a = $b . $c . $d", "($a = (($b . $c) . $d))");
    test_expression!(complex_error_control_and_pre_inc, "$a = @++$b ** $c", "($a = (@ ((++ $b) ** $c)))");
    test_expression!(
        complex_long_chain_with_parens,
        "$a = ($b + $c) * ($d - $e) / (($f % $g) ** $h)",
        "($a = ((($b + $c) * ($d - $e)) / (($f % $g) ** $h)))"
    );
    test_expression!(
        complex_shifts_and_bitwise,
        "$a = $b << $c & $d >> $e | $f",
        "($a = ((($b << $c) & ($d >> $e)) | $f))"
    );
    test_expression!(
        complex_double_ternary_and_coalesce,
        "$a = $b ? $c : $d ?? $e ? $f : $g",
        "($a = ( ( $b ? $c : ($d ?? $e) ) ? $f : $g ))"
    );
    test_expression!(
        complex_ternary_condition_with_logic,
        "$a = $b > $c && $d < $e ? $f : $g",
        "($a = ( (($b > $c) && ($d < $e)) ? $f : $g ))"
    );
    test_expression!(complex_instanceof_with_new, "$a = new C instanceof D", "($a = ((new C) instanceof D))");
    test_expression!(complex_yield_precedence, "$a = $b + yield $c * $d", "($a = ($b + (yield ($c * $d))))");
    test_expression!(complex_yield_from_precedence, "$a = $b and yield from $c", "(($a = $b) and (yield from $c))");
    test_expression!(complex_print_precedence, "$a = $b && print $c", "($a = ($b && (print $c)))");
    test_expression!(
        complex_very_long_chain_of_doom,
        "$a = $b + $c * $d > $e && $f & $g | $h ^ $i or $j = $k ?? $l",
        "(($a = ((($b + ($c * $d)) > $e) && (($f & $g) | ($h ^ $i)))) or ($j = ($k ?? $l)))"
    );
    test_expression!(complex_negation_and_bitwise_not, "$a = !~$b | $c", "($a = ((! (~ $b)) | $c))");
    test_expression!(special_throw_left_vs_and, "throw new E and $c", "(throw ((new E) and $c))");
    test_expression!(special_throw_right_vs_and, "$c and throw new E", "($c and (throw (new E)))");
    test_expression!(special_throw_left_vs_xor, "throw new E xor $c", "(throw ((new E) xor $c))");
    test_expression!(special_throw_right_vs_xor, "$c xor throw new E", "($c xor (throw (new E)))");
    test_expression!(special_throw_left_vs_or, "throw new E or $c", "(throw ((new E) or $c))");
    test_expression!(special_throw_right_vs_or, "$c or throw new E", "($c or (throw (new E)))");

    test_expression!(assignment_associativity_simple, "$a ?? $b %= $c", "($a ?? ($b %= $c))");
    test_expression!(
        assignment_associativity_complex,
        "$a >> $b ?? $c %= $d <=> $e",
        "(($a >> $b) ?? ($c %= ($d <=> $e)))"
    );
    test_expression!(assignment_associativity_with_unary, "$a ** --$b *= $c", "($a ** ((-- $b) *= $c))");
    test_expression!(assignment_associativity_pow, "$a ?? $b **= $c", "($a ?? ($b **= $c))");
    test_expression!(concat_lower_than_shift, "$a . $b << $c", "($a . ($b << $c))");
    test_expression!(shift_higher_than_concat, "$a << $b . $c", "(($a << $b) . $c)");
    test_expression!(concat_and_shift_mixed, "$a . $b << $c . $d", "(($a . ($b << $c)) . $d)");
    test_expression!(expr_1, "$e = $a ? $b : $c;", "($e = ( $a ? $b : $c ))");
    test_expression!(expr_2, "$e = $a ? $b : $c ? $d : $b;", "($e = ( ( $a ? $b : $c ) ? $d : $b ))");
    test_expression!(expr_3, "$f = $a ? $b : ($c ? $d : $b);", "($f = ( $a ? $b : ( $c ? $d : $b ) ))");
    test_expression!(expr_4, "$g = $a ? ($b ? $c : $d) : $b;", "($g = ( $a ? ( $b ? $c : $d ) : $b ))");
    test_expression!(expr_5, "$h = ($a ? $b : $c) ? $d : $b;", "($h = ( ( $a ? $b : $c ) ? $d : $b ))");
    test_expression!(
        expr_6,
        "$i = ($a ? $b : $c) ? ($d ? $a : $b) : $c;",
        "($i = ( ( $a ? $b : $c ) ? ( $d ? $a : $b ) : $c ))"
    );
    test_expression!(
        expr_7,
        "$j = $a ? ($b ? $c : $d) : ($c ? $d : $a);",
        "($j = ( $a ? ( $b ? $c : $d ) : ( $c ? $d : $a ) ))"
    );
    test_expression!(
        expr_8,
        "$k = ($a ? $b : $c) ? $d : ($c ? $a : $b);",
        "($k = ( ( $a ? $b : $c ) ? $d : ( $c ? $a : $b ) ))"
    );
    test_expression!(expr_9, "$l = $a ?: $b ?: $c ?: $d;", "($l = ((($a ?: $b) ?: $c) ?: $d))");
    test_expression!(expr_10, "$m = $a ?: ($b ?: ($c ?: $d));", "($m = ($a ?: ($b ?: ($c ?: $d))))");
    test_expression!(expr_11, "$n = ($a ?: $b) ?: ($c ?: $d);", "($n = (($a ?: $b) ?: ($c ?: $d)))");
    test_expression!(expr_12, "$o = ($a ?: $b) ?: $c ?: $d;", "($o = ((($a ?: $b) ?: $c) ?: $d))");
    test_expression!(expr_13, "$p = $a ?: ($b ?: $c) ?: $d;", "($p = (($a ?: ($b ?: $c)) ?: $d))");
    test_expression!(expr_14, "$q = $a ?: $b ?: ($c ?: $d);", "($q = (($a ?: $b) ?: ($c ?: $d)))");
    test_expression!(expr_15, "$r = $a ?: ($b ?: $c ?: $d);", "($r = ($a ?: (($b ?: $c) ?: $d)))");
    test_expression!(expr_16, "$s = ($a ?: $b ?: $c) ?: $d;", "($s = ((($a ?: $b) ?: $c) ?: $d))");
    test_expression!(expr_17, "$t = $a ? $b : $c ?: $d;", "($t = (( $a ? $b : $c ) ?: $d))");
    test_expression!(rand_0, "$a || $b & $c <=> ++$d - $e or $f", "(($a || ($b & ($c <=> ((++ $d) - $e)))) or $f)");
    test_expression!(rand_1, "$a << $b ?? --$c <=> $d && ++$e", "(($a << $b) ?? (((-- $c) <=> $d) && (++ $e)))");
    test_expression!(rand_2, "$a & $b && -$c ^ $d or $e > $f", "((($a & $b) && ((- $c) ^ $d)) or ($e > $f))");
    test_expression!(rand_4, "$a and $b ** $c and $d and $e ** $f", "((($a and ($b ** $c)) and $d) and ($e ** $f))");
    test_expression!(
        rand_5,
        "$a = $b != ++$c and --$d || $e || $f || ++$g",
        "(($a = ($b != (++ $c))) and ((((-- $d) || $e) || $f) || (++ $g)))"
    );
    test_expression!(rand_7, "$a & $b - $c && $d !== ++$e / $f", "(($a & ($b - $c)) && ($d !== ((++ $e) / $f)))");
    test_expression!(rand_8, "$a or $b >>= $c >> $d << $e | $f", "($a or ($b >>= ((($c >> $d) << $e) | $f)))");
    test_expression!(rand_9, "$a % +$b && +$c - $d << $e", "(($a % (+ $b)) && (((+ $c) - $d) << $e))");
    test_expression!(rand_10, "$a ?? $b == $c || $d or $e", "(($a ?? (($b == $c) || $d)) or $e)");
    test_expression!(rand_11, "$a / $b + $c ** $d && --$e", "((($a / $b) + ($c ** $d)) && (-- $e))");
    test_expression!(
        rand_12,
        "$a & -$b = $c > $d ^ $e xor $f && ++$g ?? $h",
        "(($a & (- ($b = (($c > $d) ^ $e)))) xor (($f && (++ $g)) ?? $h))"
    );
    test_expression!(rand_13, "$a xor $b | $c <= $d >> $e", "($a xor ($b | ($c <= ($d >> $e))))");
    test_expression!(
        rand_14,
        "$a !== $b & $c % $d + $e <= $f | $g xor $h",
        "(((($a !== $b) & ((($c % $d) + $e) <= $f)) | $g) xor $h)"
    );
    test_expression!(
        rand_15,
        "$a ?? +$b << $c or $d ^ ~$e | $f ** $g",
        "(($a ?? ((+ $b) << $c)) or (($d ^ (~ $e)) | ($f ** $g)))"
    );
    test_expression!(rand_16, "$a && $b ?? $c | $d || $e and $f", "((($a && $b) ?? (($c | $d) || $e)) and $f)");
    test_expression!(rand_17, "$a && $b < $c ?? $d and $e", "((($a && ($b < $c)) ?? $d) and $e)");
    test_expression!(
        rand_18,
        "$a and $b / $c xor $d && $e ** $f & $g & --$h & $i",
        "(($a and ($b / $c)) xor ($d && (((($e ** $f) & $g) & (-- $h)) & $i)))"
    );
    test_expression!(rand_19, "$a or $b >> $c << $d << $e", "($a or ((($b >> $c) << $d) << $e))");
    test_expression!(
        rand_20,
        "$a & $b and $c && $d and $e >> $f && $g << $h ?? --$i",
        "((($a & $b) and ($c && $d)) and ((($e >> $f) && ($g << $h)) ?? (-- $i)))"
    );
    test_expression!(rand_21, "$a <= $b & $c && ++$d . $e", "((($a <= $b) & $c) && ((++ $d) . $e))");
    test_expression!(
        rand_22,
        "$a <=> $b ^ @$c and $d xor $e << $f",
        "(((($a <=> $b) ^ (@ $c)) and $d) xor ($e << $f))"
    );
    test_expression!(
        rand_23,
        "$a xor $b and $c xor $d xor $e ?? $f <= $g && ~$h",
        "((($a xor ($b and $c)) xor $d) xor ($e ?? (($f <= $g) && (~ $h))))"
    );
    test_expression!(
        rand_24,
        "$a << $b != $c ** $d ** $e && $f === $g",
        "((($a << $b) != ($c ** ($d ** $e))) && ($f === $g))"
    );
    test_expression!(
        rand_25,
        "$a - $b ?? $c * $d && +$e <= ~$f ** --$g != $h",
        "(($a - $b) ?? (($c * $d) && (((+ $e) <= (~ ($f ** (-- $g)))) != $h)))"
    );
    test_expression!(
        rand_26,
        "$a - $b << $c && --$d ?? $e & $f *= $g .= --$h",
        "(((($a - $b) << $c) && (-- $d)) ?? ($e & ($f *= ($g .= (-- $h)))))"
    );
    test_expression!(rand_27, "$a ?? $b << $c & $d * $e * --$f", "($a ?? (($b << $c) & (($d * $e) * (-- $f))))");
    test_expression!(rand_29, "$a <=> $b & $c > $d <=> $e or -$f", "((($a <=> $b) & (($c > $d) <=> $e)) or (- $f))");
    test_expression!(
        rand_30,
        "$a xor $b xor $c !== @$d || $e &= $f xor $g ^ $h ?? $i",
        "((($a xor $b) xor (($c !== (@ $d)) || ($e &= $f))) xor (($g ^ $h) ?? $i))"
    );
    test_expression!(
        rand_31,
        "$a . $b or $c <= $d | @$e ** $f === +$g ** $h",
        "(($a . $b) or (($c <= $d) | ((@ ($e ** $f)) === (+ ($g ** $h)))))"
    );
    test_expression!(
        rand_32,
        "$a <=> $b and $c || ++$d . $e && ++$f",
        "(($a <=> $b) and ($c || (((++ $d) . $e) && (++ $f))))"
    );
    test_expression!(rand_33, "$a | $b ?? $c || $d or $e", "((($a | $b) ?? ($c || $d)) or $e)");
    test_expression!(rand_34, "$a + $b <<= $c | $d xor $e and $f", "(($a + ($b <<= ($c | $d))) xor ($e and $f))");
    test_expression!(rand_36, "$a > $b xor $c ^ $d >> $e <=> $f", "(($a > $b) xor ($c ^ (($d >> $e) <=> $f)))");
    test_expression!(rand_37, "$a & $b || $c ** $d **= $e xor $f", "((($a & $b) || ($c ** ($d **= $e))) xor $f)");
    test_expression!(rand_38, "$a and +$b || $c | $d xor $e || $f", "(($a and ((+ $b) || ($c | $d))) xor ($e || $f))");
    test_expression!(rand_39, "$a | $b ?? $c <= --$d xor $e", "((($a | $b) ?? ($c <= (-- $d))) xor $e)");
    test_expression!(rand_40, "$a <=> @$b < $c xor -$d | $e | $f", "(($a <=> ((@ $b) < $c)) xor (((- $d) | $e) | $f))");
    test_expression!(rand_41, "$a & $b .= $c and $d <=> $e", "(($a & ($b .= $c)) and ($d <=> $e))");
    test_expression!(
        rand_42,
        "$a <=> $b * -$c ^ $d or $e ** ++$f || $g ** $h",
        "((($a <=> ($b * (- $c))) ^ $d) or (($e ** (++ $f)) || ($g ** $h)))"
    );
    test_expression!(
        rand_44,
        "$a or $b ?? --$c ** $d &= $e | $f ?? $g || $h <=> $i",
        "($a or ($b ?? ((-- $c) ** ($d &= (($e | $f) ?? ($g || ($h <=> $i)))))))"
    );
    test_expression!(
        rand_46,
        "$a <= $b <<= --$c >> $d ^ --$e <=> $f & $g != $h & $i",
        "($a <= ($b <<= (((-- $c) >> $d) ^ ((((-- $e) <=> $f) & ($g != $h)) & $i))))"
    );
    test_expression!(rand_47, "$a % --$b * $c and $d and $e && $f", "(((($a % (-- $b)) * $c) and $d) and ($e && $f))");
    test_expression!(rand_48, "$a and -$b && $c <=> $d + $e", "($a and ((- $b) && ($c <=> ($d + $e))))");
    test_expression!(
        rand_49,
        "$a . $b && $c && $d | $e <=> $f and ++$g && $h ** --$i",
        "(((($a . $b) && $c) && ($d | ($e <=> $f))) and ((++ $g) && ($h ** (-- $i))))"
    );
    test_expression!(
        rand_50,
        "$a >> $b and $c and -$d << $e ** $f || $g ?? $h >= $i",
        "((($a >> $b) and $c) and ((((- $d) << ($e ** $f)) || $g) ?? ($h >= $i)))"
    );
    test_expression!(
        rand_51,
        "$a or $b or $c && $d * $e ?? $f || !$g or ~$h ^ $i",
        "((($a or $b) or (($c && ($d * $e)) ?? ($f || (! $g)))) or ((~ $h) ^ $i))"
    );
    test_expression!(
        rand_52,
        "$a >> $b && $c <= $d <=> $e or $f -= $g",
        "((($a >> $b) && (($c <= $d) <=> $e)) or ($f -= $g))"
    );
    test_expression!(
        rand_54,
        "$a ?? $b | $c && $d ^ $e || $f * $g /= $h * $i",
        "($a ?? ((($b | $c) && ($d ^ $e)) || ($f * ($g /= ($h * $i)))))"
    );
    test_expression!(
        rand_55,
        "$a << $b <= $c | $d or $e <<= $f < !$g",
        "(((($a << $b) <= $c) | $d) or ($e <<= ($f < (! $g))))"
    );
    test_expression!(rand_56, "$a ** $b + $c === ++$d | $e & $f", "(((($a ** $b) + $c) === (++ $d)) | ($e & $f))");
    test_expression!(
        rand_57,
        "$a <=> $b . $c xor $d ??= $e != $f || $g ** $h",
        "(($a <=> ($b . $c)) xor ($d ??= (($e != $f) || ($g ** $h))))"
    );
    test_expression!(
        rand_58,
        "$a ** $b | $c && $d ?? --$e xor ++$f >> $g",
        "((((($a ** $b) | $c) && $d) ?? (-- $e)) xor ((++ $f) >> $g))"
    );
    test_expression!(rand_59, "$a != $b / $c >> $d ** $e ** $f", "($a != (($b / $c) >> ($d ** ($e ** $f))))");
    test_expression!(rand_60, "$a <= $b ??= $c * $d ?? $e", "($a <= ($b ??= (($c * $d) ?? $e)))");
    test_expression!(rand_61, "$a or $b ^ $c or $d <<= $e", "(($a or ($b ^ $c)) or ($d <<= $e))");
    test_expression!(
        rand_62,
        "$a << $b % $c << $d | $e / $f != $g || ++$h",
        "(((($a << ($b % $c)) << $d) | (($e / $f) != $g)) || (++ $h))"
    );
    test_expression!(rand_63, "$a & $b ** $c xor $d === --$e", "(($a & ($b ** $c)) xor ($d === (-- $e)))");
    test_expression!(
        rand_64,
        "$a >> ~$b or --$c && $d ** $e - $f >> $g",
        "(($a >> (~ $b)) or ((-- $c) && ((($d ** $e) - $f) >> $g)))"
    );
    test_expression!(
        rand_65,
        "$a & $b ^= ++$c && $d or $e or ++$f - $g % !$h ** $i",
        "((($a & ($b ^= ((++ $c) && $d))) or $e) or ((++ $f) - ($g % (! ($h ** $i)))))"
    );
    test_expression!(rand_66, "$a && $b & $c & $d and $e xor ++$f", "((($a && (($b & $c) & $d)) and $e) xor (++ $f))");
    test_expression!(
        rand_68,
        "$a >> -$b & $c xor $d ?? $e <=> $f xor @$g . $h << !$i",
        "(((($a >> (- $b)) & $c) xor ($d ?? ($e <=> $f))) xor ((@ $g) . ($h << (! $i))))"
    );
    test_expression!(
        rand_69,
        "$a >> --$b ?? $c ^ $d && $e ?? $f %= $g >> ++$h <=> $i",
        "(($a >> (-- $b)) ?? ((($c ^ $d) && $e) ?? ($f %= (($g >> (++ $h)) <=> $i))))"
    );
    test_expression!(
        rand_70,
        "$a || $b || ++$c & $d + $e && $f != $g & ~$h",
        "(($a || $b) || (((++ $c) & ($d + $e)) && (($f != $g) & (~ $h))))"
    );
    test_expression!(
        rand_71,
        "$a or $b % $c ?? $d and ++$e or !$f && $g < $h + $i",
        "(($a or ((($b % $c) ?? $d) and (++ $e))) or ((! $f) && ($g < ($h + $i))))"
    );
    test_expression!(
        rand_72,
        "$a && $b && $c ^ --$d | ++$e << $f . -$g",
        "(($a && $b) && (($c ^ (-- $d)) | (((++ $e) << $f) . (- $g))))"
    );
    test_expression!(
        rand_74,
        "$a >> $b ^ $c + $d .= $e | $f = $g > $h <<= $i",
        "(($a >> $b) ^ ($c + ($d .= ($e | ($f = ($g > ($h <<= $i)))))))"
    );
    test_expression!(
        rand_75,
        "$a / $b & $c ?? $d | $e << $f %= $g & $h <=> $i",
        "((($a / $b) & $c) ?? ($d | ($e << ($f %= ($g & ($h <=> $i))))))"
    );
    test_expression!(rand_76, "$a xor $b = $c or $d || @$e", "(($a xor ($b = $c)) or ($d || (@ $e)))");
    test_expression!(
        rand_78,
        "$a - $b xor $c or $d = $e xor $f + $g xor $h || $i",
        "((($a - $b) xor $c) or ((($d = $e) xor ($f + $g)) xor ($h || $i)))"
    );
    test_expression!(rand_79, "$a xor $b xor $c % $d || $e < $f", "(($a xor $b) xor (($c % $d) || ($e < $f)))");
    test_expression!(rand_80, "$a << $b xor $c && $d ?? $e xor $f", "((($a << $b) xor (($c && $d) ?? $e)) xor $f)");
    test_expression!(
        rand_82,
        "$a || $b <=> -$c ** $d >> $e ** $f or $g and $h",
        "(($a || ($b <=> ((- ($c ** $d)) >> ($e ** $f)))) or ($g and $h))"
    );
    test_expression!(
        rand_83,
        "$a ^ $b and $c /= $d & $e - $f and !$g >> $h <=> $i",
        "((($a ^ $b) and ($c /= ($d & ($e - $f)))) and (((! $g) >> $h) <=> $i))"
    );
    test_expression!(rand_84, "$a xor $b || $c - $d and $e", "($a xor (($b || ($c - $d)) and $e))");
    test_expression!(
        rand_85,
        "$a >> $b && $c & $d != $e ** $f xor $g",
        "((($a >> $b) && ($c & ($d != ($e ** $f)))) xor $g)"
    );
    test_expression!(rand_86, "$a .= $b = $c || ~$d < $e", "($a .= ($b = ($c || ((~ $d) < $e))))");
    test_expression!(
        rand_87,
        "$a *= $b & -$c | ++$d ?? --$e ** $f",
        "($a *= ((($b & (- $c)) | (++ $d)) ?? ((-- $e) ** $f)))"
    );
    test_expression!(
        rand_88,
        "$a + $b |= $c & $d = $e ^ +$f . $g",
        "($a + ($b |= ($c & ($d = ($e ^ ((+ $f) . $g))))))"
    );
    test_expression!(rand_89, "$a > $b - $c ^ $d or --$e", "((($a > ($b - $c)) ^ $d) or (-- $e))");
    test_expression!(rand_90, "$a >= $b or $c ??= ++$d ^ $e ?? $f", "(($a >= $b) or ($c ??= (((++ $d) ^ $e) ?? $f)))");
    test_expression!(
        rand_91,
        "$a xor $b >= $c and !$d != $e | $f || $g ** +$h < $i",
        "($a xor (($b >= $c) and ((((! $d) != $e) | $f) || (($g ** (+ $h)) < $i))))"
    );
    test_expression!(rand_92, "$a ^ $b ?? $c **= $d & $e != $f", "(($a ^ $b) ?? ($c **= ($d & ($e != $f))))");
    test_expression!(
        rand_93,
        "$a ** $b * $c >> $d xor $e ??= -$f ?? --$g === $h & $i",
        "(((($a ** $b) * $c) >> $d) xor ($e ??= ((- $f) ?? (((-- $g) === $h) & $i))))"
    );
    test_expression!(rand_95, "$a or $b or $c % $d << $e", "(($a or $b) or (($c % $d) << $e))");
    test_expression!(rand_98, "$a xor $b === $c || $d >= ++$e", "($a xor (($b === $c) || ($d >= (++ $e))))");
    test_expression!(
        rand_99,
        "$a ?? $b or $c & $d > !$e xor @$f ^ $g && $h",
        "(($a ?? $b) or (($c & ($d > (! $e))) xor (((@ $f) ^ $g) && $h)))"
    );
    test_expression!(
        rand_100,
        "$a <=> -$b %= $c && $d . +$e |= --$f",
        "($a <=> (- ($b %= ($c && ($d . (+ ($e |= (-- $f))))))))"
    );
    test_expression!(rand_101, "$a ^ @$b ^ $c << $d << $e", "(($a ^ (@ $b)) ^ (($c << $d) << $e))");
    test_expression!(rand_102, "$a and $b - $c <=> $d or $e", "(($a and (($b - $c) <=> $d)) or $e)");
    test_expression!(
        rand_103,
        "$a or $b and $c | $d -= ++$e < $f <=> $g",
        "($a or ($b and ($c | ($d -= (((++ $e) < $f) <=> $g)))))"
    );
    test_expression!(rand_104, "$a || $b &= $c & $d or $e ^ $f", "(($a || ($b &= ($c & $d))) or ($e ^ $f))");
    test_expression!(
        rand_106,
        "$a ^= $b ^ @$c && $d << $e . $f * $g",
        "($a ^= (($b ^ (@ $c)) && (($d << $e) . ($f * $g))))"
    );
    test_expression!(
        rand_107,
        "$a & $b ** $c xor $d and $e && ++$f != $g xor +$h && $i",
        "((($a & ($b ** $c)) xor ($d and ($e && ((++ $f) != $g)))) xor ((+ $h) && $i))"
    );
    test_expression!(
        rand_108,
        "$a ** $b xor $c % ++$d ** --$e ?? $f",
        "(($a ** $b) xor (($c % ((++ $d) ** (-- $e))) ?? $f))"
    );
    test_expression!(rand_109, "$a ?? $b ** $c xor $d ** @$e", "(($a ?? ($b ** $c)) xor ($d ** (@ $e)))");
    test_expression!(rand_110, "$a = $b << !$c && $d ^ $e", "($a = (($b << (! $c)) && ($d ^ $e)))");
    test_expression!(rand_112, "$a & $b ** $c <=> $d <<= $e += $f", "($a & (($b ** $c) <=> ($d <<= ($e += $f))))");
    test_expression!(
        rand_113,
        "$a or $b <=> $c + $d && $e xor $f && ~$g ?? $h << $i",
        "($a or ((($b <=> ($c + $d)) && $e) xor (($f && (~ $g)) ?? ($h << $i))))"
    );
    test_expression!(
        rand_114,
        "$a ** $b << $c / $d or $e and $f ^ $g",
        "((($a ** $b) << ($c / $d)) or ($e and ($f ^ $g)))"
    );
    test_expression!(
        rand_115,
        "$a / $b | $c or $d >> $e && $f & $g << $h + !$i",
        "((($a / $b) | $c) or (($d >> $e) && ($f & ($g << ($h + (! $i))))))"
    );
    test_expression!(
        rand_116,
        "$a **= $b or $c * +$d . --$e and $f",
        "(($a **= $b) or ((($c * (+ $d)) . (-- $e)) and $f))"
    );
    test_expression!(
        rand_117,
        "$a !== $b << $c && $d & $e <=> $f xor $g",
        "((($a !== ($b << $c)) && ($d & ($e <=> $f))) xor $g)"
    );
    test_expression!(rand_118, "$a || $b . $c | $d ?? $e ?? @$f", "(($a || (($b . $c) | $d)) ?? ($e ?? (@ $f)))");
    test_expression!(rand_119, "$a <=> $b + $c xor +$d < $e & $f", "(($a <=> ($b + $c)) xor (((+ $d) < $e) & $f))");
    test_expression!(
        rand_120,
        "$a or $b ?? --$c & $d + $e ?? $f ^ $g . $h",
        "($a or ($b ?? (((-- $c) & ($d + $e)) ?? ($f ^ ($g . $h)))))"
    );
    test_expression!(rand_121, "$a and $b * $c != $d ** $e", "($a and (($b * $c) != ($d ** $e)))");
    test_expression!(
        rand_122,
        "$a ?? $b and $c ** $d and $e || $f && $g ?? $h >> $i",
        "((($a ?? $b) and ($c ** $d)) and (($e || ($f && $g)) ?? ($h >> $i)))"
    );
    test_expression!(
        rand_123,
        "$a / $b ^ $c % $d /= --$e && $f << $g < $h",
        "(($a / $b) ^ ($c % ($d /= ((-- $e) && (($f << $g) < $h)))))"
    );
    test_expression!(
        rand_124,
        "$a ^ $b **= $c . $d << $e != $f ?? $g xor ~$h and $i",
        "(($a ^ ($b **= ((($c . ($d << $e)) != $f) ?? $g))) xor ((~ $h) and $i))"
    );
    test_expression!(rand_125, "$a ^ $b >>= $c > $d or $e > $f", "(($a ^ ($b >>= ($c > $d))) or ($e > $f))");
    test_expression!(
        rand_126,
        "$a xor $b <=> $c | $d or $e or $f >> $g <= --$h ^ $i",
        "((($a xor (($b <=> $c) | $d)) or $e) or ((($f >> $g) <= (-- $h)) ^ $i))"
    );
    test_expression!(rand_127, "$a & $b ?? $c || $d ^ $e xor $f", "((($a & $b) ?? ($c || ($d ^ $e))) xor $f)");
    test_expression!(
        rand_129,
        "$a xor $b | $c ?? $d ** $e .= $f xor $g == $h xor $i",
        "((($a xor (($b | $c) ?? ($d ** ($e .= $f)))) xor ($g == $h)) xor $i)"
    );
    test_expression!(rand_130, "$a * $b = $c **= $d ^ $e * $f", "($a * ($b = ($c **= ($d ^ ($e * $f)))))");
    test_expression!(rand_131, "$a | $b . $c !== $d or ++$e == $f", "(($a | (($b . $c) !== $d)) or ((++ $e) == $f))");
    test_expression!(
        rand_132,
        "$a || $b ** $c >> $d ^ $e != -$f && $g ** $h + $i",
        "($a || (((($b ** $c) >> $d) ^ ($e != (- $f))) && (($g ** $h) + $i)))"
    );
    test_expression!(rand_133, "$a or $b - $c >> $d & $e", "($a or ((($b - $c) >> $d) & $e))");
    test_expression!(rand_134, "$a && $b ** $c & $d ^ ++$e", "($a && ((($b ** $c) & $d) ^ (++ $e)))");
    test_expression!(
        rand_136,
        "$a <=> $b << $c | $d <=> $e ** $f >= $g | $h ?? $i",
        "(((($a <=> ($b << $c)) | ($d <=> (($e ** $f) >= $g))) | $h) ?? $i)"
    );
    test_expression!(rand_138, "$a >> $b * $c and -$d ^ $e xor $f", "((($a >> ($b * $c)) and ((- $d) ^ $e)) xor $f)");
    test_expression!(rand_139, "$a ** $b / $c && $d and $e", "(((($a ** $b) / $c) && $d) and $e)");
    test_expression!(
        rand_140,
        "$a and $b and $c && $d ?? $e | !$f + $g",
        "(($a and $b) and (($c && $d) ?? ($e | ((! $f) + $g))))"
    );
    test_expression!(rand_141, "$a % $b ** $c % $d >> $e", "((($a % ($b ** $c)) % $d) >> $e)");
    test_expression!(
        rand_142,
        "$a ?? ~$b && $c | $d <=> $e ** $f and $g or ~$h | $i",
        "((($a ?? ((~ $b) && ($c | ($d <=> ($e ** $f))))) and $g) or ((~ $h) | $i))"
    );
    test_expression!(
        rand_143,
        "$a xor $b ** $c ** $d | $e ^= $f * ~$g && !$h xor +$i",
        "(($a xor (($b ** ($c ** $d)) | ($e ^= (($f * (~ $g)) && (! $h))))) xor (+ $i))"
    );
    test_expression!(rand_144, "$a && $b %= $c ^ $d ** $e", "($a && ($b %= ($c ^ ($d ** $e))))");
    test_expression!(rand_145, "$a ** $b ?? $c == $d ** $e", "(($a ** $b) ?? ($c == ($d ** $e)))");
    test_expression!(
        rand_146,
        "$a /= --$b ^ $c / $d ?? ++$e >= $f || $g && --$h == $i",
        "($a /= (((-- $b) ^ ($c / $d)) ?? (((++ $e) >= $f) || ($g && ((-- $h) == $i)))))"
    );
    test_expression!(rand_147, "$a && $b && $c / $d && $e", "((($a && $b) && ($c / $d)) && $e)");
    test_expression!(rand_148, "$a ** $b ^ $c | +$d -= $e & $f", "((($a ** $b) ^ $c) | (+ ($d -= ($e & $f))))");
    test_expression!(
        rand_149,
        "$a /= ++$b ^ $c ^ $d >> $e %= $f && $g",
        "($a /= (((++ $b) ^ $c) ^ ($d >> ($e %= ($f && $g)))))"
    );
    test_expression!(
        rand_150,
        "$a and $b .= $c . --$d <=> @$e /= $f << ++$g ?? $h",
        "($a and ($b .= (($c . (-- $d)) <=> (@ ($e /= (($f << (++ $g)) ?? $h))))))"
    );
    test_expression!(rand_151, "$a <=> $b + --$c and $d | $e or $f", "((($a <=> ($b + (-- $c))) and ($d | $e)) or $f)");
    test_expression!(rand_152, "$a -= $b and $c or $d != $e", "((($a -= $b) and $c) or ($d != $e))");
    test_expression!(
        rand_153,
        "$a xor $b = $c | ~$d or +$e or $f or -$g >> $h",
        "(((($a xor ($b = ($c | (~ $d)))) or (+ $e)) or $f) or ((- $g) >> $h))"
    );
    test_expression!(rand_154, "$a or $b << $c and $d != $e or $f", "(($a or (($b << $c) and ($d != $e))) or $f)");
    test_expression!(
        rand_155,
        "$a ?? $b & $c & ~$d >> $e ^ $f >> $g",
        "($a ?? ((($b & $c) & ((~ $d) >> $e)) ^ ($f >> $g)))"
    );
    test_expression!(
        rand_156,
        "$a xor $b & --$c << $d && $e ^ --$f ** ++$g % $h",
        "($a xor (($b & ((-- $c) << $d)) && ($e ^ (((-- $f) ** (++ $g)) % $h))))"
    );
    test_expression!(
        rand_157,
        "$a and --$b <=> $c xor $d ?? $e and $f ** $g",
        "(($a and ((-- $b) <=> $c)) xor (($d ?? $e) and ($f ** $g)))"
    );
    test_expression!(rand_158, "$a ^ --$b ?? $c || $d >> $e", "(($a ^ (-- $b)) ?? ($c || ($d >> $e)))");
    test_expression!(rand_159, "$a = $b xor $c & $d && -$e >> $f", "(($a = $b) xor (($c & $d) && ((- $e) >> $f)))");
    test_expression!(
        rand_160,
        "$a + !$b ** ~$c ^ $d % $e * $f != $g",
        "(($a + (! ($b ** (~ $c)))) ^ ((($d % $e) * $f) != $g))"
    );
    test_expression!(
        rand_161,
        "$a - $b or $c + $d >> $e + $f ** $g & $h ?? $i",
        "(($a - $b) or (((($c + $d) >> ($e + ($f ** $g))) & $h) ?? $i))"
    );
    test_expression!(
        rand_162,
        "$a % $b ** --$c <=> $d & $e < $f ?? $g ?? $h",
        "(((($a % ($b ** (-- $c))) <=> $d) & ($e < $f)) ?? ($g ?? $h))"
    );
    test_expression!(
        rand_163,
        "$a && $b < $c | ++$d or ++$e ^ $f",
        "(($a && (($b < $c) | (++ $d))) or ((++ $e) ^ $f))"
    );
    test_expression!(rand_164, "$a && $b | $c > $d . ~$e", "($a && ($b | ($c > ($d . (~ $e)))))");
    test_expression!(
        rand_165,
        "$a *= $b & $c ** $d && -$e && +$f > $g | $h",
        "($a *= ((($b & ($c ** $d)) && (- $e)) && (((+ $f) > $g) | $h)))"
    );
    test_expression!(rand_167, "$a ^ $b <=> $c && $d === $e & $f", "(($a ^ ($b <=> $c)) && (($d === $e) & $f))");
    test_expression!(rand_168, "$a && $b |= $c ^ $d && $e", "($a && ($b |= (($c ^ $d) && $e)))");
    test_expression!(
        rand_169,
        "$a ** -$b ^ $c or $d xor ~$e + $f xor $g",
        "((($a ** (- $b)) ^ $c) or (($d xor ((~ $e) + $f)) xor $g))"
    );
    test_expression!(
        rand_170,
        "$a | $b && ++$c | ++$d . $e & $f && $g + $h || $i",
        "(((($a | $b) && ((++ $c) | (((++ $d) . $e) & $f))) && ($g + $h)) || $i)"
    );
    test_expression!(rand_171, "$a ^ $b / @$c | $d += --$e", "(($a ^ ($b / (@ $c))) | ($d += (-- $e)))");
    test_expression!(
        rand_172,
        "$a or $b <=> $c / $d | $e / $f | $g",
        "($a or ((($b <=> ($c / $d)) | ($e / $f)) | $g))"
    );
    test_expression!(
        rand_174,
        "$a or $b && $c <= $d || $e | -$f ^ $g !== $h or $i",
        "(($a or (($b && ($c <= $d)) || ($e | ((- $f) ^ ($g !== $h))))) or $i)"
    );
    test_expression!(rand_175, "$a || $b >> $c ^ $d >> $e | $f", "($a || ((($b >> $c) ^ ($d >> $e)) | $f))");
    test_expression!(
        rand_176,
        "$a << ~$b & $c | $d xor $e ** $f xor $g",
        "((((($a << (~ $b)) & $c) | $d) xor ($e ** $f)) xor $g)"
    );
    test_expression!(rand_177, "$a or ++$b >= $c **= $d << $e", "($a or ((++ $b) >= ($c **= ($d << $e))))");
    test_expression!(rand_178, "$a % $b ** $c && $d . $e", "(($a % ($b ** $c)) && ($d . $e))");
    test_expression!(rand_179, "$a xor $b + $c <=> $d or $e", "(($a xor (($b + $c) <=> $d)) or $e)");
    test_expression!(
        rand_180,
        "$a | $b > $c and --$d or $e ^ $f ^ $g ?? $h",
        "((($a | ($b > $c)) and (-- $d)) or ((($e ^ $f) ^ $g) ?? $h))"
    );
    test_expression!(
        rand_181,
        "$a ^ $b && $c && $d -= --$e & $f and $g",
        "(((($a ^ $b) && $c) && ($d -= ((-- $e) & $f))) and $g)"
    );
    test_expression!(rand_182, "$a <=> $b /= +$c >= $d >> $e", "($a <=> ($b /= ((+ $c) >= ($d >> $e))))");
    test_expression!(rand_183, "$a || $b . $c * $d and $e << --$f", "(($a || ($b . ($c * $d))) and ($e << (-- $f)))");
    test_expression!(rand_184, "$a + $b or $c + $d || $e", "(($a + $b) or (($c + $d) || $e))");
    test_expression!(
        rand_186,
        "$a xor $b >> $c / $d == $e ?? --$f !== ++$g",
        "($a xor ((($b >> ($c / $d)) == $e) ?? ((-- $f) !== (++ $g))))"
    );
    test_expression!(
        rand_187,
        "$a & $b xor -$c << $d -= +$e && $f <=> $g",
        "(($a & $b) xor ((- $c) << ($d -= ((+ $e) && ($f <=> $g)))))"
    );
    test_expression!(rand_188, "$a += $b or $c << $d and $e ^ $f", "(($a += $b) or (($c << $d) and ($e ^ $f)))");
    test_expression!(
        rand_190,
        "$a | $b ^ !$c xor !$d % ++$e <=> $f ^ $g | $h ** !$i",
        "(($a | ($b ^ (! $c))) xor (((((! $d) % (++ $e)) <=> $f) ^ $g) | ($h ** (! $i))))"
    );
    test_expression!(
        rand_191,
        "$a & $b - +$c xor $d or $e ?? $f === $g | $h **= $i",
        "((($a & ($b - (+ $c))) xor $d) or ($e ?? (($f === $g) | ($h **= $i))))"
    );
    test_expression!(
        rand_192,
        "$a and $b or ++$c <=> $d - $e or $f & ++$g <=> $h",
        "((($a and $b) or ((++ $c) <=> ($d - $e))) or ($f & ((++ $g) <=> $h)))"
    );
    test_expression!(rand_193, "$a || $b == ~$c - $d | $e | $f", "($a || ((($b == ((~ $c) - $d)) | $e) | $f))");
    test_expression!(
        rand_194,
        "$a ^ $b && $c ?? $d and $e >= $f ^ $g && $h",
        "(((($a ^ $b) && $c) ?? $d) and ((($e >= $f) ^ $g) && $h))"
    );
    test_expression!(rand_195, "$a || $b != $c xor $d ** $e", "(($a || ($b != $c)) xor ($d ** $e))");
    test_expression!(
        rand_196,
        "$a xor $b || $c & $d xor $e <=> --$f ?? ~$g",
        "(($a xor ($b || ($c & $d))) xor (($e <=> (-- $f)) ?? (~ $g)))"
    );
    test_expression!(
        rand_197,
        "$a >> $b and ++$c >> $d ?? !$e . $f ^ $g <= $h && $i",
        "(($a >> $b) and (((++ $c) >> $d) ?? ((((! $e) . $f) ^ ($g <= $h)) && $i)))"
    );
    test_expression!(
        rand_198,
        "$a **= --$b and $c xor $d & $e ^ $f xor $g != $h",
        "(((($a **= (-- $b)) and $c) xor (($d & $e) ^ $f)) xor ($g != $h))"
    );
    test_expression!(rand_199, "$a or $b || $c ** $d >= $e", "($a or ($b || (($c ** $d) >= $e)))");

    smoke_test!(closing_tag_echo_tag, "<?= $a ?> <?= $b;");
}
