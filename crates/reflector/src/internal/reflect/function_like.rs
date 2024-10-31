use fennec_ast::*;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::function_like::parameter::*;
use fennec_reflection::function_like::r#return::*;
use fennec_reflection::function_like::*;
use fennec_reflection::identifier::FunctionLikeName;
use fennec_reflection::identifier::Name;
use fennec_span::*;

use crate::internal::context::Context;
use crate::internal::reflect::attribute::reflect_attributes;
use crate::internal::reflect::r#type::maybe_reflect_hint;
use crate::internal::reflect::r#type::reflect_hint;

pub fn reflect_function<'i, 'ast>(
    function: &'ast Function,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeReflection {
    FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&function.attributes, context),
        visibility_reflection: None,
        name: FunctionLikeName::Function(Name::new(context.semantics.names.get(&function.name), function.name.span)),
        parameter_reflections: reflect_function_like_parameter_list(&function.parameters, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(&function.return_type_hint, context, scope),
        returns_by_reference: function.ampersand.is_some(),
        has_yield: fennec_ast_utils::block_has_yield(&function.body),
        has_throws: fennec_ast_utils::block_has_throws(&function.body),
        is_anonymous: false,
        is_static: true,
        is_final: true,
        is_abstract: false,
        is_overriding: false,
        span: function.span(),
        is_populated: false,
    }
}

pub fn reflect_closure<'i, 'ast>(
    closure: &'ast Closure,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeReflection {
    FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&closure.attributes, context),
        visibility_reflection: None,
        name: FunctionLikeName::Closure(closure.span()),
        parameter_reflections: reflect_function_like_parameter_list(&closure.parameters, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(&closure.return_type_hint, context, scope),
        returns_by_reference: closure.ampersand.is_some(),
        has_yield: fennec_ast_utils::block_has_yield(&closure.body),
        has_throws: fennec_ast_utils::block_has_throws(&closure.body),
        is_anonymous: true,
        is_static: closure.r#static.is_some(),
        is_final: true,
        is_abstract: false,
        is_overriding: false,
        span: closure.span(),
        is_populated: false,
    }
}

pub fn reflect_arrow_function<'i, 'ast>(
    arrow_function: &'ast ArrowFunction,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeReflection {
    FunctionLikeReflection {
        attribute_reflections: reflect_attributes(&arrow_function.attributes, context),
        visibility_reflection: None,
        name: FunctionLikeName::ArrowFunction(arrow_function.span()),
        parameter_reflections: reflect_function_like_parameter_list(&arrow_function.parameters, context, scope),
        return_type_reflection: reflect_function_like_return_type_hint(
            &arrow_function.return_type_hint,
            context,
            scope,
        ),
        returns_by_reference: arrow_function.ampersand.is_some(),
        has_yield: fennec_ast_utils::expression_has_yield(&arrow_function.expression),
        has_throws: fennec_ast_utils::expression_has_throws(&arrow_function.expression),
        is_anonymous: true,
        is_static: arrow_function.r#static.is_some(),
        is_final: true,
        is_abstract: false,
        is_overriding: false,
        span: arrow_function.span(),
        is_populated: false,
    }
}

pub fn reflect_function_like_parameter_list<'i, 'ast>(
    parameter_list: &'ast FunctionLikeParameterList,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> Vec<FunctionLikeParameterReflection> {
    let mut parameters = vec![];
    for parameter in parameter_list.parameters.iter() {
        parameters.push(reflect_function_like_parameter(parameter, context, scope));
    }

    parameters
}

pub fn reflect_function_like_parameter<'i, 'ast>(
    parameter: &'ast FunctionLikeParameter,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> FunctionLikeParameterReflection {
    FunctionLikeParameterReflection {
        attribute_reflections: reflect_attributes(&parameter.attributes, context),
        type_reflection: maybe_reflect_hint(&parameter.hint, context, scope),
        name: parameter.variable.name,
        is_variadic: parameter.ellipsis.is_some(),
        is_passed_by_reference: parameter.ampersand.is_some(),
        is_promoted_property: parameter.is_promoted_property(),
        default: parameter.default_value.as_ref().map(|d| FunctionLikeParameterDefaultValueReflection {
            type_reflection: fennec_inference::infere(&context.interner, &context.semantics, &d.value),
            span: d.span(),
        }),
    }
}

pub fn reflect_function_like_return_type_hint<'i, 'ast>(
    return_type_hint: &'ast Option<FunctionLikeReturnTypeHint>,
    context: &'ast mut Context<'i>,
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
