use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;

#[inline]
pub fn scan_function_like_parameter<'ast>(
    parameter: &'ast FunctionLikeParameter,
    classname: Option<&StringIdentifier>,
    context: &'ast mut Context<'_>,
) -> FunctionLikeParameterMetadata {
    let mut metadata = FunctionLikeParameterMetadata::new(
        VariableIdentifier(parameter.variable.name),
        parameter.span(),
        parameter.variable.span,
    )
    .with_attributes(scan_attribute_lists(&parameter.attribute_lists, context))
    .with_is_variadic(parameter.ellipsis.is_some())
    .with_is_by_reference(parameter.ampersand.is_some())
    .with_is_promoted_property(parameter.is_promoted_property())
    .with_type_signature(parameter.hint.as_ref().map(|hint| get_type_metadata_from_hint(hint, classname, context)));

    if let Some(default_value) = &parameter.default_value {
        metadata = metadata.with_has_default(true).with_default_type(
            infer(context.interner, context.resolved_names, &default_value.value).map(|u| {
                let mut type_metadata = TypeMetadata::new(u, default_value.span());
                type_metadata.inferred = true;
                type_metadata
            }),
        );
    }

    metadata
}
