use fennec_ast::*;

#[inline]
pub fn find_returns_in_block<'ast>(block: &'ast Block) -> Vec<&'ast Return> {
    let mut returns = vec![];

    for statement in block.statements.iter() {
        returns.extend(find_returns_in_statement(statement));
    }

    returns
}

#[inline]
pub fn find_returns_in_statement<'ast>(statement: &'ast Statement) -> Vec<&'ast Return> {
    let mut returns = vec![];

    match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                returns.extend(find_returns_in_statement(statement));
            }
        }
        Statement::Block(block) => {
            returns.extend(find_returns_in_block(block));
        }
        Statement::Try(r#try) => {
            returns.extend(find_returns_in_block(&r#try.block));

            for catch in r#try.catch_clauses.iter() {
                returns.extend(find_returns_in_block(&catch.block));
            }

            if let Some(finally) = &r#try.finally_clause {
                returns.extend(find_returns_in_block(&finally.block));
            }
        }
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(statement) => {
                returns.extend(find_returns_in_statement(statement));
            }
            ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                for statement in foreach_colon_delimited_body.statements.iter() {
                    returns.extend(find_returns_in_statement(statement));
                }
            }
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(statement) => {
                returns.extend(find_returns_in_statement(statement));
            }
            ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                for statement in foreach_colon_delimited_body.statements.iter() {
                    returns.extend(find_returns_in_statement(statement));
                }
            }
        },
        Statement::While(r#while) => match &r#while.body {
            WhileBody::Statement(statement) => {
                returns.extend(find_returns_in_statement(statement));
            }
            WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                for statement in foreach_colon_delimited_body.statements.iter() {
                    returns.extend(find_returns_in_statement(statement));
                }
            }
        },
        Statement::DoWhile(do_while) => {
            returns.extend(find_returns_in_statement(&do_while.statement));
        }
        Statement::Switch(switch) => {
            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            for case in cases.iter() {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        for statement in switch_expression_case.statements.iter() {
                            returns.extend(find_returns_in_statement(statement));
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in switch_default_case.statements.iter() {
                            returns.extend(find_returns_in_statement(statement));
                        }
                    }
                }
            }
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(if_statement_body) => {
                returns.extend(find_returns_in_statement(&if_statement_body.statement));

                for else_if in if_statement_body.else_if_clauses.iter() {
                    returns.extend(find_returns_in_statement(&else_if.statement));
                }

                if let Some(else_clause) = &if_statement_body.else_clause {
                    returns.extend(find_returns_in_statement(&else_clause.statement));
                }
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                for statement in if_colon_delimited_body.statements.iter() {
                    returns.extend(find_returns_in_statement(statement));
                }

                for else_if in if_colon_delimited_body.else_if_clauses.iter() {
                    for statement in else_if.statements.iter() {
                        returns.extend(find_returns_in_statement(statement));
                    }
                }

                if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                    for statement in else_clause.statements.iter() {
                        returns.extend(find_returns_in_statement(statement));
                    }
                }
            }
        },
        Statement::Return(r#return) => {
            returns.push(r#return);
        }
        _ => {}
    }

    returns
}

#[inline]
pub fn block_has_yield<'ast>(block: &'ast Block) -> bool {
    for statement in block.statements.iter() {
        if statement_has_yield(statement) {
            return true;
        }
    }

    false
}

#[inline]
pub fn statement_has_yield<'ast>(statement: &'ast Statement) -> bool {
    return match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                if statement_has_yield(statement) {
                    return true;
                }
            }

            false
        }
        Statement::Block(block) => block_has_yield(block),
        Statement::Try(r#try) => {
            if r#try.catch_clauses.iter().any(|catch| block_has_yield(&catch.block)) {
                return true;
            }

            for catch in r#try.catch_clauses.iter() {
                if block_has_yield(&catch.block) {
                    return true;
                }
            }

            if let Some(finally) = &r#try.finally_clause {
                if block_has_yield(&finally.block) {
                    return true;
                }
            }

            false
        }
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(statement) => statement_has_yield(statement),
            ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_yield(statement))
            }
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(statement) => statement_has_yield(statement),
            ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_yield(statement))
            }
        },
        Statement::While(r#while) => match &r#while.body {
            WhileBody::Statement(statement) => statement_has_yield(statement),
            WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_yield(statement))
            }
        },
        Statement::DoWhile(do_while) => statement_has_yield(&do_while.statement),
        Statement::Switch(switch) => {
            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            for case in cases.iter() {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        for statement in switch_expression_case.statements.iter() {
                            if statement_has_yield(statement) {
                                return true;
                            }
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in switch_default_case.statements.iter() {
                            if statement_has_yield(statement) {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(if_statement_body) => {
                if statement_has_yield(&if_statement_body.statement) {
                    return true;
                }

                for else_if in if_statement_body.else_if_clauses.iter() {
                    if statement_has_yield(&else_if.statement) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_statement_body.else_clause {
                    if statement_has_yield(&else_clause.statement) {
                        return true;
                    }
                }

                false
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if if_colon_delimited_body.statements.iter().any(|statement| statement_has_yield(statement)) {
                    return true;
                }

                for else_if in if_colon_delimited_body.else_if_clauses.iter() {
                    if else_if.statements.iter().any(|statement| statement_has_yield(statement)) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                    if else_clause.statements.iter().any(|statement| statement_has_yield(statement)) {
                        return true;
                    }
                }

                false
            }
        },
        Statement::Expression(expression) => expression_has_yield(&expression.expression),
        _ => false,
    };
}

#[inline]
pub fn expression_has_yield<'ast>(expression: &'ast Expression) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => expression_has_yield(&parenthesized.expression),
        Expression::Referenced(referenced) => expression_has_yield(&referenced.expression),
        Expression::Suppressed(suppressed) => expression_has_yield(&suppressed.expression),
        Expression::Literal(_) => false,
        Expression::CompositeString(_) => false,
        Expression::ArithmeticOperation(arithmetic_operation) => match arithmetic_operation.as_ref() {
            ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                expression_has_yield(&arithmetic_prefix_operation.value)
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                expression_has_yield(&arithmetic_infix_operation.lhs)
                    || expression_has_yield(&arithmetic_infix_operation.rhs)
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                expression_has_yield(&arithmetic_postfix_operation.value)
            }
        },
        Expression::AssignmentOperation(assignment_operation) => {
            expression_has_yield(&assignment_operation.lhs) || expression_has_yield(&assignment_operation.rhs)
        }
        Expression::BitwiseOperation(bitwise_operation) => match bitwise_operation.as_ref() {
            BitwiseOperation::Prefix(bitwise_prefix_operation) => expression_has_yield(&bitwise_prefix_operation.value),
            BitwiseOperation::Infix(bitwise_infix_operation) => {
                expression_has_yield(&bitwise_infix_operation.lhs) || expression_has_yield(&bitwise_infix_operation.rhs)
            }
        },
        Expression::ComparisonOperation(comparison_operation) => {
            expression_has_yield(&comparison_operation.lhs) || expression_has_yield(&comparison_operation.rhs)
        }
        Expression::LogicalOperation(logical_operation) => match logical_operation.as_ref() {
            LogicalOperation::Prefix(logical_prefix_operation) => expression_has_yield(&logical_prefix_operation.value),
            LogicalOperation::Infix(logical_infix_operation) => {
                expression_has_yield(&logical_infix_operation.lhs) || expression_has_yield(&logical_infix_operation.rhs)
            }
        },
        Expression::CastOperation(cast_operation) => expression_has_yield(&cast_operation.value),
        Expression::TernaryOperation(ternary_operation) => match ternary_operation.as_ref() {
            TernaryOperation::Conditional(conditional_ternary_operation) => {
                expression_has_yield(&conditional_ternary_operation.condition)
                    || conditional_ternary_operation.then.as_ref().map(expression_has_yield).unwrap_or(false)
                    || expression_has_yield(&conditional_ternary_operation.r#else)
            }
            TernaryOperation::Elvis(elvis_ternary_operation) => {
                expression_has_yield(&elvis_ternary_operation.condition)
                    || expression_has_yield(&elvis_ternary_operation.r#else)
            }
        },
        Expression::CoalesceOperation(coalesce_operation) => {
            expression_has_yield(&coalesce_operation.lhs) || expression_has_yield(&coalesce_operation.rhs)
        }
        Expression::ConcatOperation(concat_operation) => {
            expression_has_yield(&concat_operation.lhs) || expression_has_yield(&concat_operation.rhs)
        }
        Expression::InstanceofOperation(instanceof_operation) => {
            expression_has_yield(&instanceof_operation.lhs) || expression_has_yield(&instanceof_operation.rhs)
        }
        Expression::Array(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(&key_value_array_element.key)
                    || expression_has_yield(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(&variadic_array_element.value),
            _ => false,
        }),
        Expression::LegacyArray(legacy_array) => legacy_array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(&key_value_array_element.key)
                    || expression_has_yield(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(&variadic_array_element.value),
            _ => false,
        }),
        Expression::List(list) => list.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_yield(&key_value_array_element.key)
                    || expression_has_yield(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_yield(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_yield(&variadic_array_element.value),
            _ => false,
        }),
        Expression::ArrayAccess(array_access) => {
            expression_has_yield(&array_access.array) || expression_has_yield(&array_access.index)
        }
        Expression::ArrayAppend(array_append) => expression_has_yield(&array_append.array),
        Expression::Match(r#match) => {
            expression_has_yield(&r#match.expression)
                || r#match.arms.iter().any(|arm| match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        match_expression_arm.conditions.iter().any(expression_has_yield)
                            || expression_has_yield(&match_expression_arm.expression)
                    }
                    MatchArm::Default(match_default_arm) => expression_has_yield(&match_default_arm.expression),
                })
        }
        Expression::Construct(construct) => match construct.as_ref() {
            Construct::Isset(isset_construct) => isset_construct.values.iter().any(expression_has_yield),
            Construct::Empty(empty_construct) => expression_has_yield(&empty_construct.value),
            Construct::Eval(eval_construct) => expression_has_yield(&eval_construct.value),
            Construct::Include(include_construct) => expression_has_yield(&include_construct.value),
            Construct::IncludeOnce(include_once_construct) => expression_has_yield(&include_once_construct.value),
            Construct::Require(require_construct) => expression_has_yield(&require_construct.value),
            Construct::RequireOnce(require_once_construct) => expression_has_yield(&require_once_construct.value),
            Construct::Print(print_construct) => expression_has_yield(&print_construct.value),
            Construct::Exit(exit_construct) => exit_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
                })
                .unwrap_or(false),
            Construct::Die(die_construct) => die_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
                })
                .unwrap_or(false),
        },
        Expression::Throw(throw) => expression_has_yield(&throw.exception),
        Expression::Clone(clone) => expression_has_yield(&clone.object),
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                expression_has_yield(&function_call.function)
                    || function_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::Method(method_call) => {
                expression_has_yield(&method_call.object)
                    || matches!(&method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
                    || method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                expression_has_yield(&null_safe_method_call.object)
                    || matches!(&null_safe_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
                    || null_safe_method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
            Call::StaticMethod(static_method_call) => {
                expression_has_yield(&static_method_call.class)
                    || matches!(&static_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
                    || static_method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_yield(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                    })
            }
        },
        Expression::Access(access) => match access.as_ref() {
            Access::Property(property_access) => {
                expression_has_yield(&property_access.object)
                    || matches!(&property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                expression_has_yield(&null_safe_property_access.object)
                    || matches!(&null_safe_property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
            }
            Access::StaticProperty(static_property_access) => expression_has_yield(&static_property_access.class),
            Access::ClassConstant(class_constant_access) => {
                expression_has_yield(&class_constant_access.class)
                    || matches!(&class_constant_access.constant, ClassLikeConstantSelector::Expression(selector) if expression_has_yield(&selector.expression))
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation.as_ref() {
            ClosureCreation::Function(function_closure_creation) => {
                expression_has_yield(&function_closure_creation.function)
            }
            ClosureCreation::Method(method_closure_creation) => {
                expression_has_yield(&method_closure_creation.object)
                    || matches!(&method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                expression_has_yield(&static_method_closure_creation.class)
                    || matches!(&static_method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_yield(&selector.expression))
            }
        },
        Expression::Instantiation(instantiation) => {
            expression_has_yield(&instantiation.class)
                || instantiation
                    .arguments
                    .as_ref()
                    .map(|arguments| {
                        arguments.arguments.iter().any(|argument| match argument {
                            Argument::Positional(positional_argument) => {
                                expression_has_yield(&positional_argument.value)
                            }
                            Argument::Named(named_argument) => expression_has_yield(&named_argument.value),
                        })
                    })
                    .unwrap_or(false)
        }
        Expression::Yield(_) => true,
        _ => false,
    }
}

#[inline]
pub fn block_has_throws<'ast>(block: &'ast Block) -> bool {
    for statement in block.statements.iter() {
        if statement_has_throws(statement) {
            return true;
        }
    }

    false
}

#[inline]
pub fn statement_has_throws<'ast>(statement: &'ast Statement) -> bool {
    return match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                if statement_has_throws(statement) {
                    return true;
                }
            }

            false
        }
        Statement::Block(block) => block_has_throws(block),
        Statement::Try(r#try) => {
            if r#try.catch_clauses.iter().any(|catch| block_has_throws(&catch.block)) {
                return true;
            }

            for catch in r#try.catch_clauses.iter() {
                if block_has_throws(&catch.block) {
                    return true;
                }
            }

            if let Some(finally) = &r#try.finally_clause {
                if block_has_throws(&finally.block) {
                    return true;
                }
            }

            false
        }
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(statement) => statement_has_throws(statement),
            ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_throws(statement))
            }
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(statement) => statement_has_throws(statement),
            ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_throws(statement))
            }
        },
        Statement::While(r#while) => match &r#while.body {
            WhileBody::Statement(statement) => statement_has_throws(statement),
            WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.iter().any(|statement| statement_has_throws(statement))
            }
        },
        Statement::DoWhile(do_while) => statement_has_throws(&do_while.statement),
        Statement::Switch(switch) => {
            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            for case in cases.iter() {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        for statement in switch_expression_case.statements.iter() {
                            if statement_has_throws(statement) {
                                return true;
                            }
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in switch_default_case.statements.iter() {
                            if statement_has_throws(statement) {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(if_statement_body) => {
                if statement_has_throws(&if_statement_body.statement) {
                    return true;
                }

                for else_if in if_statement_body.else_if_clauses.iter() {
                    if statement_has_throws(&else_if.statement) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_statement_body.else_clause {
                    if statement_has_throws(&else_clause.statement) {
                        return true;
                    }
                }

                false
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                if if_colon_delimited_body.statements.iter().any(|statement| statement_has_throws(statement)) {
                    return true;
                }

                for else_if in if_colon_delimited_body.else_if_clauses.iter() {
                    if else_if.statements.iter().any(|statement| statement_has_throws(statement)) {
                        return true;
                    }
                }

                if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                    if else_clause.statements.iter().any(|statement| statement_has_throws(statement)) {
                        return true;
                    }
                }

                false
            }
        },
        Statement::Expression(expression) => expression_has_throws(&expression.expression),
        _ => false,
    };
}

#[inline]
pub fn expression_has_throws<'ast>(expression: &'ast Expression) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => expression_has_throws(&parenthesized.expression),
        Expression::Referenced(referenced) => expression_has_throws(&referenced.expression),
        Expression::Suppressed(suppressed) => expression_has_throws(&suppressed.expression),
        Expression::Literal(_) => false,
        Expression::CompositeString(_) => false,
        Expression::ArithmeticOperation(arithmetic_operation) => match arithmetic_operation.as_ref() {
            ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                expression_has_throws(&arithmetic_prefix_operation.value)
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                expression_has_throws(&arithmetic_infix_operation.lhs)
                    || expression_has_throws(&arithmetic_infix_operation.rhs)
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                expression_has_throws(&arithmetic_postfix_operation.value)
            }
        },
        Expression::AssignmentOperation(assignment_operation) => {
            expression_has_throws(&assignment_operation.lhs) || expression_has_throws(&assignment_operation.rhs)
        }
        Expression::BitwiseOperation(bitwise_operation) => match bitwise_operation.as_ref() {
            BitwiseOperation::Prefix(bitwise_prefix_operation) => {
                expression_has_throws(&bitwise_prefix_operation.value)
            }
            BitwiseOperation::Infix(bitwise_infix_operation) => {
                expression_has_throws(&bitwise_infix_operation.lhs)
                    || expression_has_throws(&bitwise_infix_operation.rhs)
            }
        },
        Expression::ComparisonOperation(comparison_operation) => {
            expression_has_throws(&comparison_operation.lhs) || expression_has_throws(&comparison_operation.rhs)
        }
        Expression::LogicalOperation(logical_operation) => match logical_operation.as_ref() {
            LogicalOperation::Prefix(logical_prefix_operation) => {
                expression_has_throws(&logical_prefix_operation.value)
            }
            LogicalOperation::Infix(logical_infix_operation) => {
                expression_has_throws(&logical_infix_operation.lhs)
                    || expression_has_throws(&logical_infix_operation.rhs)
            }
        },
        Expression::CastOperation(cast_operation) => expression_has_throws(&cast_operation.value),
        Expression::TernaryOperation(ternary_operation) => match ternary_operation.as_ref() {
            TernaryOperation::Conditional(conditional_ternary_operation) => {
                expression_has_throws(&conditional_ternary_operation.condition)
                    || conditional_ternary_operation.then.as_ref().map(expression_has_throws).unwrap_or(false)
                    || expression_has_throws(&conditional_ternary_operation.r#else)
            }
            TernaryOperation::Elvis(elvis_ternary_operation) => {
                expression_has_throws(&elvis_ternary_operation.condition)
                    || expression_has_throws(&elvis_ternary_operation.r#else)
            }
        },
        Expression::CoalesceOperation(coalesce_operation) => {
            expression_has_throws(&coalesce_operation.lhs) || expression_has_throws(&coalesce_operation.rhs)
        }
        Expression::ConcatOperation(concat_operation) => {
            expression_has_throws(&concat_operation.lhs) || expression_has_throws(&concat_operation.rhs)
        }
        Expression::InstanceofOperation(instanceof_operation) => {
            expression_has_throws(&instanceof_operation.lhs) || expression_has_throws(&instanceof_operation.rhs)
        }
        Expression::Array(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_throws(&key_value_array_element.key)
                    || expression_has_throws(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_throws(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_throws(&variadic_array_element.value),
            _ => false,
        }),
        Expression::LegacyArray(legacy_array) => legacy_array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_throws(&key_value_array_element.key)
                    || expression_has_throws(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_throws(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_throws(&variadic_array_element.value),
            _ => false,
        }),
        Expression::List(list) => list.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                expression_has_throws(&key_value_array_element.key)
                    || expression_has_throws(&key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => expression_has_throws(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => expression_has_throws(&variadic_array_element.value),
            _ => false,
        }),
        Expression::ArrayAccess(array_access) => {
            expression_has_throws(&array_access.array) || expression_has_throws(&array_access.index)
        }
        Expression::ArrayAppend(array_append) => expression_has_throws(&array_append.array),
        Expression::Match(r#match) => {
            expression_has_throws(&r#match.expression)
                || r#match.arms.iter().any(|arm| match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        match_expression_arm.conditions.iter().any(expression_has_throws)
                            || expression_has_throws(&match_expression_arm.expression)
                    }
                    MatchArm::Default(match_default_arm) => expression_has_throws(&match_default_arm.expression),
                })
        }
        Expression::Construct(construct) => match construct.as_ref() {
            Construct::Isset(isset_construct) => isset_construct.values.iter().any(expression_has_throws),
            Construct::Empty(empty_construct) => expression_has_throws(&empty_construct.value),
            Construct::Eval(eval_construct) => expression_has_throws(&eval_construct.value),
            Construct::Include(include_construct) => expression_has_throws(&include_construct.value),
            Construct::IncludeOnce(include_once_construct) => expression_has_throws(&include_once_construct.value),
            Construct::Require(require_construct) => expression_has_throws(&require_construct.value),
            Construct::RequireOnce(require_once_construct) => expression_has_throws(&require_once_construct.value),
            Construct::Print(print_construct) => expression_has_throws(&print_construct.value),
            Construct::Exit(exit_construct) => exit_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
                })
                .unwrap_or(false),
            Construct::Die(die_construct) => die_construct
                .arguments
                .as_ref()
                .map(|arguments| {
                    arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
                })
                .unwrap_or(false),
        },
        Expression::Clone(clone) => expression_has_throws(&clone.object),
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                expression_has_throws(&function_call.function)
                    || function_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
            }
            Call::Method(method_call) => {
                expression_has_throws(&method_call.object)
                    || matches!(&method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
                    || method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                expression_has_throws(&null_safe_method_call.object)
                    || matches!(&null_safe_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
                    || null_safe_method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
            }
            Call::StaticMethod(static_method_call) => {
                expression_has_throws(&static_method_call.class)
                    || matches!(&static_method_call.method, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
                    || static_method_call.arguments.arguments.iter().any(|argument| match argument {
                        Argument::Positional(positional_argument) => expression_has_throws(&positional_argument.value),
                        Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                    })
            }
        },
        Expression::Access(access) => match access.as_ref() {
            Access::Property(property_access) => {
                expression_has_throws(&property_access.object)
                    || matches!(&property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                expression_has_throws(&null_safe_property_access.object)
                    || matches!(&null_safe_property_access.property, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
            }
            Access::StaticProperty(static_property_access) => expression_has_throws(&static_property_access.class),
            Access::ClassConstant(class_constant_access) => {
                expression_has_throws(&class_constant_access.class)
                    || matches!(&class_constant_access.constant, ClassLikeConstantSelector::Expression(selector) if expression_has_throws(&selector.expression))
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation.as_ref() {
            ClosureCreation::Function(function_closure_creation) => {
                expression_has_throws(&function_closure_creation.function)
            }
            ClosureCreation::Method(method_closure_creation) => {
                expression_has_throws(&method_closure_creation.object)
                    || matches!(&method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                expression_has_throws(&static_method_closure_creation.class)
                    || matches!(&static_method_closure_creation.method, ClassLikeMemberSelector::Expression(selector) if expression_has_throws(&selector.expression))
            }
        },
        Expression::Instantiation(instantiation) => {
            expression_has_throws(&instantiation.class)
                || instantiation
                    .arguments
                    .as_ref()
                    .map(|arguments| {
                        arguments.arguments.iter().any(|argument| match argument {
                            Argument::Positional(positional_argument) => {
                                expression_has_throws(&positional_argument.value)
                            }
                            Argument::Named(named_argument) => expression_has_throws(&named_argument.value),
                        })
                    })
                    .unwrap_or(false)
        }
        Expression::Yield(y) => match y.as_ref() {
            Yield::Value(yield_value) => {
                if let Some(v) = &yield_value.value {
                    expression_has_throws(v)
                } else {
                    false
                }
            }
            Yield::Pair(yield_pair) => {
                expression_has_throws(&yield_pair.key) || expression_has_throws(&yield_pair.value)
            }
            Yield::From(yield_from) => expression_has_throws(&yield_from.iterator),
        },
        Expression::Throw(_) => true,
        _ => false,
    }
}

/// Get the assignment operation from an expression.
///
/// This function will recursively search through the expression and its children to find
///  the first assignment operation.
///
/// If no assignment operation is found, it will return `None`.
#[inline]
pub fn get_assignment_from_expression<'ast>(expression: &'ast Expression) -> Option<&'ast AssignmentOperation> {
    match &expression {
        Expression::AssignmentOperation(assignment_operation) => Some(assignment_operation),
        Expression::Parenthesized(parenthesized) => get_assignment_from_expression(&parenthesized.expression),
        Expression::Referenced(referenced) => get_assignment_from_expression(&referenced.expression),
        Expression::Suppressed(suppressed) => get_assignment_from_expression(&suppressed.expression),
        Expression::ArithmeticOperation(arithmetic_operation) => match arithmetic_operation.as_ref() {
            ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                get_assignment_from_expression(&arithmetic_prefix_operation.value)
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                get_assignment_from_expression(&arithmetic_infix_operation.lhs)
                    .or_else(|| get_assignment_from_expression(&arithmetic_infix_operation.rhs))
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                get_assignment_from_expression(&arithmetic_postfix_operation.value)
            }
        },
        Expression::BitwiseOperation(bitwise_operation) => match bitwise_operation.as_ref() {
            BitwiseOperation::Prefix(bitwise_prefix_operation) => {
                get_assignment_from_expression(&bitwise_prefix_operation.value)
            }
            BitwiseOperation::Infix(bitwise_infix_operation) => {
                get_assignment_from_expression(&bitwise_infix_operation.lhs)
                    .or_else(|| get_assignment_from_expression(&bitwise_infix_operation.rhs))
            }
        },
        Expression::ComparisonOperation(comparison_operation) => {
            get_assignment_from_expression(&comparison_operation.lhs)
                .or_else(|| get_assignment_from_expression(&comparison_operation.rhs))
        }
        Expression::LogicalOperation(logical_operation) => match logical_operation.as_ref() {
            LogicalOperation::Prefix(logical_prefix_operation) => {
                get_assignment_from_expression(&logical_prefix_operation.value)
            }
            LogicalOperation::Infix(logical_infix_operation) => {
                get_assignment_from_expression(&logical_infix_operation.lhs)
                    .or_else(|| get_assignment_from_expression(&logical_infix_operation.rhs))
            }
        },
        Expression::CastOperation(cast_operation) => get_assignment_from_expression(&cast_operation.value),
        Expression::TernaryOperation(ternary_operation) => match ternary_operation.as_ref() {
            TernaryOperation::Conditional(conditional_ternary_operation) => {
                get_assignment_from_expression(&conditional_ternary_operation.condition)
                    .or_else(|| {
                        conditional_ternary_operation
                            .then
                            .as_ref()
                            .and_then(|then| get_assignment_from_expression(then))
                    })
                    .or_else(|| get_assignment_from_expression(&conditional_ternary_operation.r#else))
            }
            TernaryOperation::Elvis(elvis_ternary_operation) => {
                get_assignment_from_expression(&elvis_ternary_operation.condition)
                    .or_else(|| get_assignment_from_expression(&elvis_ternary_operation.r#else))
            }
        },
        Expression::CoalesceOperation(coalesce_operation) => get_assignment_from_expression(&coalesce_operation.lhs)
            .or_else(|| get_assignment_from_expression(&coalesce_operation.rhs)),
        Expression::ConcatOperation(concat_operation) => get_assignment_from_expression(&concat_operation.lhs)
            .or_else(|| get_assignment_from_expression(&concat_operation.rhs)),
        Expression::InstanceofOperation(instanceof_operation) => {
            get_assignment_from_expression(&instanceof_operation.lhs)
                .or_else(|| get_assignment_from_expression(&instanceof_operation.rhs))
        }
        Expression::Array(array) => array.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(&key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(&key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(&variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::LegacyArray(legacy_array) => legacy_array.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(&key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(&key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(&variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::List(list) => list.elements.iter().find_map(|element| match &element {
            ArrayElement::KeyValue(key_value_array_element) => {
                get_assignment_from_expression(&key_value_array_element.key)
                    .or_else(|| get_assignment_from_expression(&key_value_array_element.value))
            }
            ArrayElement::Value(value_array_element) => get_assignment_from_expression(&value_array_element.value),
            ArrayElement::Variadic(variadic_array_element) => {
                get_assignment_from_expression(&variadic_array_element.value)
            }
            ArrayElement::Missing(_) => None,
        }),
        Expression::ArrayAccess(array_access) => get_assignment_from_expression(&array_access.array)
            .or_else(|| get_assignment_from_expression(&array_access.index)),
        Expression::ArrayAppend(array_append) => get_assignment_from_expression(&array_append.array),
        Expression::Match(r#match) => get_assignment_from_expression(&r#match.expression).or_else(|| {
            r#match.arms.iter().find_map(|arm| match arm {
                MatchArm::Expression(match_expression_arm) => match_expression_arm
                    .conditions
                    .iter()
                    .find_map(|condition| get_assignment_from_expression(condition))
                    .or_else(|| get_assignment_from_expression(&match_expression_arm.expression)),
                MatchArm::Default(match_default_arm) => get_assignment_from_expression(&match_default_arm.expression),
            })
        }),
        Expression::Yield(r#yield) => match r#yield.as_ref() {
            Yield::Value(yield_value) => {
                yield_value.value.as_ref().and_then(|value| get_assignment_from_expression(value))
            }
            Yield::Pair(yield_pair) => get_assignment_from_expression(&yield_pair.key)
                .or_else(|| get_assignment_from_expression(&yield_pair.value)),
            Yield::From(yield_from) => get_assignment_from_expression(&yield_from.iterator),
        },
        Expression::Construct(construct) => match construct.as_ref() {
            Construct::Isset(isset_construct) => {
                isset_construct.values.iter().find_map(|v| get_assignment_from_expression(v))
            }
            Construct::Empty(empty_construct) => get_assignment_from_expression(&empty_construct.value),
            Construct::Eval(eval_construct) => get_assignment_from_expression(&eval_construct.value),
            Construct::Include(include_construct) => get_assignment_from_expression(&include_construct.value),
            Construct::IncludeOnce(include_once_construct) => {
                get_assignment_from_expression(&include_once_construct.value)
            }
            Construct::Require(require_construct) => get_assignment_from_expression(&require_construct.value),
            Construct::RequireOnce(require_once_construct) => {
                get_assignment_from_expression(&require_once_construct.value)
            }
            Construct::Print(print_construct) => get_assignment_from_expression(&print_construct.value),
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
        Expression::Throw(throw) => get_assignment_from_expression(&throw.exception),
        Expression::Clone(clone) => get_assignment_from_expression(&clone.object),
        Expression::Call(call) => match &call {
            Call::Function(function_call) => get_assignment_from_expression(&function_call.function).or_else(|| {
                function_call.arguments.arguments.iter().find_map(|argument| match &argument {
                    Argument::Positional(positional_argument) => {
                        get_assignment_from_expression(&positional_argument.value)
                    }
                    Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                })
            }),
            Call::Method(method_call) => get_assignment_from_expression(&method_call.object)
                .or_else(|| match &method_call.method {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(&class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
                .or_else(|| {
                    method_call.arguments.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                }),
            Call::NullSafeMethod(null_safe_method_call) => {
                get_assignment_from_expression(&null_safe_method_call.object)
                    .or_else(|| match &null_safe_method_call.method {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(&class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    })
                    .or_else(|| {
                        null_safe_method_call.arguments.arguments.iter().find_map(|argument| match &argument {
                            Argument::Positional(positional_argument) => {
                                get_assignment_from_expression(&positional_argument.value)
                            }
                            Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                        })
                    })
            }
            Call::StaticMethod(static_method_call) => get_assignment_from_expression(&static_method_call.class)
                .or_else(|| match &static_method_call.method {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(&class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
                .or_else(|| {
                    static_method_call.arguments.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                }),
        },
        Expression::Access(access) => match access.as_ref() {
            Access::Property(property_access) => {
                get_assignment_from_expression(&property_access.object).or_else(|| match &property_access.property {
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        get_assignment_from_expression(&class_like_member_expression_selector.expression)
                    }
                    _ => None,
                })
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                get_assignment_from_expression(&null_safe_property_access.object).or_else(|| {
                    match &null_safe_property_access.property {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(&class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
            Access::StaticProperty(static_property_access) => {
                get_assignment_from_expression(&static_property_access.class)
            }
            Access::ClassConstant(class_constant_access) => {
                get_assignment_from_expression(&class_constant_access.class).or_else(|| {
                    match &class_constant_access.constant {
                        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(&class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation.as_ref() {
            ClosureCreation::Function(function_closure_creation) => {
                get_assignment_from_expression(&function_closure_creation.function)
            }
            ClosureCreation::Method(method_closure_creation) => {
                get_assignment_from_expression(&method_closure_creation.object).or_else(|| {
                    match &method_closure_creation.method {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(&class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                get_assignment_from_expression(&static_method_closure_creation.class).or_else(|| {
                    match &static_method_closure_creation.method {
                        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                            get_assignment_from_expression(&class_like_member_expression_selector.expression)
                        }
                        _ => None,
                    }
                })
            }
        },
        Expression::Instantiation(instantiation) => {
            get_assignment_from_expression(&instantiation.class).or_else(|| {
                instantiation.arguments.as_ref().and_then(|arguments| {
                    arguments.arguments.iter().find_map(|argument| match &argument {
                        Argument::Positional(positional_argument) => {
                            get_assignment_from_expression(&positional_argument.value)
                        }
                        Argument::Named(named_argument) => get_assignment_from_expression(&named_argument.value),
                    })
                })
            })
        }
        _ => None,
    }
}

/// Determine if an expression is truthy.
///
/// This function will return true if the expression is truthy, and false otherwise.
///
/// When this function returns true, it does not necessarily mean that the expression will always evaluate to true.
/// It simply means that the expression is truthy in the context of PHP.
#[inline]
pub fn is_truthy(expression: &Expression) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => is_truthy(&parenthesized.expression),
        Expression::Referenced(referenced) => is_truthy(&referenced.expression),
        Expression::Suppressed(suppressed) => is_truthy(&suppressed.expression),
        Expression::Literal(literal) => match &literal {
            Literal::True(_) => true,
            _ => false,
        },
        Expression::CoalesceOperation(coalesce_operation) => is_truthy(&coalesce_operation.lhs),
        Expression::AnonymousClass(_) => true,
        Expression::Closure(_) => true,
        Expression::ArrowFunction(_) => true,
        Expression::Array(array) => !array.elements.is_empty(),
        Expression::LegacyArray(array) => !array.elements.is_empty(),
        Expression::ClosureCreation(_) => true,
        Expression::LogicalOperation(operation) => match operation.as_ref() {
            LogicalOperation::Prefix(logical_prefix_operation) => return is_falsy(&logical_prefix_operation.value),
            LogicalOperation::Infix(logical_infix_operation) => match &logical_infix_operation.operator {
                LogicalInfixOperator::LowPrecedenceAnd(_) | LogicalInfixOperator::And(_) => {
                    is_truthy(&logical_infix_operation.lhs) && is_truthy(&logical_infix_operation.rhs)
                }
                LogicalInfixOperator::LowPrecedenceOr(_) | LogicalInfixOperator::Or(_) => {
                    is_truthy(&logical_infix_operation.lhs) || is_truthy(&logical_infix_operation.rhs)
                }
                _ => false,
            },
        },
        Expression::AssignmentOperation(assignment) => is_truthy(&assignment.rhs),
        _ => false,
    }
}

/// Determine if an expression is falsy.
///
/// This function will return true if the expression is falsy, and false otherwise.
///
/// When this function returns false, it does not mean that the expression is truthy,
/// it just means that we could not determine if the expression is falsy.
#[inline]
pub fn is_falsy(expression: &Expression) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => is_falsy(&parenthesized.expression),
        Expression::Referenced(referenced) => is_falsy(&referenced.expression),
        Expression::Suppressed(suppressed) => is_falsy(&suppressed.expression),
        Expression::Literal(literal) => match &literal {
            Literal::False(_) | Literal::Null(_) => true,
            _ => false,
        },
        Expression::Array(array) => array.elements.is_empty(),
        Expression::LegacyArray(array) => array.elements.is_empty(),
        Expression::CoalesceOperation(coalesce_operation) => {
            is_falsy(&coalesce_operation.lhs) && is_falsy(&coalesce_operation.rhs)
        }
        Expression::LogicalOperation(operation) => match operation.as_ref() {
            LogicalOperation::Prefix(logical_prefix_operation) => return is_truthy(&logical_prefix_operation.value),
            LogicalOperation::Infix(logical_infix_operation) => match &logical_infix_operation.operator {
                LogicalInfixOperator::LowPrecedenceAnd(_) | LogicalInfixOperator::And(_) => {
                    is_falsy(&logical_infix_operation.lhs) || is_falsy(&logical_infix_operation.rhs)
                }
                LogicalInfixOperator::LowPrecedenceOr(_) | LogicalInfixOperator::Or(_) => {
                    is_falsy(&logical_infix_operation.lhs) && is_falsy(&logical_infix_operation.rhs)
                }
                _ => false,
            },
        },
        Expression::AssignmentOperation(assignment) => is_falsy(&assignment.rhs),
        _ => false,
    }
}

/// Determine if a statement contains only definitions.
#[inline]
pub fn statement_contains_only_definitions<'ast>(statement: &'ast Statement) -> bool {
    let (definitions, statements) = get_statement_stats(&statement);

    definitions != 0 && statements == 0
}

#[inline]
pub fn statement_sequence_contains_only_definitions<'ast>(statement: &'ast Sequence<Statement>) -> bool {
    let mut definitions = 0;
    let mut statements = 0;
    for statement in statement.iter() {
        let (def, stmt) = get_statement_stats(statement);

        definitions += def;
        statements += stmt;
    }

    definitions != 0 && statements == 0
}

#[inline]
fn get_statement_stats<'ast>(statement: &'ast Statement) -> (usize, usize) {
    let mut total_definitions = 0;
    let mut total_statements = 0;

    match &statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                let (definitions, statements) = get_statement_stats(statement);
                total_definitions += definitions;
                total_statements += statements;
            }
        }
        Statement::Block(block) => {
            for statement in block.statements.iter() {
                let (definitions, statements) = get_statement_stats(statement);
                total_definitions += definitions;
                total_statements += statements;
            }
        }
        Statement::Class(_)
        | Statement::Interface(_)
        | Statement::Trait(_)
        | Statement::Enum(_)
        | Statement::Function(_)
        | Statement::Constant(_) => {
            total_definitions += 1;
        }
        _ => {
            total_statements += 1;
        }
    }

    (total_definitions, total_statements)
}
