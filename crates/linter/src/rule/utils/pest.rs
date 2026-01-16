use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Access;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Call;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Literal;
use mago_syntax::ast::MethodCall;

use crate::context::LintContext;
use crate::rule::utils::call::function_call_matches;

/// Checks if the given function call is `expect($arg)` and returns the argument expression.
pub fn get_expect_argument<'arena>(
    ctx: &LintContext<'_, 'arena>,
    call: &'arena FunctionCall<'arena>,
) -> Option<&'arena Expression<'arena>> {
    if !function_call_matches(ctx, call, "expect") {
        return None;
    }

    // Get the first argument (the value being tested)
    let first_arg = call.argument_list.arguments.first()?;
    Some(first_arg.value())
}

/// Checks if the method call chain has a `->not->` modifier.
///
/// This traverses up the method call chain looking for a property access named "not".
#[inline]
pub fn has_not_modifier(method_call: &MethodCall<'_>) -> bool {
    check_for_not_in_chain(method_call.object)
}

fn check_for_not_in_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                if let ClassLikeMemberSelector::Identifier(ident) = &property_access.property
                    && ident.value == "not"
                {
                    return true;
                }

                check_for_not_in_chain(property_access.object)
            }
            Access::NullSafeProperty(property_access) => {
                if let ClassLikeMemberSelector::Identifier(ident) = &property_access.property
                    && ident.value == "not"
                {
                    return true;
                }

                check_for_not_in_chain(property_access.object)
            }
            _ => false,
        },
        Expression::Call(call) => match call {
            Call::Method(inner_method) => check_for_not_in_chain(inner_method.object),
            Call::NullSafeMethod(inner_method) => check_for_not_in_chain(inner_method.object),
            _ => false,
        },
        _ => false,
    }
}

/// Finds the expect() function call at the root of a method call chain.
pub fn find_expect_call<'arena>(method_call: &'arena MethodCall<'arena>) -> Option<&'arena FunctionCall<'arena>> {
    find_function_call_in_chain(method_call.object)
}

fn find_function_call_in_chain<'arena>(expr: &'arena Expression<'arena>) -> Option<&'arena FunctionCall<'arena>> {
    match expr {
        Expression::Call(call) => match call {
            Call::Function(func_call) => Some(func_call),
            Call::Method(inner_method) => find_function_call_in_chain(inner_method.object),
            Call::NullSafeMethod(inner_method) => find_function_call_in_chain(inner_method.object),
            _ => None,
        },
        Expression::Access(access) => match access {
            Access::Property(property_access) => find_function_call_in_chain(property_access.object),
            Access::NullSafeProperty(property_access) => find_function_call_in_chain(property_access.object),
            _ => None,
        },
        _ => None,
    }
}

/// Checks if an expression is an empty array literal `[]`.
pub fn is_empty_array(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Array(array) => array.elements.is_empty(),
        Expression::LegacyArray(array) => array.elements.is_empty(),
        _ => false,
    }
}

/// Checks if an expression is an empty string literal `''` or `""`.
pub fn is_empty_string(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Literal(Literal::String(string_literal)) => {
            // The value is the unquoted content, so check if it's empty
            string_literal.value.is_some_and(|v| v.is_empty())
        }
        _ => false,
    }
}

/// Gets the first argument of a method call if it exists.
pub fn get_first_argument<'a>(method_call: &'a MethodCall<'a>) -> Option<&'a Argument<'a>> {
    method_call.argument_list.arguments.first()
}

/// Finds the span covering `->not->method()` in a method call chain.
///
/// For `expect($x)->not->toBeFalse()`, this returns the span from `->not` to `)`.
/// This is useful for replacing double-negative patterns.
pub fn find_not_to_method_span(method_call: &MethodCall<'_>) -> Option<Span> {
    find_not_arrow_span(method_call.object).map(|arrow_span| arrow_span.join(method_call.span()))
}

fn find_not_arrow_span(expr: &Expression<'_>) -> Option<Span> {
    match expr {
        Expression::Access(access) => match access {
            Access::Property(property_access) => {
                if let ClassLikeMemberSelector::Identifier(ident) = &property_access.property
                    && ident.value == "not"
                {
                    // Return the span of the arrow before `not`
                    return Some(property_access.arrow);
                }

                find_not_arrow_span(property_access.object)
            }
            Access::NullSafeProperty(property_access) => {
                if let ClassLikeMemberSelector::Identifier(ident) = &property_access.property
                    && ident.value == "not"
                {
                    return Some(property_access.question_mark_arrow);
                }

                find_not_arrow_span(property_access.object)
            }
            _ => None,
        },
        Expression::Call(call) => match call {
            Call::Method(inner_method) => find_not_arrow_span(inner_method.object),
            Call::NullSafeMethod(inner_method) => find_not_arrow_span(inner_method.object),
            _ => None,
        },
        _ => None,
    }
}
