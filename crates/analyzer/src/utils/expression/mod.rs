use std::rc::Rc;

use foldhash::HashMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::union::TUnion;

use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::ArrayAccess;
use mago_syntax::ast::Call;
use mago_syntax::ast::ClassLikeConstantSelector;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Literal;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::NullSafeMethodCall;
use mago_syntax::ast::StaticMethodCall;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::Variable;

use crate::utils::misc::unwrap_expression;

pub mod array;
pub mod variable;

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

pub fn get_variable_id<'arena>(variable: &Variable<'arena>) -> Option<&'arena str> {
    match variable {
        Variable::Direct(direct_variable) => Some(direct_variable.name),
        _ => None,
    }
}

pub fn get_member_selector_id<'ast, 'arena>(
    selector: &'ast ClassLikeMemberSelector<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    match selector {
        ClassLikeMemberSelector::Identifier(local_identifier) => Some(atom(local_identifier.value)),
        ClassLikeMemberSelector::Variable(variable) => get_variable_id(variable).map(atom),
        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
            let expr_id = get_expression_id(
                class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                codebase,
            )?;
            Some(concat_atom!("{", expr_id.as_str(), "}"))
        }
        ClassLikeMemberSelector::Missing(_) => None,
    }
}

pub fn get_constant_selector_id<'ast, 'arena>(
    selector: &'ast ClassLikeConstantSelector<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    match selector {
        ClassLikeConstantSelector::Identifier(local_identifier) => Some(atom(local_identifier.value)),
        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => {
            let expr_id = get_expression_id(
                class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                codebase,
            )?;
            Some(concat_atom!("{", expr_id.as_str(), "}"))
        }
        ClassLikeConstantSelector::Missing(_) => None,
    }
}

/** Gets the identifier for a simple variable */
pub fn get_expression_id<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    get_extended_expression_id(expression, this_class_name, resolved_names, codebase, false)
}

fn get_extended_expression_id<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
    solve_identifiers: bool,
) -> Option<Atom> {
    let expression = unwrap_expression(expression);

    if let Expression::Assignment(assignment) = expression {
        return get_expression_id(assignment.lhs, this_class_name, resolved_names, codebase);
    }

    Some(match expression {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), operand }) => {
            return get_expression_id(operand, this_class_name, resolved_names, codebase);
        }
        Expression::Variable(variable) => atom(get_variable_id(variable)?),
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

                concat_atom!(class.as_str(), "::", constant.as_str())
            }
        },
        Expression::ArrayAccess(array_access) => {
            get_array_access_id(array_access, this_class_name, resolved_names, codebase)?
        }
        Expression::Self_(_) => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                atom("self")
            }
        }
        Expression::Parent(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                atom("parent")
            }
        }
        Expression::Static(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                class_name
            } else {
                atom("static")
            }
        }
        Expression::Identifier(identifier) if solve_identifiers => {
            let identifier_id = resolved_names.get(&identifier);

            atom(identifier_id)
        }
        _ => return None,
    })
}

pub fn get_property_access_expression_id<'ast, 'arena>(
    object_expression: &'ast Expression<'arena>,
    selector: &ClassLikeMemberSelector,
    is_null_safe: bool,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    let object = get_expression_id(object_expression, this_class_name, resolved_names, codebase)?;
    let property = get_member_selector_id(selector, this_class_name, resolved_names, codebase)?;

    Some(if is_null_safe {
        concat_atom!(object.as_str(), "?->", property.as_str())
    } else {
        concat_atom!(object.as_str(), "->", property.as_str())
    })
}

pub fn get_static_property_access_expression_id<'ast, 'arena>(
    class_expr: &'ast Expression<'arena>,
    property: &'ast Variable<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    let class = get_extended_expression_id(class_expr, this_class_name, resolved_names, codebase, true)?;
    let property = get_variable_id(property)?;

    Some(concat_atom!(class.as_str(), "::", property))
}

#[inline]
pub fn get_array_access_id<'ast, 'arena>(
    array_access: &'ast ArrayAccess<'arena>,
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    let array = get_expression_id(array_access.array, this_class_name, resolved_names, codebase)?;
    let index = get_index_id(array_access.index, this_class_name, resolved_names, codebase)?;

    Some(concat_atom!(array.as_str(), "[", index.as_str(), "]"))
}

pub fn get_root_expression_id(expression: &Expression<'_>) -> Option<Atom> {
    let expression = unwrap_expression(expression);

    match expression {
        Expression::Variable(Variable::Direct(variable)) => Some(atom(variable.name)),
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
    this_class_name: Option<Atom>,
    resolved_names: &'ast ResolvedNames<'arena>,
    codebase: Option<&CodebaseMetadata>,
) -> Option<Atom> {
    Some(match expression {
        Expression::Literal(Literal::String(literal_string)) => atom(literal_string.raw),
        Expression::Literal(Literal::Integer(literal_integer)) => atom(literal_integer.raw),
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

            Some(FunctionLikeIdentifier::Function(atom(function_name)))
        }
        Call::StaticMethod(StaticMethodCall {
            class: Expression::Identifier(class_identifier),
            method: ClassLikeMemberSelector::Identifier(method),
            ..
        }) => {
            let class_name = resolved_names.get(&class_identifier);

            let class_id = atom(class_name);
            let method_id = atom(method.value);

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

            let method_id = atom(method.value);

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
pub fn is_derived_access_path(derived_path: Atom, base_path: Atom) -> bool {
    derived_path.as_str().starts_with(base_path.as_str())
        && derived_path.as_str().chars().nth(base_path.len()).is_some_and(|c| c == ':' || c == '-' || c == '[')
}
