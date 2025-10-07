use mago_atom::Atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;

#[inline]
pub fn scan_function_like_parameter<'ctx, 'arena>(
    parameter: &'arena FunctionLikeParameter<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'ctx, 'arena>,
    scope: &NamespaceScope,
) -> FunctionLikeParameterMetadata {
    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if parameter.ellipsis.is_some() {
        flags |= MetadataFlags::VARIADIC;
    }

    if parameter.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    if parameter.is_promoted_property() {
        flags |= MetadataFlags::PROMOTED_PROPERTY;
    }

    let mut metadata = FunctionLikeParameterMetadata::new(
        VariableIdentifier(atom(parameter.variable.name)),
        parameter.span(),
        parameter.variable.span,
        flags,
    )
    .with_attributes(scan_attribute_lists(&parameter.attribute_lists, context))
    .with_type_signature(parameter.hint.as_ref().map(|hint| get_type_metadata_from_hint(hint, classname, context)));

    if let Some(default_value) = &parameter.default_value {
        metadata.flags |= MetadataFlags::HAS_DEFAULT;
        metadata.default_type = infer(context, scope, &default_value.value).map(|u| {
            let mut type_metadata = TypeMetadata::new(u, default_value.span());
            type_metadata.inferred = true;
            type_metadata
        });
    }

    metadata
}
