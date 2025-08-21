use mago_span::*;

use crate::ast::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MethodReference<'ast, 'arena> {
    MethodCall(&'ast MethodCall<'arena>),
    StaticMethodCall(&'ast StaticMethodCall<'arena>),
    MethodClosureCreation(&'ast MethodClosureCreation<'arena>),
    StaticMethodClosureCreation(&'ast StaticMethodClosureCreation<'arena>),
}

impl<'ast, 'arena> MethodReference<'ast, 'arena> {
    pub fn get_class_or_object(&self) -> &'ast Expression<'arena> {
        match self {
            MethodReference::MethodCall(call) => call.object,
            MethodReference::StaticMethodCall(call) => call.class,
            MethodReference::MethodClosureCreation(closure) => closure.object,
            MethodReference::StaticMethodClosureCreation(closure) => closure.class,
        }
    }

    pub fn get_selector(&self) -> &'ast ClassLikeMemberSelector<'arena> {
        match self {
            MethodReference::MethodCall(call) => &call.method,
            MethodReference::StaticMethodCall(call) => &call.method,
            MethodReference::MethodClosureCreation(closure) => &closure.method,
            MethodReference::StaticMethodClosureCreation(closure) => &closure.method,
        }
    }
}

impl HasSpan for MethodReference<'_, '_> {
    fn span(&self) -> Span {
        match self {
            MethodReference::MethodCall(call) => call.span(),
            MethodReference::StaticMethodCall(call) => call.span(),
            MethodReference::MethodClosureCreation(closure) => closure.span(),
            MethodReference::StaticMethodClosureCreation(closure) => closure.span(),
        }
    }
}

pub fn find_method_references_in_block<'ast, 'arena, F>(
    block: &'ast Block<'arena>,
    predicate: &F,
) -> Vec<MethodReference<'ast, 'arena>>
where
    F: Fn(&MethodReference<'ast, 'arena>) -> bool,
{
    let mut method_references = vec![];
    for statement in block.statements.iter() {
        method_references.extend(find_method_references_in_statement(statement, predicate));
    }

    method_references
}

pub fn find_method_references_in_statement<'ast, 'arena, F>(
    statement: &'ast Statement<'arena>,
    predicate: &F,
) -> Vec<MethodReference<'ast, 'arena>>
where
    F: Fn(&MethodReference<'ast, 'arena>) -> bool,
{
    match statement {
        Statement::Block(block) => {
            let mut references = vec![];
            for statement in block.statements.iter() {
                references.extend(find_method_references_in_statement(statement, predicate));
            }

            references
        }
        Statement::Try(try_catch) => {
            let mut references = vec![];
            for statement in try_catch.block.statements.iter() {
                references.extend(find_method_references_in_statement(statement, predicate));
            }

            for catch in try_catch.catch_clauses.iter() {
                for statement in catch.block.statements.iter() {
                    references.extend(find_method_references_in_statement(statement, predicate));
                }
            }

            if let Some(finally) = &try_catch.finally_clause {
                for statement in finally.block.statements.iter() {
                    references.extend(find_method_references_in_statement(statement, predicate));
                }
            }

            references
        }
        Statement::Foreach(foreach) => {
            let mut references = vec![];

            references.extend(find_method_references_in_expression(foreach.expression, predicate));

            match &foreach.target {
                ForeachTarget::Value(foreach_value_target) => {
                    references.extend(find_method_references_in_expression(foreach_value_target.value, predicate));
                }
                ForeachTarget::KeyValue(foreach_key_value_target) => {
                    references.extend(find_method_references_in_expression(foreach_key_value_target.key, predicate));
                    references.extend(find_method_references_in_expression(foreach_key_value_target.value, predicate));
                }
            }

            match &foreach.body {
                ForeachBody::Statement(statement) => {
                    references.extend(find_method_references_in_statement(statement, predicate));
                }
                ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in foreach_colon_delimited_body.statements.iter() {
                        references.extend(find_method_references_in_statement(statement, predicate));
                    }
                }
            }

            references
        }
        Statement::For(for_loop) => {
            let mut references = vec![];

            for init in for_loop.initializations.iter() {
                references.extend(find_method_references_in_expression(init, predicate));
            }

            for condition in for_loop.conditions.iter() {
                references.extend(find_method_references_in_expression(condition, predicate));
            }

            for increment in for_loop.increments.iter() {
                references.extend(find_method_references_in_expression(increment, predicate));
            }

            match &for_loop.body {
                ForBody::Statement(statement) => {
                    references.extend(find_method_references_in_statement(statement, predicate));
                }
                ForBody::ColonDelimited(for_colon_delimited_body) => {
                    for statement in for_colon_delimited_body.statements.iter() {
                        references.extend(find_method_references_in_statement(statement, predicate));
                    }
                }
            }

            references
        }
        Statement::While(while_loop) => {
            let mut references = vec![];

            references.extend(find_method_references_in_expression(while_loop.condition, predicate));

            match &while_loop.body {
                WhileBody::Statement(statement) => {
                    references.extend(find_method_references_in_statement(statement, predicate));
                }
                WhileBody::ColonDelimited(while_colon_delimited_body) => {
                    for statement in while_colon_delimited_body.statements.iter() {
                        references.extend(find_method_references_in_statement(statement, predicate));
                    }
                }
            }

            references
        }
        Statement::DoWhile(do_while) => {
            let mut references = vec![];

            references.extend(find_method_references_in_expression(do_while.condition, predicate));
            references.extend(find_method_references_in_statement(do_while.statement, predicate));

            references
        }
        Statement::Switch(switch) => {
            let mut references = find_method_references_in_expression(switch.expression, predicate);

            for case in switch.body.cases() {
                match case {
                    SwitchCase::Expression(expression_case) => {
                        references.extend(find_method_references_in_expression(expression_case.expression, predicate));

                        for statement in expression_case.statements.iter() {
                            references.extend(find_method_references_in_statement(statement, predicate));
                        }
                    }
                    SwitchCase::Default(default_case) => {
                        for statement in default_case.statements.iter() {
                            references.extend(find_method_references_in_statement(statement, predicate));
                        }
                    }
                }
            }

            references
        }
        Statement::If(if_stmt) => {
            let mut references = vec![];

            references.extend(find_method_references_in_expression(if_stmt.condition, predicate));
            match &if_stmt.body {
                IfBody::Statement(if_stmt_body) => {
                    references.extend(find_method_references_in_statement(if_stmt_body.statement, predicate));
                    for else_if_clause in if_stmt_body.else_if_clauses.iter() {
                        references.extend(find_method_references_in_expression(else_if_clause.condition, predicate));
                        references.extend(find_method_references_in_statement(else_if_clause.statement, predicate));
                    }

                    if let Some(else_clause) = &if_stmt_body.else_clause {
                        references.extend(find_method_references_in_statement(else_clause.statement, predicate));
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    for statement in if_colon_delimited_body.statements.iter() {
                        references.extend(find_method_references_in_statement(statement, predicate));
                    }

                    for else_if_clause in if_colon_delimited_body.else_if_clauses.iter() {
                        references.extend(find_method_references_in_expression(else_if_clause.condition, predicate));
                        for statement in else_if_clause.statements.iter() {
                            references.extend(find_method_references_in_statement(statement, predicate));
                        }
                    }

                    if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                        for statement in else_clause.statements.iter() {
                            references.extend(find_method_references_in_statement(statement, predicate));
                        }
                    }
                }
            }

            references
        }
        Statement::Return(r#return) => {
            if let Some(expression) = &r#return.value {
                find_method_references_in_expression(expression, predicate)
            } else {
                vec![]
            }
        }
        Statement::Expression(expression_statement) => {
            find_method_references_in_expression(expression_statement.expression, predicate)
        }
        Statement::Echo(echo) => {
            let mut references = vec![];
            for expression in echo.values.iter() {
                references.extend(find_method_references_in_expression(expression, predicate));
            }

            references
        }
        _ => {
            vec![]
        }
    }
}

pub fn find_method_references_in_expression<'ast, 'arena, F>(
    expression: &'ast Expression<'arena>,
    predicate: &F,
) -> Vec<MethodReference<'ast, 'arena>>
where
    F: Fn(&MethodReference<'ast, 'arena>) -> bool,
{
    match expression {
        Expression::Binary(binary) => {
            let mut references = vec![];
            references.extend(find_method_references_in_expression(binary.lhs, predicate));
            references.extend(find_method_references_in_expression(binary.rhs, predicate));

            references
        }
        Expression::UnaryPrefix(unary_prefix) => find_method_references_in_expression(unary_prefix.operand, predicate),
        Expression::UnaryPostfix(unary_postfix) => {
            find_method_references_in_expression(unary_postfix.operand, predicate)
        }
        Expression::Parenthesized(parenthesized) => {
            find_method_references_in_expression(parenthesized.expression, predicate)
        }
        Expression::Assignment(assignment) => {
            let mut references = vec![];
            references.extend(find_method_references_in_expression(assignment.lhs, predicate));
            references.extend(find_method_references_in_expression(assignment.rhs, predicate));

            references
        }
        Expression::Conditional(conditional) => {
            let mut references = vec![];
            references.extend(find_method_references_in_expression(conditional.condition, predicate));
            if let Some(then) = &conditional.then {
                references.extend(find_method_references_in_expression(then, predicate));
            }
            references.extend(find_method_references_in_expression(conditional.r#else, predicate));

            references
        }
        Expression::Array(Array { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. })
        | Expression::List(List { elements, .. }) => {
            let mut references = vec![];
            for element in elements.iter() {
                match element {
                    ArrayElement::KeyValue(kv) => {
                        references.extend(find_method_references_in_expression(kv.key, predicate));
                        references.extend(find_method_references_in_expression(kv.value, predicate));
                    }
                    ArrayElement::Value(v) => {
                        references.extend(find_method_references_in_expression(v.value, predicate));
                    }
                    ArrayElement::Variadic(v) => {
                        references.extend(find_method_references_in_expression(v.value, predicate));
                    }
                    ArrayElement::Missing(_) => {}
                }
            }

            references
        }
        Expression::ArrayAccess(array_access) => {
            let mut references = vec![];
            references.extend(find_method_references_in_expression(array_access.array, predicate));
            references.extend(find_method_references_in_expression(array_access.index, predicate));

            references
        }
        Expression::ArrayAppend(array_append) => find_method_references_in_expression(array_append.array, predicate),
        Expression::AnonymousClass(anonymous_class) => {
            if let Some(argument_list) = &anonymous_class.argument_list {
                find_references_in_argument_list(argument_list, predicate)
            } else {
                vec![]
            }
        }
        Expression::Match(r#match) => {
            let mut references = vec![];
            references.extend(find_method_references_in_expression(r#match.expression, predicate));

            for arm in r#match.arms.iter() {
                match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        for condition in match_expression_arm.conditions.iter() {
                            references.extend(find_method_references_in_expression(condition, predicate));
                        }

                        references
                            .extend(find_method_references_in_expression(match_expression_arm.expression, predicate));
                    }
                    MatchArm::Default(match_default_arm) => {
                        references
                            .extend(find_method_references_in_expression(match_default_arm.expression, predicate));
                    }
                }
            }

            references
        }
        Expression::Yield(r#yield) => match r#yield {
            Yield::Value(yield_value) => match &yield_value.value {
                Some(value) => find_method_references_in_expression(value, predicate),
                None => vec![],
            },
            Yield::Pair(yield_pair) => {
                let mut references = vec![];
                references.extend(find_method_references_in_expression(yield_pair.key, predicate));
                references.extend(find_method_references_in_expression(yield_pair.value, predicate));

                references
            }
            Yield::From(yield_from) => find_method_references_in_expression(yield_from.iterator, predicate),
        },
        Expression::Throw(throw) => find_method_references_in_expression(throw.exception, predicate),
        Expression::Clone(clone) => find_method_references_in_expression(clone.object, predicate),
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                let mut references = vec![];

                references.extend(find_method_references_in_expression(function_call.function, predicate));
                references.extend(find_references_in_argument_list(&function_call.argument_list, predicate));
                references
            }
            Call::Method(method_call) => {
                let reference = MethodReference::MethodCall(method_call);
                let mut references = if predicate(&reference) { vec![reference] } else { vec![] };

                references.extend(find_method_references_in_expression(method_call.object, predicate));
                references.extend(find_references_in_argument_list(&method_call.argument_list, predicate));
                references
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                let mut references = vec![];

                references.extend(find_method_references_in_expression(null_safe_method_call.object, predicate));
                references.extend(find_references_in_argument_list(&null_safe_method_call.argument_list, predicate));
                references
            }
            Call::StaticMethod(static_method_call) => {
                let reference = MethodReference::StaticMethodCall(static_method_call);
                let mut references = if predicate(&reference) { vec![reference] } else { vec![] };

                references.extend(find_method_references_in_expression(static_method_call.class, predicate));
                references.extend(find_references_in_argument_list(&static_method_call.argument_list, predicate));
                references
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation {
            ClosureCreation::Method(method_closure_creation) => {
                let reference = MethodReference::MethodClosureCreation(method_closure_creation);
                let mut references = if predicate(&reference) { vec![reference] } else { vec![] };

                references.extend(find_method_references_in_expression(method_closure_creation.object, predicate));
                references
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                let reference = MethodReference::StaticMethodClosureCreation(static_method_closure_creation);
                let mut references = if predicate(&reference) { vec![reference] } else { vec![] };

                references
                    .extend(find_method_references_in_expression(static_method_closure_creation.class, predicate));
                references
            }
            ClosureCreation::Function(_) => vec![],
        },
        Expression::Instantiation(instantiation) => {
            if let Some(argument_list) = &instantiation.argument_list {
                find_references_in_argument_list(argument_list, predicate)
            } else {
                vec![]
            }
        }
        _ => {
            vec![]
        }
    }
}

fn find_references_in_argument_list<'ast, 'arena, F>(
    argument_list: &'ast ArgumentList<'arena>,
    predicate: &F,
) -> Vec<MethodReference<'ast, 'arena>>
where
    F: Fn(&MethodReference<'ast, 'arena>) -> bool,
{
    let mut references = vec![];
    for argument in argument_list.arguments.iter() {
        match argument {
            Argument::Positional(positional_argument) => {
                references.extend(find_method_references_in_expression(&positional_argument.value, predicate));
            }
            Argument::Named(named_argument) => {
                references.extend(find_method_references_in_expression(&named_argument.value, predicate));
            }
        }
    }

    references
}
