use mago_ast::*;

use crate::context::LintContext;

pub fn is_method_setter_or_getter(method: &Method, context: &LintContext<'_>) -> bool {
    let MethodBody::Concrete(block) = &method.body else {
        return false;
    };

    let statements_len = block.statements.len();
    if statements_len > 2 {
        return false;
    }

    let Some(statement) = block.statements.first() else {
        return false;
    };

    match statement {
        Statement::Return(return_statement) if method.parameter_list.parameters.is_empty() => {
            let Some(expression) = &return_statement.value else {
                return false;
            };

            if !is_accessing_property_of_this(expression, context) {
                return false;
            }

            statements_len == 1
        }
        Statement::Expression(expression_statement) if method.parameter_list.parameters.len() == 1 => {
            let Expression::Assignment(assignment) = expression_statement.expression.as_ref() else {
                return false;
            };

            if !is_accessing_property_of_this(assignment.lhs.as_ref(), context) {
                return false;
            }

            match block.statements.last() {
                Some(statement) => match statement {
                    Statement::Return(return_statement) => {
                        let Some(expression) = &return_statement.value else {
                            return false;
                        };

                        is_variable_named(expression, "$this", context)
                    }
                    _ => false,
                },
                None => true,
            }
        }
        _ => false,
    }
}

fn is_accessing_property_of_this(expression: &Expression, context: &LintContext<'_>) -> bool {
    let Expression::Access(access) = expression else {
        return false;
    };

    let Access::Property(property_access) = access else {
        return false;
    };

    is_variable_named(&property_access.object, "$this", context)
}

fn is_variable_named(expression: &Expression, name: &str, context: &LintContext<'_>) -> bool {
    let Expression::Variable(variable) = expression else {
        return false;
    };

    let Variable::Direct(direct_variable) = variable else {
        return false;
    };

    context.interner.lookup(&direct_variable.name) == name
}
