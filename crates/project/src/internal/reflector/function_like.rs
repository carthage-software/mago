use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::function_like::parameter::*;
use mago_reflection::function_like::r#return::*;
use mago_reflection::function_like::*;
use mago_reflection::identifier::FunctionLikeName;
use mago_reflection::identifier::Name;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;
use crate::internal::reflector::attribute::reflect_attributes;
use crate::internal::reflector::r#type::maybe_reflect_hint;
use crate::internal::reflector::r#type::reflect_hint;

use super::should_reflect_element;

#[inline]
pub fn reflect_function<'ast>(
    function: &'ast Function,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> Option<FunctionLikeReflection> {
    if !should_reflect_element(context, &function.attribute_lists) {
        return None;
    }

    let name = Name::new(*context.names.get(&function.name), function.name.span);

    Some(FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&function.attribute_lists, context),
        visibility_reflection: None,
        name: FunctionLikeName::Function(name),
        // TODO: parse docblock to get the template list
        templates: vec![],
        parameters: reflect_function_like_parameter_list(&function.parameter_list, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(&function.return_type_hint, context, scope),
        returns_by_reference: function.ampersand.is_some(),
        has_yield: mago_syntax::utils::block_has_yield(&function.body),
        has_throws: mago_syntax::utils::block_has_throws(&function.body),
        is_anonymous: false,
        is_static: true,
        is_final: true,
        // TODO: parse docblock to determine if pure
        is_pure: false,
        is_abstract: false,
        is_overriding: false,
        span: function.span(),
        is_populated: false,
        issues: Default::default(),
    })
}

#[inline]
pub fn reflect_closure<'ast>(
    closure: &'ast Closure,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeReflection {
    FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&closure.attribute_lists, context),
        visibility_reflection: None,
        name: FunctionLikeName::Closure(closure.span()),
        // TODO: parse docblock to get the template list
        templates: vec![],
        parameters: reflect_function_like_parameter_list(&closure.parameter_list, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(&closure.return_type_hint, context, scope),
        returns_by_reference: closure.ampersand.is_some(),
        has_yield: mago_syntax::utils::block_has_yield(&closure.body),
        has_throws: mago_syntax::utils::block_has_throws(&closure.body),
        is_anonymous: true,
        is_static: closure.r#static.is_some(),
        is_final: true,
        // TODO: parse docblock to determine if pure
        is_pure: false,
        is_abstract: false,
        is_overriding: false,
        span: closure.span(),
        is_populated: false,
        issues: Default::default(),
    }
}

#[inline]
pub fn reflect_arrow_function<'ast>(
    arrow_function: &'ast ArrowFunction,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeReflection {
    FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&arrow_function.attribute_lists, context),
        visibility_reflection: None,
        name: FunctionLikeName::ArrowFunction(arrow_function.span()),
        // TODO: parse docblock to get the template list
        templates: vec![],
        parameters: reflect_function_like_parameter_list(&arrow_function.parameter_list, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(
            &arrow_function.return_type_hint,
            context,
            scope,
        ),
        returns_by_reference: arrow_function.ampersand.is_some(),
        has_yield: mago_syntax::utils::expression_has_yield(&arrow_function.expression),
        has_throws: mago_syntax::utils::expression_has_throws(&arrow_function.expression),
        is_anonymous: true,
        is_static: arrow_function.r#static.is_some(),
        is_final: true,
        // TODO: parse docblock to determine if pure
        is_pure: false,
        is_abstract: false,
        is_overriding: false,
        span: arrow_function.span(),
        is_populated: false,
        issues: Default::default(),
    }
}

#[inline]
pub fn reflect_function_like_parameter_list<'ast>(
    parameter_list: &'ast FunctionLikeParameterList,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> Vec<FunctionLikeParameterReflection> {
    let mut parameters = vec![];
    for parameter in parameter_list.parameters.iter() {
        if let Some(parameter) = reflect_function_like_parameter(parameter, context, scope) {
            parameters.push(parameter);
        }
    }

    parameters
}

#[inline]
pub fn reflect_function_like_parameter<'ast>(
    parameter: &'ast FunctionLikeParameter,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> Option<FunctionLikeParameterReflection> {
    if !should_reflect_element(context, &parameter.attribute_lists) {
        return None;
    }

    Some(FunctionLikeParameterReflection {
        attribute_reflections: reflect_attributes(&parameter.attribute_lists, context),
        type_reflection: maybe_reflect_hint(&parameter.hint, context, scope),
        name: parameter.variable.name,
        is_variadic: parameter.ellipsis.is_some(),
        is_passed_by_reference: parameter.ampersand.is_some(),
        is_promoted_property: parameter.is_promoted_property(),
        default: parameter.default_value.as_ref().map(|d| FunctionLikeParameterDefaultValueReflection {
            type_reflection: mago_typing::infere(context.interner, context.source, context.names, &d.value),
            span: d.span(),
        }),
        span: parameter.span(),
    })
}

#[inline]
pub fn reflect_function_like_return_type_hint<'ast>(
    return_type_hint: &'ast Option<FunctionLikeReturnTypeHint>,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> Option<FunctionLikeReturnTypeReflection> {
    let Some(return_type_hint) = return_type_hint else {
        return None;
    };

    Some(FunctionLikeReturnTypeReflection {
        type_reflection: reflect_hint(&return_type_hint.hint, context, scope),
        span: return_type_hint.span(),
    })
}
