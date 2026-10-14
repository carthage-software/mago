use std::sync::Arc;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;

pub fn get_throw_type_atomics(atomic_type: &TAtomic) -> Vec<TAtomic> {
    let TAtomic::GenericParameter(parameter) = atomic_type else {
        return vec![atomic_type.clone()];
    };

    if parameter.constraint.types.len() <= 1 {
        return vec![atomic_type.clone()];
    }

    parameter
        .constraint
        .types
        .iter()
        .map(|constraint_atomic| {
            let mut constraint = TUnion::from_atomic(constraint_atomic.clone());
            constraint.flags = parameter.constraint.flags;

            TAtomic::GenericParameter(TGenericParameter {
                parameter_name: parameter.parameter_name,
                constraint: Arc::new(constraint),
                defining_entity: parameter.defining_entity,
                intersection_types: parameter.intersection_types.clone(),
            })
        })
        .collect()
}

pub fn get_throw_type_object_names(codebase: &CodebaseMetadata, atomic_type: &TAtomic) -> Vec<Word> {
    let mut object_names = Vec::new();
    append_throw_type_object_names(codebase, atomic_type, &mut object_names);
    object_names
}

fn append_throw_type_object_names(codebase: &CodebaseMetadata, atomic_type: &TAtomic, object_names: &mut Vec<Word>) {
    match atomic_type {
        TAtomic::GenericParameter(parameter) => {
            for constraint_type in parameter.constraint.types.iter() {
                let mut intersection_types = vec![constraint_type.clone()];
                if let Some(additional_types) = &parameter.intersection_types {
                    intersection_types.extend(additional_types.iter().cloned());
                }

                append_throw_type_intersection_names(codebase, &intersection_types, object_names);
            }
        }
        _ => {
            let Some(intersection_types) = atomic_type.get_intersection_types().filter(|types| !types.is_empty())
            else {
                append_throw_type_intersection_names(codebase, std::slice::from_ref(atomic_type), object_names);
                return;
            };

            let mut base_type = atomic_type.clone();
            if let Some(base_intersection_types) = base_type.get_intersection_types_mut() {
                base_intersection_types.clear();
            }

            let mut intersection = vec![base_type];
            intersection.extend(intersection_types.iter().cloned());
            append_throw_type_intersection_names(codebase, &intersection, object_names);
        }
    }
}

fn append_throw_type_intersection_names(
    codebase: &CodebaseMetadata,
    intersection_types: &[TAtomic],
    object_names: &mut Vec<Word>,
) {
    let mut throwable_names = Vec::new();
    for intersection_type in intersection_types {
        if matches!(intersection_type, TAtomic::GenericParameter(_))
            || intersection_type.get_intersection_types().is_some_and(|types| !types.is_empty())
        {
            let mut nested_names = Vec::new();
            append_throw_type_object_names(codebase, intersection_type, &mut nested_names);
            for nested_name in nested_names {
                if !throwable_names.contains(&nested_name) {
                    throwable_names.push(nested_name);
                }
            }
        } else {
            for throwable_name in intersection_type.get_all_object_names() {
                if codebase.is_instance_of(throwable_name.as_bytes(), b"Throwable")
                    && !throwable_names.contains(&throwable_name)
                {
                    throwable_names.push(throwable_name);
                }
            }
        }
    }

    let throwable_classes = throwable_names
        .iter()
        .copied()
        .filter(|name| {
            codebase.is_instance_of(name.as_bytes(), b"Throwable")
                && codebase.get_class_like(name.as_bytes()).is_some_and(|metadata| metadata.kind.is_class())
        })
        .collect::<Vec<_>>();

    if throwable_classes.is_empty() {
        object_names.extend(throwable_names);
        return;
    }

    for throwable_class in &throwable_classes {
        let is_superclass_of_another_bound = throwable_classes.iter().any(|other_class| {
            other_class != throwable_class
                && codebase.is_instance_of(other_class.as_bytes(), throwable_class.as_bytes())
        });

        if !is_superclass_of_another_bound && !object_names.contains(throwable_class) {
            object_names.push(*throwable_class);
        }
    }
}
