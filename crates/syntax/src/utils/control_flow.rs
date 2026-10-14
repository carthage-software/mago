use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Access;
use crate::cst::Array;
use crate::cst::ArrayElement;
use crate::cst::Block;
use crate::cst::Break;
use crate::cst::Call;
use crate::cst::ClassLikeConstantSelector;
use crate::cst::ClassLikeMemberSelector;
use crate::cst::Construct;
use crate::cst::Continue;
use crate::cst::Expression;
use crate::cst::ForBody;
use crate::cst::ForeachBody;
use crate::cst::ForeachTarget;
use crate::cst::IfBody;
use crate::cst::LegacyArray;
use crate::cst::List;
use crate::cst::Literal;
use crate::cst::LiteralInteger;
use crate::cst::MatchArm;
use crate::cst::PartialApplication;
use crate::cst::Return;
use crate::cst::Statement;
use crate::cst::StringPart;
use crate::cst::SwitchBody;
use crate::cst::SwitchCase;
use crate::cst::Throw;
use crate::cst::Variable;
use crate::cst::WhileBody;
use crate::cst::Yield;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow<'arena> {
    Return(&'arena Return<'arena>),
    Throw(&'arena Throw<'arena>),
    Continue(&'arena Continue<'arena>),
    Break(&'arena Break<'arena>),
}

impl HasSpan for ControlFlow<'_> {
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
#[must_use]
pub fn find_control_flows_in_block<'arena>(block: &'arena Block<'arena>) -> Vec<ControlFlow<'arena>> {
    let mut controls = vec![];
    visit_control_flows_in_block(block, &mut |control| {
        controls.push(control);
        true
    });

    controls
}

#[inline]
#[must_use]
pub fn find_control_flows_in_statement<'arena>(statement: &'arena Statement<'arena>) -> Vec<ControlFlow<'arena>> {
    let mut controls = vec![];
    visit_control_flows_in_statement(statement, &mut |control| {
        controls.push(control);
        true
    });

    controls
}

#[inline]
#[must_use]
pub fn find_control_flows_in_expression<'arena>(expression: &'arena Expression<'arena>) -> Vec<ControlFlow<'arena>> {
    let mut controls = vec![];
    visit_control_flows_in_expression(expression, &mut |control| {
        controls.push(control);
        true
    });

    controls
}

pub fn visit_control_flows_in_block<'arena>(
    block: &'arena Block<'arena>,
    visitor: &mut dyn FnMut(ControlFlow<'arena>) -> bool,
) -> bool {
    for statement in &block.statements {
        if !visit_control_flows_in_statement(statement, visitor) {
            return false;
        }
    }

    true
}

pub fn visit_control_flows_in_statement<'arena>(
    statement: &'arena Statement<'arena>,
    visitor: &mut dyn FnMut(ControlFlow<'arena>) -> bool,
) -> bool {
    match statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements() {
                if !visit_control_flows_in_statement(statement, visitor) {
                    return false;
                }
            }
        }
        Statement::Block(block) => {
            if !visit_control_flows_in_block(block, visitor) {
                return false;
            }
        }
        Statement::Try(r#try) => {
            if !visit_control_flows_in_block(&r#try.block, visitor) {
                return false;
            }

            for catch in &r#try.catch_clauses {
                if !visit_control_flows_in_block(&catch.block, visitor) {
                    return false;
                }
            }

            if let Some(finally) = &r#try.finally_clause {
                if !visit_control_flows_in_block(&finally.block, visitor) {
                    return false;
                }
            }
        }
        Statement::Foreach(foreach) => {
            if !visit_control_flows_in_expression(foreach.expression, visitor) {
                return false;
            }
            match &foreach.target {
                ForeachTarget::Value(foreach_value_target) => {
                    if !visit_control_flows_in_expression(foreach_value_target.value, visitor) {
                        return false;
                    }
                }
                ForeachTarget::KeyValue(foreach_key_value_target) => {
                    if !visit_control_flows_in_expression(foreach_key_value_target.key, visitor) {
                        return false;
                    }
                    if !visit_control_flows_in_expression(foreach_key_value_target.value, visitor) {
                        return false;
                    }
                }
            }

            match &foreach.body {
                ForeachBody::Statement(statement) => {
                    if !visit_control_flows_in_statement(statement, visitor) {
                        return false;
                    }
                }
                ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in &foreach_colon_delimited_body.statements {
                        if !visit_control_flows_in_statement(statement, visitor) {
                            return false;
                        }
                    }
                }
            }
        }
        Statement::For(r#for) => {
            for initialization in &r#for.initializations {
                if !visit_control_flows_in_expression(initialization, visitor) {
                    return false;
                }
            }

            for condition in &r#for.conditions {
                if !visit_control_flows_in_expression(condition, visitor) {
                    return false;
                }
            }

            for increment in &r#for.increments {
                if !visit_control_flows_in_expression(increment, visitor) {
                    return false;
                }
            }

            match &r#for.body {
                ForBody::Statement(statement) => {
                    if !visit_control_flows_in_statement(statement, visitor) {
                        return false;
                    }
                }
                ForBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in &foreach_colon_delimited_body.statements {
                        if !visit_control_flows_in_statement(statement, visitor) {
                            return false;
                        }
                    }
                }
            }
        }
        Statement::While(r#while) => {
            if !visit_control_flows_in_expression(r#while.condition, visitor) {
                return false;
            }

            match &r#while.body {
                WhileBody::Statement(statement) => {
                    if !visit_control_flows_in_statement(statement, visitor) {
                        return false;
                    }
                }
                WhileBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in &foreach_colon_delimited_body.statements {
                        if !visit_control_flows_in_statement(statement, visitor) {
                            return false;
                        }
                    }
                }
            }
        }
        Statement::DoWhile(do_while) => {
            if !visit_control_flows_in_expression(do_while.condition, visitor) {
                return false;
            }
            if !visit_control_flows_in_statement(do_while.statement, visitor) {
                return false;
            }
        }
        Statement::Switch(switch) => {
            if !visit_control_flows_in_expression(switch.expression, visitor) {
                return false;
            }

            let cases = match &switch.body {
                SwitchBody::BraceDelimited(switch_brace_delimited_body) => &switch_brace_delimited_body.cases,
                SwitchBody::ColonDelimited(switch_colon_delimited_body) => &switch_colon_delimited_body.cases,
            };

            let mut switch_visitor = |control: ControlFlow<'arena>| match control {
                ControlFlow::Break(r#break)
                    if matches!(
                        r#break.level,
                        Some(Expression::Literal(Literal::Integer(LiteralInteger { value: Some(1), .. }))) | None
                    ) =>
                {
                    true
                }
                control => visitor(control),
            };

            for case in cases {
                match &case {
                    SwitchCase::Expression(switch_expression_case) => {
                        if !visit_control_flows_in_expression(switch_expression_case.expression, &mut switch_visitor) {
                            return false;
                        }

                        for statement in &switch_expression_case.statements {
                            if !visit_control_flows_in_statement(statement, &mut switch_visitor) {
                                return false;
                            }
                        }
                    }
                    SwitchCase::Default(switch_default_case) => {
                        for statement in &switch_default_case.statements {
                            if !visit_control_flows_in_statement(statement, &mut switch_visitor) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        Statement::If(r#if) => {
            if !visit_control_flows_in_expression(r#if.condition, visitor) {
                return false;
            }

            match &r#if.body {
                IfBody::Statement(if_statement_body) => {
                    if !visit_control_flows_in_statement(if_statement_body.statement, visitor) {
                        return false;
                    }

                    for else_if in &if_statement_body.else_if_clauses {
                        if !visit_control_flows_in_expression(else_if.condition, visitor) {
                            return false;
                        }
                        if !visit_control_flows_in_statement(else_if.statement, visitor) {
                            return false;
                        }
                    }

                    if let Some(else_clause) = &if_statement_body.else_clause {
                        if !visit_control_flows_in_statement(else_clause.statement, visitor) {
                            return false;
                        }
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    for statement in &if_colon_delimited_body.statements {
                        if !visit_control_flows_in_statement(statement, visitor) {
                            return false;
                        }
                    }

                    for else_if in &if_colon_delimited_body.else_if_clauses {
                        if !visit_control_flows_in_expression(else_if.condition, visitor) {
                            return false;
                        }
                        for statement in &else_if.statements {
                            if !visit_control_flows_in_statement(statement, visitor) {
                                return false;
                            }
                        }
                    }

                    if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                        for statement in &else_clause.statements {
                            if !visit_control_flows_in_statement(statement, visitor) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        Statement::Return(r#return) => {
            if !visitor(ControlFlow::Return(r#return)) {
                return false;
            }
            if let Some(value) = &r#return.value {
                if !visit_control_flows_in_expression(value, visitor) {
                    return false;
                }
            }
        }
        Statement::Continue(r#continue) => {
            if !visitor(ControlFlow::Continue(r#continue)) {
                return false;
            }
            if let Some(level) = &r#continue.level {
                if !visit_control_flows_in_expression(level, visitor) {
                    return false;
                }
            }
        }
        Statement::Break(r#break) => {
            if !visitor(ControlFlow::Break(r#break)) {
                return false;
            }

            if let Some(level) = &r#break.level {
                if !visit_control_flows_in_expression(level, visitor) {
                    return false;
                }
            }
        }
        Statement::Expression(expression_statement) => {
            if !visit_control_flows_in_expression(expression_statement.expression, visitor) {
                return false;
            }
        }
        Statement::Echo(echo) => {
            for expression in &echo.values {
                if !visit_control_flows_in_expression(expression, visitor) {
                    return false;
                }
            }
        }
        Statement::Unset(unset) => {
            for value in &unset.values {
                if !visit_control_flows_in_expression(value, visitor) {
                    return false;
                }
            }
        }
        _ => {}
    }

    true
}

pub fn visit_control_flows_in_expression<'arena>(
    expression: &'arena Expression<'arena>,
    visitor: &mut dyn FnMut(ControlFlow<'arena>) -> bool,
) -> bool {
    match expression {
        Expression::Binary(binary) => {
            if !visit_control_flows_in_expression(binary.lhs, visitor) {
                return false;
            }
            if !visit_control_flows_in_expression(binary.rhs, visitor) {
                return false;
            }
        }
        Expression::UnaryPrefix(unary_prefix) => {
            if !visit_control_flows_in_expression(unary_prefix.operand, visitor) {
                return false;
            }
        }
        Expression::UnaryPostfix(unary_postfix) => {
            if !visit_control_flows_in_expression(unary_postfix.operand, visitor) {
                return false;
            }
        }
        Expression::Parenthesized(parenthesized) => {
            if !visit_control_flows_in_expression(parenthesized.expression, visitor) {
                return false;
            }
        }
        Expression::CompositeString(composite_string) => {
            for part in composite_string.parts() {
                match part {
                    StringPart::Expression(expression) => {
                        if !visit_control_flows_in_expression(expression, visitor) {
                            return false;
                        }
                    }
                    StringPart::BracedExpression(braced_expression_string_part) => {
                        if !visit_control_flows_in_expression(braced_expression_string_part.expression, visitor) {
                            return false;
                        }
                    }
                    StringPart::Literal(_) => {}
                }
            }
        }
        Expression::Assignment(assignment) => {
            if !visit_control_flows_in_expression(assignment.lhs, visitor) {
                return false;
            }
            if !visit_control_flows_in_expression(assignment.rhs, visitor) {
                return false;
            }
        }
        Expression::Conditional(conditional) => {
            if !visit_control_flows_in_expression(conditional.condition, visitor) {
                return false;
            }
            if let Some(then) = &conditional.then {
                if !visit_control_flows_in_expression(then, visitor) {
                    return false;
                }
            }

            if !visit_control_flows_in_expression(conditional.r#else, visitor) {
                return false;
            }
        }
        Expression::Array(Array { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. })
        | Expression::List(List { elements, .. }) => {
            for element in elements {
                match element {
                    ArrayElement::KeyValue(key_value_array_element) => {
                        if !visit_control_flows_in_expression(key_value_array_element.key, visitor) {
                            return false;
                        }
                        if !visit_control_flows_in_expression(key_value_array_element.value, visitor) {
                            return false;
                        }
                    }
                    ArrayElement::Value(value_array_element) => {
                        if !visit_control_flows_in_expression(value_array_element.value, visitor) {
                            return false;
                        }
                    }
                    ArrayElement::Variadic(variadic_array_element) => {
                        if !visit_control_flows_in_expression(variadic_array_element.value, visitor) {
                            return false;
                        }
                    }
                    ArrayElement::Missing(_) => {}
                }
            }
        }
        Expression::ArrayAccess(array_access) => {
            if !visit_control_flows_in_expression(array_access.array, visitor) {
                return false;
            }
            if !visit_control_flows_in_expression(array_access.index, visitor) {
                return false;
            }
        }
        Expression::ArrayAppend(array_append) => {
            if !visit_control_flows_in_expression(array_append.array, visitor) {
                return false;
            }
        }
        Expression::AnonymousClass(anonymous_class) => {
            if let Some(arguments) = &anonymous_class.argument_list {
                for argument in &arguments.arguments {
                    let Some(value) = argument.value() else {
                        continue;
                    };

                    if !visit_control_flows_in_expression(value, visitor) {
                        return false;
                    }
                }
            }
        }
        Expression::Match(r#match) => {
            if !visit_control_flows_in_expression(r#match.expression, visitor) {
                return false;
            }
            for arm in &r#match.arms {
                match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        for condition in &match_expression_arm.conditions {
                            if !visit_control_flows_in_expression(condition, visitor) {
                                return false;
                            }
                        }

                        if !visit_control_flows_in_expression(match_expression_arm.expression, visitor) {
                            return false;
                        }
                    }
                    MatchArm::Default(match_default_arm) => {
                        if !visit_control_flows_in_expression(match_default_arm.expression, visitor) {
                            return false;
                        }
                    }
                }
            }
        }
        Expression::Yield(r#yield) => match r#yield {
            Yield::Value(yield_value) => {
                if let Some(value) = &yield_value.value {
                    if !visit_control_flows_in_expression(value, visitor) {
                        return false;
                    }
                }
            }
            Yield::Pair(yield_pair) => {
                if !visit_control_flows_in_expression(yield_pair.key, visitor) {
                    return false;
                }
                if !visit_control_flows_in_expression(yield_pair.value, visitor) {
                    return false;
                }
            }
            Yield::From(yield_from) => {
                if !visit_control_flows_in_expression(yield_from.iterator, visitor) {
                    return false;
                }
            }
        },
        Expression::Construct(construct) => match construct {
            Construct::Isset(isset_construct) => {
                for expression in &isset_construct.values {
                    if !visit_control_flows_in_expression(expression, visitor) {
                        return false;
                    }
                }
            }
            Construct::Empty(empty_construct) => {
                if !visit_control_flows_in_expression(empty_construct.value, visitor) {
                    return false;
                }
            }
            Construct::Eval(eval_construct) => {
                if !visit_control_flows_in_expression(eval_construct.value, visitor) {
                    return false;
                }
            }
            Construct::Include(include_construct) => {
                if !visit_control_flows_in_expression(include_construct.value, visitor) {
                    return false;
                }
            }
            Construct::IncludeOnce(include_once_construct) => {
                if !visit_control_flows_in_expression(include_once_construct.value, visitor) {
                    return false;
                }
            }
            Construct::Require(require_construct) => {
                if !visit_control_flows_in_expression(require_construct.value, visitor) {
                    return false;
                }
            }
            Construct::RequireOnce(require_once_construct) => {
                if !visit_control_flows_in_expression(require_once_construct.value, visitor) {
                    return false;
                }
            }
            Construct::Print(print_construct) => {
                if !visit_control_flows_in_expression(print_construct.value, visitor) {
                    return false;
                }
            }
            Construct::Exit(exit_construct) => {
                if let Some(arguments) = &exit_construct.arguments {
                    for argument in &arguments.arguments {
                        if !visit_control_flows_in_expression(argument.value(), visitor) {
                            return false;
                        }
                    }
                }
            }
            Construct::Die(die_construct) => {
                if let Some(arguments) = &die_construct.arguments {
                    for argument in &arguments.arguments {
                        if !visit_control_flows_in_expression(argument.value(), visitor) {
                            return false;
                        }
                    }
                }
            }
        },
        Expression::Throw(throw) => {
            if !visitor(ControlFlow::Throw(throw)) {
                return false;
            }
        }
        Expression::Clone(clone) => {
            if !visit_control_flows_in_expression(clone.object, visitor) {
                return false;
            }
        }
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                if !visit_control_flows_in_expression(function_call.function, visitor) {
                    return false;
                }
                for argument in &function_call.argument_list.arguments {
                    if !visit_control_flows_in_expression(argument.value(), visitor) {
                        return false;
                    }
                }
            }
            Call::Method(method_call) => {
                if !visit_control_flows_in_expression(method_call.object, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&method_call.method, visitor) {
                    return false;
                }

                for argument in &method_call.argument_list.arguments {
                    if !visit_control_flows_in_expression(argument.value(), visitor) {
                        return false;
                    }
                }
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                if !visit_control_flows_in_expression(null_safe_method_call.object, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&null_safe_method_call.method, visitor) {
                    return false;
                }

                for argument in &null_safe_method_call.argument_list.arguments {
                    if !visit_control_flows_in_expression(argument.value(), visitor) {
                        return false;
                    }
                }
            }
            Call::StaticMethod(static_method_call) => {
                if !visit_control_flows_in_expression(static_method_call.class, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&static_method_call.method, visitor) {
                    return false;
                }

                for argument in &static_method_call.argument_list.arguments {
                    if !visit_control_flows_in_expression(argument.value(), visitor) {
                        return false;
                    }
                }
            }
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                if !visit_control_flows_in_expression(property_access.object, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&property_access.property, visitor) {
                    return false;
                }
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                if !visit_control_flows_in_expression(null_safe_property_access.object, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&null_safe_property_access.property, visitor) {
                    return false;
                }
            }
            Access::StaticProperty(static_property_access) => {
                if !visit_control_flows_in_expression(static_property_access.class, visitor) {
                    return false;
                }
                if !visit_control_flows_in_variable(&static_property_access.property, visitor) {
                    return false;
                }
            }
            Access::ClassConstant(class_constant_access) => {
                if !visit_control_flows_in_expression(class_constant_access.class, visitor) {
                    return false;
                }
                if let ClassLikeConstantSelector::Expression(class_like_member_expression_selector) =
                    &class_constant_access.constant
                {
                    if !visit_control_flows_in_expression(class_like_member_expression_selector.expression, visitor) {
                        return false;
                    }
                }
            }
        },
        Expression::Variable(variable) => {
            if !visit_control_flows_in_variable(variable, visitor) {
                return false;
            }
        }
        Expression::PartialApplication(partial_application) => match partial_application {
            PartialApplication::Function(function_partial_application) => {
                if !visit_control_flows_in_expression(function_partial_application.function, visitor) {
                    return false;
                }
            }
            PartialApplication::Method(method_partial_application) => {
                if !visit_control_flows_in_expression(method_partial_application.object, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&method_partial_application.method, visitor) {
                    return false;
                }
            }
            PartialApplication::StaticMethod(static_method_partial_application) => {
                if !visit_control_flows_in_expression(static_method_partial_application.class, visitor) {
                    return false;
                }
                if !visit_control_flows_in_member_selector(&static_method_partial_application.method, visitor) {
                    return false;
                }
            }
        },
        Expression::Instantiation(instantiation) => {
            if !visit_control_flows_in_expression(instantiation.class, visitor) {
                return false;
            }
            if let Some(argument_list) = &instantiation.argument_list {
                for argument in &argument_list.arguments {
                    if !visit_control_flows_in_expression(argument.value(), visitor) {
                        return false;
                    }
                }
            }
        }
        _ => {}
    }

    true
}

fn visit_control_flows_in_member_selector<'arena>(
    selector: &'arena ClassLikeMemberSelector<'arena>,
    visitor: &mut dyn FnMut(ControlFlow<'arena>) -> bool,
) -> bool {
    match selector {
        ClassLikeMemberSelector::Variable(variable) => visit_control_flows_in_variable(variable, visitor),
        ClassLikeMemberSelector::Expression(selector) => {
            visit_control_flows_in_expression(selector.expression, visitor)
        }
        _ => true,
    }
}

fn visit_control_flows_in_variable<'arena>(
    variable: &'arena Variable<'arena>,
    visitor: &mut dyn FnMut(ControlFlow<'arena>) -> bool,
) -> bool {
    match variable {
        Variable::Indirect(indirect_variable) => {
            visit_control_flows_in_expression(indirect_variable.expression, visitor)
        }
        Variable::Nested(nested_variable) => visit_control_flows_in_variable(nested_variable.variable, visitor),
        Variable::Direct(_) => true,
    }
}
