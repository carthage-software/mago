use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

/// Returns the minimal span for a class-like node (just keyword + name).
///
/// For named classes/traits/enums/interfaces, this returns the span from the keyword to the name.
/// For anonymous classes, this returns just the `class` keyword span.
#[inline]
pub fn get_class_like_header_span(node: Node<'_, '_>) -> Span {
    match node {
        Node::Class(class) => class.class.span().join(class.name.span()),
        Node::Trait(r#trait) => r#trait.r#trait.span().join(r#trait.name.span()),
        Node::Enum(r#enum) => r#enum.r#enum.span().join(r#enum.name.span()),
        Node::Interface(interface) => interface.interface.span().join(interface.name.span()),
        Node::AnonymousClass(class) => class.class.span(),
        _ => node.span(),
    }
}

#[inline]
pub fn get_single_return_statement<'ast, 'arena>(block: &'ast Block<'arena>) -> Option<&'ast Return<'arena>> {
    let statements = block.statements.as_slice();

    if statements.len() != 1 {
        return None;
    }

    let Statement::Return(return_stmt) = &statements[0] else {
        return None;
    };

    Some(return_stmt)
}

pub fn is_method_setter_or_getter(method: &Method<'_>) -> bool {
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

            if !is_accessing_property_of_this(expression) {
                return false;
            }

            statements_len == 1
        }
        Statement::Expression(expression_statement) if method.parameter_list.parameters.len() == 1 => {
            let Expression::Assignment(assignment) = expression_statement.expression else {
                return false;
            };

            if !is_accessing_property_of_this(assignment.lhs) {
                return false;
            }

            match block.statements.last() {
                Some(statement) => match statement {
                    Statement::Return(return_statement) => {
                        let Some(expression) = &return_statement.value else {
                            return false;
                        };

                        is_variable_named(expression, "$this")
                    }
                    _ => false,
                },
                None => true,
            }
        }
        _ => false,
    }
}

fn is_accessing_property_of_this(expression: &Expression<'_>) -> bool {
    let Expression::Access(access) = expression else {
        return false;
    };

    let Access::Property(property_access) = access else {
        return false;
    };

    is_variable_named(property_access.object, "$this")
}

fn is_variable_named(expression: &Expression<'_>, name: &str) -> bool {
    let Expression::Variable(variable) = expression else {
        return false;
    };

    let Variable::Direct(direct_variable) = variable else {
        return false;
    };

    direct_variable.name == name
}
