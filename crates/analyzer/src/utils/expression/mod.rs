use std::rc::Rc;

use foldhash::HashMap;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::union::TUnion;
use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::cst::Access;
use mago_syntax::cst::ArrayAccess;
use mago_syntax::cst::Call;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Literal;
use mago_syntax::cst::MethodCall;
use mago_syntax::cst::NullSafeMethodCall;
use mago_syntax::cst::StaticMethodCall;
use mago_syntax::cst::UnaryPostfixOperator;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_syntax::cst::Variable;
use mago_word::Word;
use mago_word::concat_word;
use mago_word::word;

use crate::utils::misc::unwrap_expression;

pub mod array;
pub mod variable;

/// Checks if an expression has observable side effects.
///
/// An expression is considered to have observable side effects if it performs operations that can modify state
/// or have effects beyond just computing a value.
pub(crate) const fn expression_has_observable_side_effect(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Parenthesized(p) => expression_has_observable_side_effect(p.expression),
        Expression::Assignment(_) | Expression::Throw(_) | Expression::Yield(_) | Expression::Clone(_) => true,
        Expression::UnaryPrefix(u) => {
            matches!(
                u.operator,
                UnaryPrefixOperator::PreIncrement(_)
                    | UnaryPrefixOperator::PreDecrement(_)
                    | UnaryPrefixOperator::Reference(_)
            ) || expression_has_observable_side_effect(u.operand)
        }
        Expression::UnaryPostfix(u) => {
            matches!(u.operator, UnaryPostfixOperator::PostIncrement(_) | UnaryPostfixOperator::PostDecrement(_))
        }
        Expression::Binary(b) => {
            expression_has_observable_side_effect(b.lhs) || expression_has_observable_side_effect(b.rhs)
        }
        Expression::Conditional(c) => {
            matches!(c.then, Some(then_expr) if expression_has_observable_side_effect(then_expr))
                || expression_has_observable_side_effect(c.r#else)
                || expression_has_observable_side_effect(c.condition)
        }
        _ => false,
    }
}

/// Checks if an expression is using nullsafe access anywhere in its chain.
///
/// Given an expression, this function recursively checks if any part of the expression
/// involves nullsafe access (i.e., `?->`). It handles various expression types including
/// array accesses, method calls, property accesses, and parenthesized expressions.
#[inline]
pub(crate) const fn expression_is_nullsafe(expr: &'_ Expression<'_>) -> bool {
    match expr {
        Expression::ArrayAccess(array_access) => expression_is_nullsafe(array_access.array),
        Expression::Call(Call::NullSafeMethod(_)) => true,
        Expression::Call(Call::Method(method_call)) => expression_is_nullsafe(method_call.object),
        Expression::Call(Call::StaticMethod(static_method_call)) => expression_is_nullsafe(static_method_call.class),
        Expression::Access(Access::NullSafeProperty(_)) => true,
        Expression::Access(Access::Property(property_access)) => expression_is_nullsafe(property_access.object),
        Expression::Access(Access::StaticProperty(static_property_access)) => {
            expression_is_nullsafe(static_property_access.class)
        }
        // PHP is weird..
        // - https://github.com/php/php-src/issues/20684
        // - https://github.com/php/php-src/pull/20685
        Expression::Parenthesized(parenthesized) => expression_is_nullsafe(parenthesized.expression),
        _ => false,
    }
}

pub const fn expression_has_logic(expression: &Expression<'_>) -> bool {
    match unwrap_expression(expression) {
        Expression::Binary(binary) => {
            binary.operator.is_instanceof()
                || binary.operator.is_equality()
                || binary.operator.is_logical()
                || binary.operator.is_null_coalesce()
        }
        _ => false,
    }
}

pub fn get_variable_id<'arena>(variable: &Variable<'arena>) -> Option<&'arena [u8]> {
    match variable {
        Variable::Direct(direct_variable) => Some(direct_variable.name),
        _ => None,
    }
}

pub fn get_member_selector_id<'ast, 'arena>(
    selector: &'ast ClassLikeMemberSelector<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    match selector {
        ClassLikeMemberSelector::Identifier(local_identifier) => Some(word(local_identifier.value)),
        ClassLikeMemberSelector::Variable(variable) => get_variable_id(variable).map(word),
        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
            let expr_id = get_expression_id(
                class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                codebase,
            )?;
            Some(concat_word!(b"{", expr_id.as_bytes(), b"}"))
        }
        ClassLikeMemberSelector::Missing(_) => None,
    }
}

pub fn get_constant_selector_id<'ast, 'arena>(
    selector: &'ast ClassLikeConstantSelector<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    match selector {
        ClassLikeConstantSelector::Identifier(local_identifier) => Some(word(local_identifier.value)),
        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
            let expr_id = get_expression_id(
                class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                codebase,
            )?;
            Some(concat_word!(b"{", expr_id.as_bytes(), b"}"))
        }
        ClassLikeConstantSelector::Missing(_) => None,
    }
}

/** Gets the identifier for a simple variable */
pub fn get_expression_id<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    get_extended_expression_id(expression, this_class_name, resolved_names, codebase, false)
}

fn get_extended_expression_id<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
    solve_identifiers: bool,
) -> Option<Word> {
    let expression = unwrap_expression(expression);

    if let Expression::Assignment(assignment) = expression {
        return get_expression_id(assignment.lhs, this_class_name, resolved_names, codebase);
    }

    Some(match expression {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), operand }) => {
            return get_expression_id(operand, this_class_name, resolved_names, codebase);
        }
        Expression::Variable(variable) => word(get_variable_id(variable)?),
        Expression::Access(access) => match access {
            Access::Property(property_access) => get_property_access_expression_id(
                property_access.object,
                &property_access.property,
                false,
                this_class_name,
                resolved_names,
                codebase,
            )?,
            Access::NullSafeProperty(null_safe_property_access) => get_property_access_expression_id(
                null_safe_property_access.object,
                &null_safe_property_access.property,
                true,
                this_class_name,
                resolved_names,
                codebase,
            )?,
            Access::StaticProperty(static_property_access) => get_static_property_access_expression_id(
                static_property_access.class,
                &static_property_access.property,
                this_class_name,
                resolved_names,
                codebase,
            )?,
            Access::ClassConstant(class_constant_access) => {
                let class = get_extended_expression_id(
                    class_constant_access.class,
                    this_class_name,
                    resolved_names,
                    codebase,
                    true,
                )?;

                let constant = get_constant_selector_id(
                    &class_constant_access.constant,
                    this_class_name,
                    resolved_names,
                    codebase,
                )?;

                concat_word!(class.as_bytes(), b"::", constant.as_bytes())
            }
        },
        Expression::ArrayAccess(array_access) => {
            get_array_access_id(array_access, this_class_name, resolved_names, codebase)?
        }
        Expression::Self_(_) => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                word(b"self")
            }
        }
        Expression::Parent(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                word(b"parent")
            }
        }
        Expression::Static(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                word(b"static")
            }
        }
        Expression::Identifier(identifier) if solve_identifiers => {
            let identifier_id = resolved_names.get(&identifier);

            word(identifier_id)
        }
        _ => return None,
    })
}

pub fn get_property_access_expression_id<'ast, 'arena>(
    object_expression: &'ast Expression<'arena>,
    selector: &ClassLikeMemberSelector,
    is_null_safe: bool,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    let object = get_expression_id(object_expression, this_class_name, resolved_names, codebase)?;
    let property = get_member_selector_id(selector, this_class_name, resolved_names, codebase)?;

    Some(if is_null_safe {
        concat_word!(object.as_bytes(), b"?->", property.as_bytes())
    } else {
        concat_word!(object.as_bytes(), b"->", property.as_bytes())
    })
}

pub fn get_static_property_access_expression_id<'ast, 'arena>(
    class_expr: &'ast Expression<'arena>,
    property: &'ast Variable<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    let class = get_extended_expression_id(class_expr, this_class_name, resolved_names, codebase, true)?;
    let property = get_variable_id(property)?;

    Some(concat_word!(class.as_bytes(), b"::", property))
}

#[inline]
pub fn get_array_access_id<'ast, 'arena>(
    array_access: &'ast ArrayAccess<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    let array = get_expression_id(array_access.array, this_class_name, resolved_names, codebase)?;
    let index = get_index_id(array_access.index, this_class_name, resolved_names, codebase)?;

    Some(concat_word!(array.as_bytes(), b"[", index.as_bytes(), b"]"))
}

pub fn get_root_expression_id(expression: &Expression<'_>) -> Option<Word> {
    let expression = unwrap_expression(expression);

    match expression {
        Expression::Variable(Variable::Direct(variable)) => Some(word(variable.name)),
        Expression::ArrayAccess(array_access) => get_root_expression_id(array_access.array),
        Expression::Access(access) => match access {
            Access::Property(access) => get_root_expression_id(access.object),
            Access::NullSafeProperty(access) => get_root_expression_id(access.object),
            Access::ClassConstant(access) => get_root_expression_id(access.class),
            Access::StaticProperty(access) => get_root_expression_id(access.class),
        },
        _ => None,
    }
}

pub fn get_index_id<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    this_class_name: Option<Word>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Word> {
    Some(match expression {
        Expression::Literal(Literal::String(literal_string)) => word(literal_string.raw),
        Expression::Literal(Literal::Integer(literal_integer)) => word(literal_integer.raw),
        Expression::UnaryPostfix(unary_postfix) => {
            return get_index_id(unary_postfix.operand, this_class_name, resolved_names, codebase);
        }
        _ => return get_expression_id(expression, this_class_name, resolved_names, codebase),
    })
}

pub fn get_function_like_id_from_call<'ast, 'arena>(
    call: &'ast Call<'arena>,
    resolved_names: &'ast ResolvedNames<'arena>,
    expression_types: &HashMap<(u32, u32), Rc<TUnion>>,
) -> Option<FunctionLikeIdentifier> {
    get_static_functionlike_id_from_call(call, resolved_names)
        .or_else(|| get_method_id_from_call(call, expression_types))
}

pub fn get_static_functionlike_id_from_call<'ast, 'arena>(
    call: &'ast Call<'arena>,
    resolved_names: &'ast ResolvedNames<'arena>,
) -> Option<FunctionLikeIdentifier> {
    match call {
        Call::Function(FunctionCall { function: Expression::Identifier(identifier), .. }) => {
            let function_name = resolved_names.get(&identifier);

            Some(FunctionLikeIdentifier::Function(word(function_name)))
        }
        Call::StaticMethod(StaticMethodCall {
            class: Expression::Identifier(class_identifier),
            method: ClassLikeMemberSelector::Identifier(method),
            ..
        }) => {
            let class_name = resolved_names.get(&class_identifier);

            let class_id = word(class_name);
            let method_id = word(method.value);

            Some(FunctionLikeIdentifier::Method(class_id, method_id))
        }
        _ => None,
    }
}

pub fn get_method_id_from_call(
    call: &Call<'_>,
    expression_types: &HashMap<(u32, u32), Rc<TUnion>>,
) -> Option<FunctionLikeIdentifier> {
    match call {
        Call::Method(MethodCall { object, method: ClassLikeMemberSelector::Identifier(method), .. })
        | Call::NullSafeMethod(NullSafeMethodCall {
            object,
            method: ClassLikeMemberSelector::Identifier(method),
            ..
        }) => {
            let TAtomic::Object(TObject::Named(named_object)) =
                expression_types.get(&(object.span().start.offset, object.span().end.offset))?.types.first()?
            else {
                return None;
            };

            let method_id = word(method.value);

            Some(FunctionLikeIdentifier::Method(named_object.get_name(), method_id))
        }
        _ => None,
    }
}

/// Checks if a given string (`derived_path`) represents a property access (`->`, `::`)
/// or array element access (`[]`) that originates from a `base_path` string.
///
/// Note: This function only checks the *first character* of the access operator.
/// For `::`, it checks for the first colon. For `->`, it checks for the hyphen.
///
///
/// * `true` if `derived_path` is an access path derived from `base_path`.
/// * `false` otherwise (e.g., if `derived_path` doesn't start with `base_path`,
///   or if it does but is not followed by a recognized access operator character,
///   or if `derived_path` is identical to `base_path`).
#[inline]
pub fn is_derived_access_path(derived_path: Word, base_path: Word) -> bool {
    let derived = derived_path.as_bytes();
    let base = base_path.as_bytes();
    derived.starts_with(base) && derived.get(base.len()).is_some_and(|&b| b == b':' || b == b'-' || b == b'[')
}
