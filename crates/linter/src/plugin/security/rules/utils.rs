use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::context::LintContext;

#[inline]
#[must_use]
pub fn is_user_input(context: &LintContext, expression: &Expression) -> bool {
    match expression {
        Expression::Parenthesized(parenthesized) => is_user_input(context, &parenthesized.expression),
        Expression::Assignment(assignment) => is_user_input(context, &assignment.rhs),
        Expression::Conditional(conditional) => match conditional.then.as_ref() {
            Some(then) => is_user_input(context, then) || is_user_input(context, &conditional.r#else),
            None => is_user_input(context, &conditional.condition) || is_user_input(context, &conditional.r#else),
        },
        Expression::ArrayAccess(array_access) => is_user_input(context, &array_access.array),
        Expression::Match(match_expr) => match_expr.arms.iter().any(|arm| {
            is_user_input(
                context,
                match arm {
                    MatchArm::Expression(e) => e.expression.as_ref(),
                    MatchArm::Default(d) => d.expression.as_ref(),
                },
            )
        }),
        Expression::Binary(binary) if binary.operator.is_concatenation() || binary.operator.is_null_coalesce() => {
            is_user_input(context, &binary.lhs) || is_user_input(context, &binary.rhs)
        }
        Expression::Variable(variable) => is_variable_user_input(context, variable),
        _ => false,
    }
}

#[inline]
#[must_use]
pub fn is_variable_user_input(context: &LintContext, variable: &Variable) -> bool {
    match variable {
        Variable::Direct(direct_variable) => {
            let name = context.interner.lookup(&direct_variable.name);

            name.eq_ignore_ascii_case("$_GET")
                || name.eq_ignore_ascii_case("$_POST")
                || name.eq_ignore_ascii_case("$_REQUEST")
                || name.eq_ignore_ascii_case("$_COOKIE")
                || name.eq_ignore_ascii_case("$_FILES")
                || name.eq_ignore_ascii_case("$_SERVER")
                || name.eq_ignore_ascii_case("$_SESSION")
        }
        Variable::Indirect(indirect_variable) => is_user_input(context, &indirect_variable.expression),
        Variable::Nested(nested_variable) => is_variable_user_input(context, &nested_variable.variable),
    }
}

#[inline]
#[must_use]
pub fn get_password(context: &LintContext, expr: &Expression) -> Option<Span> {
    match expr {
        Expression::Parenthesized(parenthesized) => get_password(context, &parenthesized.expression),
        Expression::Literal(Literal::String(literal_string)) if is_password_literal(context, literal_string) => {
            Some(literal_string.span())
        }
        Expression::Assignment(assignment) => {
            get_password(context, &assignment.lhs).or_else(|| get_password(context, &assignment.rhs))
        }
        Expression::ArrayAccess(array_access) => get_password(context, &array_access.index),
        Expression::Variable(variable) => get_password_from_variable(context, variable),
        Expression::Identifier(identifier) if is_password_identifier(context, identifier) => Some(identifier.span()),
        Expression::Call(call) => match call {
            Call::Method(method_call) => get_password_from_selector(context, &method_call.method),
            Call::StaticMethod(static_method_call) => get_password_from_selector(context, &static_method_call.method),
            _ => None,
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => get_password_from_selector(context, &property_access.property),
            Access::NullSafeProperty(null_safe_property_access) => {
                get_password_from_selector(context, &null_safe_property_access.property)
            }
            Access::StaticProperty(static_property_access) => {
                get_password_from_variable(context, &static_property_access.property)
            }
            Access::ClassConstant(class_constant_access) => {
                get_password_from_constant_selector(context, &class_constant_access.constant)
            }
        },
        _ => None,
    }
}

#[inline]
#[must_use]
pub fn get_password_from_selector(context: &LintContext, selector: &ClassLikeMemberSelector) -> Option<Span> {
    match selector {
        ClassLikeMemberSelector::Identifier(local_identifier) => {
            let name = context.interner.lookup(&local_identifier.value);

            if is_password(name) {
                return Some(local_identifier.span());
            }

            None
        }
        ClassLikeMemberSelector::Variable(variable) => get_password_from_variable(context, variable),
        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
            get_password(context, &class_like_member_expression_selector.expression)
        }
    }
}

#[inline]
#[must_use]
pub fn get_password_from_constant_selector(
    context: &LintContext,
    selector: &ClassLikeConstantSelector,
) -> Option<Span> {
    match selector {
        ClassLikeConstantSelector::Identifier(local_identifier) => {
            let name = context.interner.lookup(&local_identifier.value);

            if is_password(name) {
                return Some(local_identifier.span());
            }

            None
        }
        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
            get_password(context, &class_like_member_expression_selector.expression)
        }
    }
}

#[inline]
#[must_use]
pub fn get_password_from_variable(context: &LintContext, variable: &Variable) -> Option<Span> {
    match variable {
        Variable::Direct(direct_variable) => {
            let name = context.interner.lookup(&direct_variable.name);

            if is_password(&name[1..]) {
                return Some(direct_variable.span());
            }

            None
        }
        Variable::Indirect(indirect_variable) => get_password(context, &indirect_variable.expression),
        Variable::Nested(nested_variable) => get_password_from_variable(context, &nested_variable.variable),
    }
}

#[inline]
#[must_use]
pub fn is_password_identifier(context: &LintContext, identifier: &Identifier) -> bool {
    let Identifier::Local(local_identifier) = identifier else {
        return false;
    };

    is_password_string(context, &local_identifier.value)
}

#[inline]
#[must_use]
pub fn is_password_literal(context: &LintContext, literal: &LiteralString) -> bool {
    let value = context.interner.lookup(&literal.value);

    is_password(&value[1..value.len() - 1])
}

#[inline]
#[must_use]
pub fn is_password_string(context: &LintContext, identifier: &StringIdentifier) -> bool {
    is_password(context.interner.lookup(identifier))
}

#[inline]
#[must_use]
pub fn is_password(mut str: &str) -> bool {
    if str.starts_with("$") {
        str = &str[1..];
    }

    if str.starts_with("get") {
        str = &str[3..];
    }

    let lower = str.to_lowercase();

    if lower.ends_with("password")
        || lower.ends_with("token")
        || lower.ends_with("secret")
        || lower.ends_with("apiKey")
        || lower.ends_with("api_key")
    {
        return true;
    }

    false
}
