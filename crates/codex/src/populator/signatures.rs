use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::union::populate_union_type;
use mago_word::Word;

/// Populates metadata for a single function or method.
///
/// Resolves types for return types, parameters, template parameters, etc.
/// Adds symbol references based on attributes and types used.
pub fn populate_function_like_metadata(
    metadata: &mut FunctionLikeMetadata,
    codebase_symbols: &Symbols,
    reference_source: &ReferenceSource,
    symbol_references: &mut SymbolReferences,
    force_type_population: bool,
) {
    // Early exit if already populated and not forced
    if metadata.flags.is_populated() && !force_type_population {
        return;
    }

    for attribute_metadata in metadata.get_attributes() {
        add_attribute_reference(symbol_references, reference_source, attribute_metadata.name, Some(true));
    }

    for return_type in
        metadata.return_type_declaration_metadata.as_mut().into_iter().chain(metadata.return_type_metadata.as_mut())
    {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    for parameter_metadata in metadata.get_parameters_mut() {
        let parameter_types = parameter_metadata
            .type_declaration_metadata
            .as_mut()
            .into_iter()
            .chain(parameter_metadata.type_metadata.as_mut())
            .chain(parameter_metadata.out_type.as_mut())
            .chain(parameter_metadata.closure_this_type.as_mut())
            .chain(parameter_metadata.default_type.as_mut());

        for type_metadata in parameter_types {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        for attribute_metadata in &parameter_metadata.attributes {
            add_attribute_reference(symbol_references, reference_source, attribute_metadata.name, None);
        }
    }

    let context_templates = metadata
        .type_resolution_context
        .as_mut()
        .into_iter()
        .flat_map(|context| context.get_template_definitions_mut().values_mut().flatten());

    for template in metadata.template_types.values_mut().chain(context_templates) {
        if force_type_population || template.constraint.needs_population() {
            populate_union_type(
                &mut template.constraint,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(default) = template.default.as_mut()
            && (force_type_population || default.needs_population())
        {
            populate_union_type(
                default,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }
    }

    if let Some(method_metadata) = metadata.method_metadata.as_mut() {
        for where_constraint in method_metadata.where_constraints.values_mut() {
            populate_union_type(
                &mut where_constraint.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }
    }

    for thrown_type in &mut metadata.thrown_types {
        populate_union_type(
            &mut thrown_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    let all_assertions = metadata
        .assertions
        .values_mut()
        .chain(metadata.if_true_assertions.values_mut())
        .chain(metadata.if_false_assertions.values_mut());

    for assertions in all_assertions {
        for assertion in assertions {
            if let Some(assertion_type) = assertion.get_type_mut() {
                populate_atomic_type(
                    assertion_type,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    metadata.flags |= MetadataFlags::POPULATED;
}

fn add_attribute_reference(
    symbol_references: &mut SymbolReferences,
    reference_source: &ReferenceSource,
    name: Word,
    in_signature_override: Option<bool>,
) {
    match reference_source {
        ReferenceSource::Symbol(in_signature, a) => {
            symbol_references.add_symbol_reference_to_symbol(*a, name, in_signature_override.unwrap_or(*in_signature));
        }
        ReferenceSource::ClassLikeMember(in_signature, a, b) => {
            symbol_references.add_class_member_reference_to_symbol(
                (*a, *b),
                name,
                in_signature_override.unwrap_or(*in_signature),
            );
        }
        ReferenceSource::File(in_signature, file) => {
            symbol_references.add_file_reference_to_class_member(
                *file,
                (name, mago_word::empty_word()),
                in_signature_override.unwrap_or(*in_signature),
            );
        }
    }
}
