use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::FunctionLikeParameter;

use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::inference::infer_with_constants;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::version_claim::TypeOverride;
use crate::scanner::version_claim::evaluate_version_attributes;

#[inline]
pub fn scan_function_like_parameter<'arena>(
    parameter: &'arena FunctionLikeParameter<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> Option<FunctionLikeParameterMetadata> {
    scan_function_like_parameter_with_constants(parameter, classname, context, scope, None)
}

#[inline]
pub fn scan_function_like_parameter_with_constants<'arena>(
    parameter: &'arena FunctionLikeParameter<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
    constants: Option<&AtomMap<ConstantMetadata>>,
) -> Option<FunctionLikeParameterMetadata> {
    let verdict = evaluate_version_attributes(&parameter.attribute_lists, context, context.php_version);
    if !verdict.is_available(context.php_version) {
        return None;
    }

    let mut flags = MetadataFlags::origin_flags(context.file.file_type);

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
    .with_attributes(scan_attribute_lists(&parameter.attribute_lists, context));

    metadata.set_type_declaration_metadata(
        parameter.hint.as_ref().map(|hint| get_type_metadata_from_hint(hint, classname, context)),
    );

    if let Some(default_value) = &parameter.default_value {
        metadata.flags |= MetadataFlags::HAS_DEFAULT;
        metadata.default_type =
            infer_with_constants(context, scope, default_value.value, classname, constants).map(|u| {
                let mut type_metadata = TypeMetadata::new(u, default_value.span());
                type_metadata.inferred = true;
                type_metadata
            });
    }

    if let Some(optional) = verdict.optional {
        metadata.flags.set(MetadataFlags::HAS_DEFAULT, optional);
    }

    if matches!(verdict.type_override, Some(TypeOverride::Untyped)) {
        metadata.type_declaration_metadata = None;
        metadata.type_metadata = None;
    }
    // TypedWith* type-string parsing isn't wired to the type resolver yet; the
    // claim is still recognized, just left as a no-op until that lands.

    Some(metadata)
}
