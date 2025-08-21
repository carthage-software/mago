use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

#[inline]
#[must_use]
pub fn is_user_input<'ast, 'arena>(expression: &'ast Expression<'arena>) -> bool {
    match expression {
        Expression::Parenthesized(parenthesized) => is_user_input(parenthesized.expression),
        Expression::Assignment(assignment) => is_user_input(assignment.rhs),
        Expression::Conditional(conditional) => match conditional.then.as_ref() {
            Some(then) => is_user_input(then) || is_user_input(conditional.r#else),
            None => is_user_input(conditional.condition) || is_user_input(conditional.r#else),
        },
        Expression::ArrayAccess(array_access) => is_user_input(array_access.array),
        Expression::Match(match_expr) => match_expr.arms.iter().any(|arm| {
            is_user_input(match arm {
                MatchArm::Expression(e) => e.expression,
                MatchArm::Default(d) => d.expression,
            })
        }),
        Expression::Binary(binary) if binary.operator.is_concatenation() || binary.operator.is_null_coalesce() => {
            is_user_input(binary.lhs) || is_user_input(binary.rhs)
        }
        Expression::Variable(variable) => is_variable_user_input(variable),
        _ => false,
    }
}

#[inline]
#[must_use]
pub fn is_variable_user_input<'ast, 'arena>(variable: &'ast Variable<'arena>) -> bool {
    match variable {
        Variable::Direct(direct_variable) => {
            let name = direct_variable.name;

            name.eq_ignore_ascii_case("$_GET")
                || name.eq_ignore_ascii_case("$_POST")
                || name.eq_ignore_ascii_case("$_REQUEST")
                || name.eq_ignore_ascii_case("$_COOKIE")
                || name.eq_ignore_ascii_case("$_FILES")
                || name.eq_ignore_ascii_case("$_SERVER")
                || name.eq_ignore_ascii_case("$_SESSION")
        }
        Variable::Indirect(indirect_variable) => is_user_input(indirect_variable.expression),
        Variable::Nested(nested_variable) => is_variable_user_input(nested_variable.variable),
    }
}

#[inline]
#[must_use]
pub fn get_password<'ast, 'arena>(expr: &'ast Expression<'arena>) -> Option<Span> {
    match expr {
        Expression::Parenthesized(parenthesized) => get_password(parenthesized.expression),
        Expression::Literal(Literal::String(literal_string)) if is_password_literal(literal_string) => {
            Some(literal_string.span())
        }
        Expression::Assignment(assignment) => get_password(assignment.lhs).or_else(|| get_password(assignment.rhs)),
        Expression::ArrayAccess(array_access) => get_password(array_access.index),
        Expression::Variable(variable) => get_password_from_variable(variable),
        Expression::Identifier(identifier) if is_password_identifier(identifier) => Some(identifier.span()),
        Expression::Call(call) => match call {
            Call::Method(method_call) => get_password_from_selector(&method_call.method),
            Call::StaticMethod(static_method_call) => get_password_from_selector(&static_method_call.method),
            _ => None,
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => get_password_from_selector(&property_access.property),
            Access::NullSafeProperty(null_safe_property_access) => {
                get_password_from_selector(&null_safe_property_access.property)
            }
            Access::StaticProperty(static_property_access) => {
                get_password_from_variable(&static_property_access.property)
            }
            Access::ClassConstant(class_constant_access) => {
                get_password_from_constant_selector(&class_constant_access.constant)
            }
        },
        _ => None,
    }
}

#[inline]
#[must_use]
pub fn get_password_from_selector<'ast, 'arena>(selector: &'ast ClassLikeMemberSelector<'arena>) -> Option<Span> {
    match selector {
        ClassLikeMemberSelector::Identifier(local_identifier) => {
            if is_password(local_identifier.value) {
                return Some(local_identifier.span());
            }

            None
        }
        ClassLikeMemberSelector::Variable(variable) => get_password_from_variable(variable),
        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
            get_password(class_like_member_expression_selector.expression)
        }
    }
}

#[inline]
#[must_use]
pub fn get_password_from_constant_selector<'ast, 'arena>(
    selector: &'ast ClassLikeConstantSelector<'arena>,
) -> Option<Span> {
    match selector {
        ClassLikeConstantSelector::Identifier(local_identifier) => {
            if is_password(local_identifier.value) {
                return Some(local_identifier.span());
            }

            None
        }
        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
            get_password(class_like_member_expression_selector.expression)
        }
    }
}

#[inline]
#[must_use]
pub fn get_password_from_variable<'ast, 'arena>(variable: &'ast Variable<'arena>) -> Option<Span> {
    match variable {
        Variable::Direct(direct_variable) => {
            if is_password(&direct_variable.name[1..]) {
                return Some(direct_variable.span());
            }

            None
        }
        Variable::Indirect(indirect_variable) => get_password(indirect_variable.expression),
        Variable::Nested(nested_variable) => get_password_from_variable(nested_variable.variable),
    }
}

#[inline]
#[must_use]
pub fn is_password_identifier<'arena>(identifier: &'arena Identifier<'arena>) -> bool {
    let Identifier::Local(local_identifier) = identifier else {
        return false;
    };

    is_password(local_identifier.value)
}

#[inline]
#[must_use]
pub fn is_password_literal<'arena>(literal: &'arena LiteralString<'arena>) -> bool {
    is_password(&literal.raw[1..literal.raw.len() - 1])
}

#[inline]
#[must_use]
pub fn is_password(mut str: &str) -> bool {
    if str.starts_with("$") {
        str = &str[1..];
    }

    if str.starts_with("get") {
        str = &str[3..];

        if str.starts_with("_") {
            str = &str[1..];
        }
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
