use std::rc::Rc;

use ahash::HashMap;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::utils::misc::unwrap_expression;

pub mod array;
pub mod variable;

pub const fn expression_has_logic(expression: &Expression) -> bool {
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

pub fn get_variable_id(variable: &Variable, interner: &ThreadedInterner) -> Option<String> {
    match variable {
        Variable::Direct(direct_variable) => {
            let var_id = interner.lookup(&direct_variable.name).to_string();

            Some(var_id)
        }
        _ => None,
    }
}

pub fn get_member_selector_id(
    selector: &ClassLikeMemberSelector,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    match selector {
        ClassLikeMemberSelector::Identifier(local_identifier) => {
            Some(interner.lookup(&local_identifier.value).to_string())
        }
        ClassLikeMemberSelector::Variable(variable) => get_variable_id(variable, interner),
        ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => Some(format!(
            "{{{}}}",
            get_expression_id(
                &class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                interner,
                codebase,
            )?
        )),
    }
}

pub fn get_constant_selector_id(
    selector: &ClassLikeConstantSelector,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    match selector {
        ClassLikeConstantSelector::Identifier(local_identifier) => {
            Some(interner.lookup(&local_identifier.value).to_string())
        }
        ClassLikeConstantSelector::Expression(class_like_member_expression_selector) => Some(format!(
            "{{{}}}",
            get_expression_id(
                &class_like_member_expression_selector.expression,
                this_class_name,
                resolved_names,
                interner,
                codebase,
            )?
        )),
    }
}

/** Gets the identifier for a simple variable */
pub fn get_expression_id(
    expression: &Expression,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    get_extended_expression_id(expression, this_class_name, resolved_names, interner, codebase, false)
}

fn get_extended_expression_id(
    expression: &Expression,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
    solve_identifiers: bool,
) -> Option<String> {
    let expression = unwrap_expression(expression);

    if let Expression::Assignment(assignment) = expression {
        return get_expression_id(&assignment.lhs, this_class_name, resolved_names, interner, codebase);
    };

    Some(match expression {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), operand }) => {
            return get_expression_id(operand, this_class_name, resolved_names, interner, codebase);
        }
        Expression::Variable(variable) => get_variable_id(variable, interner)?,
        Expression::Access(access) => match access {
            Access::Property(property_access) => get_property_access_expression_id(
                &property_access.object,
                &property_access.property,
                false,
                this_class_name,
                resolved_names,
                interner,
                codebase,
            )?,
            Access::NullSafeProperty(null_safe_property_access) => get_property_access_expression_id(
                &null_safe_property_access.object,
                &null_safe_property_access.property,
                true,
                this_class_name,
                resolved_names,
                interner,
                codebase,
            )?,
            Access::StaticProperty(static_property_access) => get_static_property_access_expression_id(
                &static_property_access.class,
                &static_property_access.property,
                this_class_name,
                resolved_names,
                interner,
                codebase,
            )?,
            Access::ClassConstant(class_constant_access) => {
                let class = get_extended_expression_id(
                    &class_constant_access.class,
                    this_class_name,
                    resolved_names,
                    interner,
                    codebase,
                    true,
                )?;

                let constant = get_constant_selector_id(
                    &class_constant_access.constant,
                    this_class_name,
                    resolved_names,
                    interner,
                    codebase,
                )?;

                format!("{class}::{constant}")
            }
        },
        Expression::ArrayAccess(array_access) => {
            get_array_access_id(array_access, this_class_name, resolved_names, interner, codebase)?
        }
        Expression::Self_(_) => {
            if let Some(class_name) = this_class_name {
                interner.lookup(class_name).to_string()
            } else {
                "self".to_string()
            }
        }
        Expression::Parent(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                interner.lookup(class_name).to_string()
            } else {
                "parent".to_string()
            }
        }
        Expression::Static(_) if solve_identifiers => {
            if let Some(class_name) = this_class_name {
                interner.lookup(class_name).to_string()
            } else {
                "static".to_string()
            }
        }
        Expression::Identifier(identifier) if solve_identifiers => {
            let identifier_id = resolved_names.get(&identifier);

            interner.lookup(identifier_id).to_string()
        }
        _ => return None,
    })
}

pub fn get_property_access_expression_id(
    object_expression: &Expression,
    selector: &ClassLikeMemberSelector,
    is_null_safe: bool,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    let object = get_expression_id(object_expression, this_class_name, resolved_names, interner, codebase)?;
    let property = get_member_selector_id(selector, this_class_name, resolved_names, interner, codebase)?;

    Some(if is_null_safe { format!("{object}?->{property}") } else { format!("{object}->{property}") })
}

pub fn get_static_property_access_expression_id(
    class_expr: &Expression,
    property: &Variable,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    let class = get_extended_expression_id(class_expr, this_class_name, resolved_names, interner, codebase, true)?;
    let property = get_variable_id(property, interner)?;

    Some(format!("{class}::{property}"))
}

#[inline]
pub fn get_array_access_id(
    array_access: &ArrayAccess,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    let array = get_expression_id(&array_access.array, this_class_name, resolved_names, interner, codebase)?;
    let index = get_index_id(&array_access.index, this_class_name, resolved_names, interner, codebase)?;

    Some(format!("{array}[{index}]"))
}

pub fn get_root_expression_id(expression: &Expression, interner: &ThreadedInterner) -> Option<String> {
    let expression = unwrap_expression(expression);

    match expression {
        Expression::Variable(Variable::Direct(variable)) => Some(interner.lookup(&variable.name).to_string()),
        Expression::ArrayAccess(array_access) => get_root_expression_id(&array_access.array, interner),
        Expression::Access(access) => match access {
            Access::Property(access) => get_root_expression_id(&access.object, interner),
            Access::NullSafeProperty(access) => get_root_expression_id(&access.object, interner),
            _ => None,
        },
        _ => None,
    }
}

pub fn get_index_id(
    expression: &Expression,
    this_class_name: Option<&StringIdentifier>,
    resolved_names: &ResolvedNames,
    interner: &ThreadedInterner,
    codebase: Option<&CodebaseMetadata>,
) -> Option<String> {
    Some(match expression {
        Expression::Literal(Literal::String(literal_string)) => interner.lookup(&literal_string.raw).to_string(),
        Expression::Literal(Literal::Integer(literal_integer)) => interner.lookup(&literal_integer.raw).to_string(),
        _ => return get_expression_id(expression, this_class_name, resolved_names, interner, codebase),
    })
}

pub fn get_function_like_id_from_call(
    call: &Call,
    resolved_names: &ResolvedNames,
    expression_types: &HashMap<(usize, usize), Rc<TUnion>>,
) -> Option<FunctionLikeIdentifier> {
    get_static_functionlike_id_from_call(call, resolved_names)
        .or_else(|| get_method_id_from_call(call, expression_types))
}

pub fn get_static_functionlike_id_from_call(
    call: &Call,
    resolved_names: &ResolvedNames,
) -> Option<FunctionLikeIdentifier> {
    match call {
        Call::Function(FunctionCall { function, .. }) => match function.as_ref() {
            Expression::Identifier(identifier) => {
                let function_id = resolved_names.get(&identifier);

                Some(FunctionLikeIdentifier::Function(*function_id))
            }
            _ => None,
        },
        Call::StaticMethod(StaticMethodCall { class, method: ClassLikeMemberSelector::Identifier(method), .. }) => {
            let Expression::Identifier(class_identifier) = class.as_ref() else {
                return None;
            };

            let class_id = resolved_names.get(&class_identifier);

            Some(FunctionLikeIdentifier::Method(*class_id, method.value))
        }
        _ => None,
    }
}

pub fn get_method_id_from_call(
    call: &Call,
    expression_types: &HashMap<(usize, usize), Rc<TUnion>>,
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

            Some(FunctionLikeIdentifier::Method(named_object.get_name(), method.value))
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
pub fn is_derived_access_path(derived_path: &str, base_path: &str) -> bool {
    derived_path.starts_with(base_path)
        && derived_path.chars().nth(base_path.len()).is_some_and(|c| c == ':' || c == '-' || c == '[')
}
