use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow<'ast, 'arena> {
    Return(&'ast Return<'arena>),
    Throw(&'ast Throw<'arena>),
    Continue(&'ast Continue<'arena>),
    Break(&'ast Break<'arena>),
}

impl HasSpan for ControlFlow<'_, '_> {
    fn span(&self) -> Span {
        match self {
            ControlFlow::Return(r#return) => r#return.span(),
            ControlFlow::Throw(throw) => throw.span(),
            ControlFlow::Continue(r#continue) => r#continue.span(),
            ControlFlow::Break(r#break) => r#break.span(),
        }
    }
}

#[inline]
pub fn find_control_flows_in_block<'ast, 'arena>(block: &'ast Block<'arena>) -> Vec<ControlFlow<'ast, 'arena>> {
    let mut controls = vec![];

    for statement in block.statements.iter() {
        controls.extend(find_control_flows_in_statement(statement));
    }

    controls
}

#[inline]
pub fn find_control_flows_in_statement<'ast, 'arena>(
    statement: &'ast Statement<'arena>,
) -> Vec<ControlFlow<'ast, 'arena>> {
    let mut controls = vec![];

    match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                controls.extend(find_control_flows_in_statement(statement));
            }
        }
        Statement::Block(block) => {
            controls.extend(find_control_flows_in_block(block));
        }
        Statement::Try(r#try) => {
            controls.extend(find_control_flows_in_block(&r#try.block));

            for catch in r#try.catch_clauses.iter() {
                controls.extend(find_control_flows_in_block(&catch.block));
            }

            if let Some(finally) = &r#try.finally_clause {
                controls.extend(find_control_flows_in_block(&finally.block));
            }
        }
        Statement::Foreach(foreach) => {
            controls.extend(find_control_flows_in_expression(foreach.expression));
            match &foreach.target {
                ForeachTarget::Value(foreach_value_target) => {
                    controls.extend(find_control_flows_in_expression(foreach_value_target.value));
                }
                ForeachTarget::KeyValue(foreach_key_value_target) => {
                    controls.extend(find_control_flows_in_expression(foreach_key_value_target.key));
                    controls.extend(find_control_flows_in_expression(foreach_key_value_target.value));
                }
            }

            match &foreach.body {
                ForeachBody::Statement(statement) => {
                    controls.extend(find_control_flows_in_statement(statement));
                }
                ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in foreach_colon_delimited_body.statements.iter() {
                        controls.extend(find_control_flows_in_statement(statement));
                    }
                }
            }
        }
        Statement::For(r#for) => {
            for initialization in r#for.initializations.iter() {
                controls.extend(find_control_flows_in_expression(initialization));
            }

            for condition in r#for.conditions.iter() {
                controls.extend(find_control_flows_in_expression(condition));
            }

            for increment in r#for.increments.iter() {
                controls.extend(find_control_flows_in_expression(increment));
            }

            match &r#for.body {
                ForBody::Statement(statement) => {
                    controls.extend(find_control_flows_in_statement(statement));
                }
                ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in foreach_colon_delimited_body.statements.iter() {
                        controls.extend(find_control_flows_in_statement(statement));
                    }
                }
            }
        }
        Statement::While(r#while) => {
            controls.extend(find_control_flows_in_expression(r#while.condition));

            match &r#while.body {
                WhileBody::Statement(statement) => {
                    controls.extend(find_control_flows_in_statement(statement));
                }
                WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in foreach_colon_delimited_body.statements.iter() {
                        controls.extend(find_control_flows_in_statement(statement));
                    }
                }
            }
        }
        Statement::DoWhile(do_while) => {
            controls.extend(find_control_flows_in_expression(do_while.condition));
            controls.extend(find_control_flows_in_statement(do_while.statement));
        }
        Statement::Switch(switch) => {
            controls.extend(find_control_flows_in_expression(switch.expression));

            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            let mut switch_controls = vec![];
            for case in cases.iter() {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        switch_controls.extend(find_control_flows_in_expression(switch_expression_case.expression));

                        for statement in switch_expression_case.statements.iter() {
                            switch_controls.extend(find_control_flows_in_statement(statement));
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in switch_default_case.statements.iter() {
                            switch_controls.extend(find_control_flows_in_statement(statement));
                        }
                    }
                }
            }

            for control in switch_controls {
                match control {
                    ControlFlow::Break(r#break) => {
                        if !matches!(
                            r#break.level,
                            Some(Expression::Literal(Literal::Integer(LiteralInteger { value: Some(1), .. }))) | None
                        ) {
                            controls.push(control)
                        }
                    }
                    _ => controls.push(control),
                }
            }
        }
        Statement::If(r#if) => {
            controls.extend(find_control_flows_in_expression(r#if.condition));

            match &r#if.body {
                IfBody::Statement(if_statement_body) => {
                    controls.extend(find_control_flows_in_statement(if_statement_body.statement));

                    for else_if in if_statement_body.else_if_clauses.iter() {
                        controls.extend(find_control_flows_in_expression(else_if.condition));
                        controls.extend(find_control_flows_in_statement(else_if.statement));
                    }

                    if let Some(else_clause) = &if_statement_body.else_clause {
                        controls.extend(find_control_flows_in_statement(else_clause.statement));
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    for statement in if_colon_delimited_body.statements.iter() {
                        controls.extend(find_control_flows_in_statement(statement));
                    }

                    for else_if in if_colon_delimited_body.else_if_clauses.iter() {
                        controls.extend(find_control_flows_in_expression(else_if.condition));
                        for statement in else_if.statements.iter() {
                            controls.extend(find_control_flows_in_statement(statement));
                        }
                    }

                    if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                        for statement in else_clause.statements.iter() {
                            controls.extend(find_control_flows_in_statement(statement));
                        }
                    }
                }
            }
        }
        Statement::Return(r#return) => {
            controls.push(ControlFlow::Return(r#return));
            if let Some(value) = &r#return.value {
                controls.extend(find_control_flows_in_expression(value));
            }
        }
        Statement::Continue(r#continue) => {
            controls.push(ControlFlow::Continue(r#continue));
            if let Some(level) = &r#continue.level {
                controls.extend(find_control_flows_in_expression(level));
            }
        }
        Statement::Break(r#break) => {
            controls.push(ControlFlow::Break(r#break));
            if let Some(level) = &r#break.level {
                controls.extend(find_control_flows_in_expression(level));
            }
        }
        Statement::Expression(expression_statement) => {
            controls.extend(find_control_flows_in_expression(expression_statement.expression));
        }
        Statement::Echo(echo) => {
            for expression in echo.values.iter() {
                controls.extend(find_control_flows_in_expression(expression));
            }
        }
        Statement::Unset(unset) => {
            for value in unset.values.iter() {
                controls.extend(find_control_flows_in_expression(value));
            }
        }
        _ => {}
    }

    controls
}

#[inline]
pub fn find_control_flows_in_expression<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Vec<ControlFlow<'ast, 'arena>> {
    let mut controls = vec![];

    match expression {
        Expression::Binary(binary) => {
            controls.extend(find_control_flows_in_expression(binary.lhs));
            controls.extend(find_control_flows_in_expression(binary.rhs));
        }
        Expression::UnaryPrefix(unary_prefix) => {
            controls.extend(find_control_flows_in_expression(unary_prefix.operand));
        }
        Expression::UnaryPostfix(unary_postfix) => {
            controls.extend(find_control_flows_in_expression(unary_postfix.operand));
        }
        Expression::Parenthesized(parenthesized) => {
            controls.extend(find_control_flows_in_expression(parenthesized.expression));
        }
        Expression::CompositeString(composite_string) => {
            for part in composite_string.parts().iter() {
                match part {
                    StringPart::Expression(expression) => {
                        controls.extend(find_control_flows_in_expression(expression));
                    }
                    StringPart::BracedExpression(braced_expression_string_part) => {
                        controls.extend(find_control_flows_in_expression(braced_expression_string_part.expression));
                    }
                    _ => {}
                }
            }
        }
        Expression::Assignment(assignment) => {
            controls.extend(find_control_flows_in_expression(assignment.lhs));
            controls.extend(find_control_flows_in_expression(assignment.rhs));
        }
        Expression::Conditional(conditional) => {
            controls.extend(find_control_flows_in_expression(conditional.condition));
            if let Some(then) = &conditional.then {
                controls.extend(find_control_flows_in_expression(then));
            }

            controls.extend(find_control_flows_in_expression(conditional.r#else));
        }
        Expression::Array(Array { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. })
        | Expression::List(List { elements, .. }) => {
            for element in elements.iter() {
                match element {
                    ArrayElement::KeyValue(key_value_array_element) => {
                        controls.extend(find_control_flows_in_expression(key_value_array_element.key));
                        controls.extend(find_control_flows_in_expression(key_value_array_element.value));
                    }
                    ArrayElement::Value(value_array_element) => {
                        controls.extend(find_control_flows_in_expression(value_array_element.value));
                    }
                    ArrayElement::Variadic(variadic_array_element) => {
                        controls.extend(find_control_flows_in_expression(variadic_array_element.value));
                    }
                    _ => {}
                }
            }
        }
        Expression::ArrayAccess(array_access) => {
            controls.extend(find_control_flows_in_expression(array_access.array));
            controls.extend(find_control_flows_in_expression(array_access.index));
        }
        Expression::ArrayAppend(array_append) => {
            controls.extend(find_control_flows_in_expression(array_append.array));
        }
        Expression::AnonymousClass(anonymous_class) => {
            if let Some(arguments) = &anonymous_class.argument_list {
                for argument in arguments.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
        }
        Expression::Match(r#match) => {
            controls.extend(find_control_flows_in_expression(r#match.expression));
            for arm in r#match.arms.iter() {
                match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        for condition in match_expression_arm.conditions.iter() {
                            controls.extend(find_control_flows_in_expression(condition));
                        }

                        controls.extend(find_control_flows_in_expression(match_expression_arm.expression));
                    }
                    MatchArm::Default(match_default_arm) => {
                        controls.extend(find_control_flows_in_expression(match_default_arm.expression));
                    }
                }
            }
        }
        Expression::Yield(r#yield) => match r#yield {
            Yield::Value(yield_value) => {
                if let Some(value) = &yield_value.value {
                    controls.extend(find_control_flows_in_expression(value));
                }
            }
            Yield::Pair(yield_pair) => {
                controls.extend(find_control_flows_in_expression(yield_pair.key));
                controls.extend(find_control_flows_in_expression(yield_pair.value));
            }
            Yield::From(yield_from) => {
                controls.extend(find_control_flows_in_expression(yield_from.iterator));
            }
        },
        Expression::Construct(construct) => match construct {
            Construct::Isset(isset_construct) => {
                for expression in isset_construct.values.iter() {
                    controls.extend(find_control_flows_in_expression(expression));
                }
            }
            Construct::Empty(empty_construct) => {
                controls.extend(find_control_flows_in_expression(empty_construct.value));
            }
            Construct::Eval(eval_construct) => {
                controls.extend(find_control_flows_in_expression(eval_construct.value));
            }
            Construct::Include(include_construct) => {
                controls.extend(find_control_flows_in_expression(include_construct.value));
            }
            Construct::IncludeOnce(include_once_construct) => {
                controls.extend(find_control_flows_in_expression(include_once_construct.value));
            }
            Construct::Require(require_construct) => {
                controls.extend(find_control_flows_in_expression(require_construct.value));
            }
            Construct::RequireOnce(require_once_construct) => {
                controls.extend(find_control_flows_in_expression(require_once_construct.value));
            }
            Construct::Print(print_construct) => {
                controls.extend(find_control_flows_in_expression(print_construct.value));
            }
            Construct::Exit(exit_construct) => {
                if let Some(arguments) = &exit_construct.arguments {
                    for argument in arguments.arguments.iter() {
                        controls.extend(find_control_flows_in_expression(argument.value()));
                    }
                }
            }
            Construct::Die(die_construct) => {
                if let Some(arguments) = &die_construct.arguments {
                    for argument in arguments.arguments.iter() {
                        controls.extend(find_control_flows_in_expression(argument.value()));
                    }
                }
            }
        },
        Expression::Throw(throw) => {
            controls.push(ControlFlow::Throw(throw));
        }
        Expression::Clone(clone) => {
            controls.extend(find_control_flows_in_expression(clone.object));
        }
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                controls.extend(find_control_flows_in_expression(function_call.function));
                for argument in function_call.argument_list.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
            Call::Method(method_call) => {
                controls.extend(find_control_flows_in_expression(method_call.object));
                match &method_call.method {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }

                for argument in method_call.argument_list.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                controls.extend(find_control_flows_in_expression(null_safe_method_call.object));
                match &null_safe_method_call.method {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }

                for argument in null_safe_method_call.argument_list.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
            Call::StaticMethod(static_method_call) => {
                controls.extend(find_control_flows_in_expression(static_method_call.class));
                match &static_method_call.method {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }

                for argument in static_method_call.argument_list.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                controls.extend(find_control_flows_in_expression(property_access.object));
                match &property_access.property {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                controls.extend(find_control_flows_in_expression(null_safe_property_access.object));
                match &null_safe_property_access.property {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }
            }
            Access::StaticProperty(static_property_access) => {
                controls.extend(find_control_flows_in_expression(static_property_access.class));
                controls.extend(find_control_flows_in_variable(&static_property_access.property));
            }
            Access::ClassConstant(class_constant_access) => {
                controls.extend(find_control_flows_in_expression(class_constant_access.class));
                if let ClassLikeConstantSelector::Expression(class_like_member_expression_selector) =
                    &class_constant_access.constant
                {
                    controls.extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                }
            }
        },
        Expression::Variable(variable) => {
            controls.extend(find_control_flows_in_variable(variable));
        }
        Expression::ClosureCreation(closure_creation) => match closure_creation {
            ClosureCreation::Function(function_closure_creation) => {
                controls.extend(find_control_flows_in_expression(function_closure_creation.function));
            }
            ClosureCreation::Method(method_closure_creation) => {
                controls.extend(find_control_flows_in_expression(method_closure_creation.object));
                match &method_closure_creation.method {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                controls.extend(find_control_flows_in_expression(static_method_closure_creation.class));
                match &static_method_closure_creation.method {
                    ClassLikeMemberSelector::Variable(variable) => {
                        controls.extend(find_control_flows_in_variable(variable));
                    }
                    ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                        controls
                            .extend(find_control_flows_in_expression(class_like_member_expression_selector.expression));
                    }
                    _ => {}
                }
            }
        },
        Expression::Instantiation(instantiation) => {
            controls.extend(find_control_flows_in_expression(instantiation.class));
            if let Some(argument_list) = &instantiation.argument_list {
                for argument in argument_list.arguments.iter() {
                    controls.extend(find_control_flows_in_expression(argument.value()));
                }
            }
        }
        _ => {}
    }

    controls
}

fn find_control_flows_in_variable<'ast, 'arena>(variable: &'ast Variable<'arena>) -> Vec<ControlFlow<'ast, 'arena>> {
    match variable {
        Variable::Indirect(indirect_variable) => find_control_flows_in_expression(indirect_variable.expression),
        Variable::Nested(nested_variable) => find_control_flows_in_variable(nested_variable.variable),
        Variable::Direct(_) => {
            vec![]
        }
    }
}
