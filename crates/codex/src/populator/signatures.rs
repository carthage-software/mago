use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::union::populate_union_type;

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
        match reference_source {
            ReferenceSource::Symbol(_, a) => {
                symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.name, true);
            }
            ReferenceSource::ClassLikeMember(_, a, b) => {
                symbol_references.add_class_member_reference_to_symbol((*a, *b), attribute_metadata.name, true);
            }
        }
    }

    if let Some(return_type) = metadata.return_type_declaration_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    if let Some(return_type) = metadata.return_type_metadata.as_mut() {
        populate_union_type(
            &mut return_type.type_union,
            codebase_symbols,
            Some(reference_source),
            symbol_references,
            force_type_population,
        );
    }

    for parameter_metadata in metadata.get_parameters_mut() {
        if let Some(type_metadata) = parameter_metadata.type_declaration_metadata.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.type_metadata.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.out_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        if let Some(type_metadata) = parameter_metadata.default_type.as_mut() {
            populate_union_type(
                &mut type_metadata.type_union,
                codebase_symbols,
                Some(reference_source),
                symbol_references,
                force_type_population,
            );
        }

        for attribute_metadata in &parameter_metadata.attributes {
            match reference_source {
                ReferenceSource::Symbol(in_signature, a) => {
                    symbol_references.add_symbol_reference_to_symbol(*a, attribute_metadata.name, *in_signature);
                }
                ReferenceSource::ClassLikeMember(in_signature, a, b) => symbol_references
                    .add_class_member_reference_to_symbol((*a, *b), attribute_metadata.name, *in_signature),
            }
        }
    }

    for (_, type_parameter_map) in &mut metadata.template_types {
        for (_, type_parameter) in type_parameter_map {
            if force_type_population || type_parameter.needs_population() {
                populate_union_type(
                    type_parameter,
                    codebase_symbols,
                    Some(reference_source),
                    symbol_references,
                    force_type_population,
                );
            }
        }
    }

    if let Some(type_resolution_context) = metadata.type_resolution_context.as_mut() {
        for (_, type_parameter_map) in type_resolution_context.get_template_definitions_mut() {
            for (_, type_parameter) in type_parameter_map {
                if force_type_population || type_parameter.needs_population() {
                    populate_union_type(
                        type_parameter,
                        codebase_symbols,
                        Some(reference_source),
                        symbol_references,
                        force_type_population,
                    );
                }
            }
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

    for assertions in metadata.assertions.values_mut() {
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

    for assertions in metadata.if_true_assertions.values_mut() {
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

    for assertions in metadata.if_false_assertions.values_mut() {
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
