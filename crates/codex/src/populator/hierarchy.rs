use mago_atom::Atom;

use crate::metadata::class_like::ClassLikeMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::union::populate_union_type;

/// Populates types for properties, constants, enum cases, and type aliases within a class-like.
pub fn populate_class_like_types(
    name: Atom,
    metadata: &mut ClassLikeMetadata,
    codebase_symbols: &Symbols,
    symbol_references: &mut SymbolReferences,
    force_repopulation: bool,
) {
    let class_like_reference_source = ReferenceSource::Symbol(true, name);

    for (property_name, property_metadata) in &mut metadata.properties {
        let property_reference_source = ReferenceSource::ClassLikeMember(true, name, *property_name);

        if let Some(signature) = property_metadata.type_declaration_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(signature) = property_metadata.type_metadata.as_mut() {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(default) = property_metadata.default_type_metadata.as_mut() {
            populate_union_type(
                &mut default.type_union,
                codebase_symbols,
                Some(&property_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        for hook in property_metadata.hooks.values_mut() {
            if let Some(hook_return_type) = &mut hook.return_type_metadata {
                populate_union_type(
                    &mut hook_return_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            let Some(hook_parameter) = hook.parameter.as_mut() else {
                continue;
            };

            if let Some(hook_parameter_type_declaration) = &mut hook_parameter.type_declaration_metadata {
                populate_union_type(
                    &mut hook_parameter_type_declaration.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_type) = &mut hook_parameter.type_metadata {
                populate_union_type(
                    &mut hook_parameter_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_out_type) = &mut hook_parameter.out_type {
                populate_union_type(
                    &mut hook_parameter_out_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }

            if let Some(hook_parameter_default_type) = &mut hook_parameter.default_type {
                populate_union_type(
                    &mut hook_parameter_default_type.type_union,
                    codebase_symbols,
                    Some(&property_reference_source),
                    symbol_references,
                    force_repopulation,
                );
            }
        }
    }

    for v in metadata.template_types.iter_mut().flat_map(|m| m.1.iter_mut()).map(|template| &mut template.1) {
        if v.needs_population() || force_repopulation {
            populate_union_type(
                v,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for template in &mut metadata.template_extended_offsets.values_mut().flatten() {
        if template.needs_population() || force_repopulation {
            populate_union_type(
                template,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for p in metadata.template_extended_parameters.values_mut().flat_map(|m| m.values_mut()) {
        if p.needs_population() || force_repopulation {
            populate_union_type(
                p,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for type_alias in metadata.type_aliases.values_mut() {
        populate_union_type(
            &mut type_alias.type_union,
            codebase_symbols,
            Some(&class_like_reference_source),
            symbol_references,
            force_repopulation,
        );
    }

    for mixin_type in &mut metadata.mixins {
        if mixin_type.needs_population() || force_repopulation {
            populate_union_type(
                mixin_type,
                codebase_symbols,
                Some(&class_like_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for (constant_name, constant) in &mut metadata.constants {
        let constant_reference_source = ReferenceSource::ClassLikeMember(true, name, *constant_name);

        for attribute_metadata in &constant.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (name, *constant_name),
                attribute_metadata.name,
                true,
            );
        }

        if let Some(signature) = &mut constant.type_metadata {
            populate_union_type(
                &mut signature.type_union,
                codebase_symbols,
                Some(&constant_reference_source),
                symbol_references,
                force_repopulation,
            );
        }

        if let Some(inferred) = &mut constant.inferred_type {
            populate_atomic_type(
                inferred,
                codebase_symbols,
                Some(&constant_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    for (enum_case_name, enum_case) in &mut metadata.enum_cases {
        let enum_case_reference_source = ReferenceSource::ClassLikeMember(true, name, *enum_case_name);

        for attribute_metadata in &enum_case.attributes {
            symbol_references.add_class_member_reference_to_symbol(
                (name, *enum_case_name),
                attribute_metadata.name,
                true,
            );
        }

        if let Some(value_type) = &mut enum_case.value_type {
            populate_atomic_type(
                value_type,
                codebase_symbols,
                Some(&enum_case_reference_source),
                symbol_references,
                force_repopulation,
            );
        }
    }

    if let Some(enum_type) = &mut metadata.enum_type {
        populate_atomic_type(
            enum_type,
            codebase_symbols,
            Some(&ReferenceSource::Symbol(true, name)),
            symbol_references,
            force_repopulation,
        );
    }
}
