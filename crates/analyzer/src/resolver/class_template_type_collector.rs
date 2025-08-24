use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::ascii_lowercase_atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;

pub(crate) fn collect(
    codebase: &CodebaseMetadata,
    class_metadata: &ClassLikeMetadata,
    static_class_metadata: &ClassLikeMetadata,
    object_type: Option<&TObject>,
) -> Option<IndexMap<Atom, HashMap<GenericParent, TUnion>, RandomState>> {
    if class_metadata.template_types.is_empty() {
        return None;
    }

    let mut class_template_parameters: IndexMap<Atom, HashMap<GenericParent, TUnion>, RandomState> =
        IndexMap::default();

    if let Some(TObject::Named(TNamedObject { type_parameters: Some(parameters), .. })) = &object_type {
        if class_metadata.name == static_class_metadata.name && !static_class_metadata.template_types.is_empty() {
            for (i, (template_type_name, _)) in class_metadata.template_types.iter().enumerate() {
                if let Some(type_parameter) = parameters.get(i) {
                    class_template_parameters
                        .entry(*template_type_name)
                        .or_default()
                        .insert(GenericParent::ClassLike(class_metadata.name), type_parameter.clone());
                }
            }
        }

        for (template_name, _) in &class_metadata.template_types {
            if class_template_parameters.contains_key(template_name) {
                continue;
            }

            if class_metadata.name != static_class_metadata.name
                && let Some(input_type_extends) = static_class_metadata
                    .template_extended_parameters
                    .get(&class_metadata.name)
                    .and_then(|m| m.get(template_name))
            {
                let output_type_extends =
                    resolve_template_parameter(codebase, input_type_extends, static_class_metadata, parameters);

                class_template_parameters
                    .entry(*template_name)
                    .or_default()
                    .insert(GenericParent::ClassLike(class_metadata.name), output_type_extends.unwrap_or(get_mixed()));
            }

            class_template_parameters
                .entry(*template_name)
                .or_default()
                .entry(GenericParent::ClassLike(class_metadata.name))
                .or_insert(get_mixed());
        }
    }

    for (template_name, type_map) in &class_metadata.template_types {
        for (template_classname, type_) in type_map {
            if class_metadata.name != static_class_metadata.name
                && let Some(extended_type) = static_class_metadata
                    .template_extended_parameters
                    .get(&class_metadata.name)
                    .and_then(|m| m.get(template_name))
            {
                class_template_parameters
                    .entry(*template_name)
                    .or_default()
                    .entry(GenericParent::ClassLike(class_metadata.name))
                    .or_insert(TUnion::from_vec(expand_type(
                        extended_type,
                        &static_class_metadata.template_extended_parameters,
                        &static_class_metadata.name,
                        &static_class_metadata.template_types,
                    )));
            }

            let self_call =
                if let Some(TObject::Named(TNamedObject { name: self_class_name, is_this: true, .. })) = object_type {
                    template_classname == &GenericParent::ClassLike(ascii_lowercase_atom(self_class_name))
                } else {
                    false
                };

            if !self_call {
                class_template_parameters
                    .entry(*template_name)
                    .or_default()
                    .entry(GenericParent::ClassLike(class_metadata.name))
                    .or_insert(type_.clone());
            }
        }
    }

    Some(class_template_parameters)
}

pub(crate) fn resolve_template_parameter(
    codebase: &CodebaseMetadata,
    input_type_extends: &TUnion,
    static_class_storage: &ClassLikeMetadata,
    type_params: &Vec<TUnion>,
) -> Option<TUnion> {
    let mut output_type_extends = None;

    for type_extends_atomic in input_type_extends.types.as_ref() {
        if let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity: GenericParent::ClassLike(defining_entity),
            ..
        }) = &type_extends_atomic
        {
            if let Some(entry) =
                static_class_storage.template_types.iter().enumerate().find(|(_, (k, _))| k == parameter_name)
            {
                let mapped_offset = entry.0;

                if let Some(type_param) = type_params.get(mapped_offset) {
                    output_type_extends =
                        Some(add_optional_union_type(type_param.clone(), output_type_extends.as_ref(), codebase));
                }
            } else if let Some(input_type_extends) = static_class_storage
                .template_extended_parameters
                .get(defining_entity)
                .unwrap_or(&IndexMap::default())
                .get(parameter_name)
            {
                let nested_output_type =
                    resolve_template_parameter(codebase, input_type_extends, static_class_storage, type_params);

                if let Some(nested_output_type) = nested_output_type {
                    output_type_extends =
                        Some(add_optional_union_type(nested_output_type, output_type_extends.as_ref(), codebase));
                }
            }
        } else {
            output_type_extends = Some(add_optional_union_type(
                wrap_atomic(type_extends_atomic.clone()),
                output_type_extends.as_ref(),
                codebase,
            ));
        }
    }

    output_type_extends
}

fn expand_type(
    input_type_extends: &TUnion,
    template_extended_parameters: &AtomMap<IndexMap<Atom, TUnion, RandomState>>,
    static_class_name: &Atom,
    static_class_template_types: &[(Atom, Vec<(GenericParent, TUnion)>)],
) -> Vec<TAtomic> {
    let mut output_type_extends = Vec::new();

    for extends_atomic in input_type_extends.types.as_ref() {
        let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity: GenericParent::ClassLike(defining_entity),
            ..
        }) = extends_atomic
        else {
            output_type_extends.push(extends_atomic.clone());
            continue;
        };

        if static_class_name == defining_entity && static_class_template_types.iter().any(|(k, _)| k == parameter_name)
        {
            output_type_extends.push(extends_atomic.clone());
            continue;
        }

        let Some(extended_type) =
            template_extended_parameters.get(defining_entity).and_then(|map| map.get(parameter_name))
        else {
            output_type_extends.push(extends_atomic.clone());

            continue;
        };

        output_type_extends.extend(expand_type(
            extended_type,
            template_extended_parameters,
            static_class_name,
            static_class_template_types,
        ));
    }

    output_type_extends
}
