use std::borrow::Cow;

use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;

use crate::get_class_like;
use crate::get_closure;
use crate::get_declaring_method;
use crate::get_function;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::combiner;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum StaticClassType {
    #[default]
    None,
    Name(Atom),
    Object(TObject),
}

#[derive(Debug)]
pub struct TypeExpansionOptions {
    pub self_class: Option<Atom>,
    pub static_class_type: StaticClassType,
    pub parent_class: Option<Atom>,
    pub evaluate_class_constants: bool,
    pub evaluate_conditional_types: bool,
    pub function_is_final: bool,
    pub expand_generic: bool,
    pub expand_templates: bool,
}

impl Default for TypeExpansionOptions {
    fn default() -> Self {
        Self {
            self_class: None,
            static_class_type: StaticClassType::default(),
            parent_class: None,
            evaluate_class_constants: true,
            evaluate_conditional_types: false,
            function_is_final: false,
            expand_generic: false,
            expand_templates: true,
        }
    }
}

pub fn expand_union(codebase: &CodebaseMetadata, return_type: &mut TUnion, options: &TypeExpansionOptions) {
    if !return_type.is_expandable() {
        return;
    }

    let mut types = std::mem::take(&mut return_type.types).into_owned();

    types = combiner::combine(types, codebase, false);

    let mut new_return_type_parts = vec![];
    let mut skipped_keys = vec![];

    for (i, return_type_part) in types.iter_mut().enumerate() {
        let mut skip_key = false;
        expand_atomic(return_type_part, codebase, options, &mut skip_key, &mut new_return_type_parts);

        if skip_key {
            skipped_keys.push(i);
        }
    }

    if !skipped_keys.is_empty() {
        let mut i = 0;
        types.retain(|_| {
            let to_retain = !skipped_keys.contains(&i);
            i += 1;
            to_retain
        });

        new_return_type_parts.append(&mut types);

        if new_return_type_parts.is_empty() {
            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
        }

        types = if new_return_type_parts.len() > 1 {
            combiner::combine(new_return_type_parts, codebase, false)
        } else {
            new_return_type_parts
        };
    }

    return_type.types = Cow::Owned(types);
}

pub(crate) fn expand_atomic(
    return_type_part: &mut TAtomic,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
    skip_key: &mut bool,
    new_return_type_parts: &mut Vec<TAtomic>,
) {
    match return_type_part {
        TAtomic::Array(array_type) => match array_type {
            TArray::Keyed(keyed_data) => {
                if let Some((key_parameter, value_parameter)) = &mut keyed_data.parameters {
                    expand_union(codebase, key_parameter, options);
                    expand_union(codebase, value_parameter, options);
                }

                if let Some(known_items) = &mut keyed_data.known_items {
                    for (_, item_type) in known_items.values_mut() {
                        expand_union(codebase, item_type, options);
                    }
                }
            }
            TArray::List(list_data) => {
                expand_union(codebase, &mut list_data.element_type, options);

                if let Some(known_elements) = &mut list_data.known_elements {
                    for (_, element_type) in known_elements.values_mut() {
                        expand_union(codebase, element_type, options);
                    }
                }
            }
        },
        TAtomic::Object(object) => {
            expand_object(object, codebase, options);
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            if let Some(return_type) = signature.get_return_type_mut() {
                expand_union(codebase, return_type, options);
            }

            for param in signature.get_parameters_mut() {
                if let Some(param_type) = param.get_type_signature_mut() {
                    expand_union(codebase, param_type, options);
                }
            }
        }
        TAtomic::GenericParameter(parameter) => {
            expand_union(codebase, parameter.constraint.as_mut(), options);
        }
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
            let mut atomic_return_type_parts = vec![];
            expand_atomic(constraint, codebase, options, &mut false, &mut atomic_return_type_parts);

            if !atomic_return_type_parts.is_empty() {
                *constraint = Box::new(atomic_return_type_parts.remove(0));
            }
        }
        TAtomic::Reference(TReference::Member { class_like_name, member_selector }) => {
            *skip_key = true;

            match member_selector {
                TReferenceMemberSelector::Wildcard => {
                    let Some(class_like) = get_class_like(codebase, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));

                        return;
                    };

                    for constant in class_like.constants.values() {
                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types.into_owned());
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::StartsWith(prefix) => {
                    let Some(class_like) = get_class_like(codebase, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));

                        return;
                    };

                    for (constant_name, constant) in class_like.constants.iter() {
                        if !constant_name.starts_with(prefix.as_str()) {
                            continue;
                        }

                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types.into_owned());
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        if !enum_case_name.starts_with(prefix.as_str()) {
                            continue;
                        }

                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::EndsWith(suffix) => {
                    let Some(class_like) = get_class_like(codebase, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));

                        return;
                    };

                    for (constant_name, constant) in class_like.constants.iter() {
                        if !constant_name.ends_with(suffix.as_str()) {
                            continue;
                        }

                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types.into_owned());
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        if !enum_case_name.ends_with(suffix.as_str()) {
                            continue;
                        }

                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::Identifier(member_name) => {
                    let Some(class_like) = get_class_like(codebase, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));

                        return;
                    };

                    if class_like.enum_cases.contains_key(member_name) {
                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *member_name)));
                    } else if let Some(constant) = class_like.constants.get(member_name) {
                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types.into_owned());
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
                        }
                    } else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
                    }
                }
            }
        }
        TAtomic::Callable(TCallable::Alias(id)) => {
            if let Some(value) = get_atomic_of_function_like_identifier(id, codebase) {
                *skip_key = true;
                new_return_type_parts.push(value);
            }
        }
        TAtomic::Conditional(conditional) => {
            *skip_key = true;

            let mut then = conditional.then.clone();
            let mut otherwise = conditional.otherwise.clone();

            expand_union(codebase, &mut then, options);
            expand_union(codebase, &mut otherwise, options);

            new_return_type_parts.extend(then.types.into_owned());
            new_return_type_parts.extend(otherwise.types.into_owned());
        }
        TAtomic::Derived(derived) => match derived {
            TDerived::KeyOf(key_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_key_of(key_of, codebase, options));
            }
            TDerived::ValueOf(value_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_value_of(value_of, codebase, options));
            }
            TDerived::PropertiesOf(_) => todo!("expand_properties_of"),
        },
        _ => {}
    }
}

fn expand_object(named_object: &mut TObject, codebase: &CodebaseMetadata, options: &TypeExpansionOptions) {
    let Some(name) = named_object.get_name().copied() else {
        return;
    };

    let is_this = if let TObject::Named(named_object) = named_object { named_object.is_this() } else { false };
    let name_str_lc = ascii_lowercase_atom(&name);

    if is_this || name_str_lc == "static" || name_str_lc == "$this" {
        match &options.static_class_type {
            StaticClassType::Object(TObject::Enum(static_enum)) => {
                *named_object = TObject::Enum(static_enum.clone());
            }
            StaticClassType::Object(TObject::Named(static_object)) => {
                if let TObject::Named(named_object) = named_object {
                    if let Some(static_object_intersections) = &static_object.intersection_types {
                        let intersections = named_object.intersection_types.get_or_insert_with(Vec::new);
                        intersections.extend(static_object_intersections.iter().cloned());
                    }

                    if named_object.type_parameters.is_none() {
                        named_object.type_parameters = static_object.type_parameters.clone();
                    }

                    named_object.name = static_object.name;
                    named_object.is_this = true;
                }
            }
            StaticClassType::Name(static_class_name) => {
                if let TObject::Named(named_object) = named_object {
                    named_object.name = *static_class_name;
                    named_object.is_this = options.function_is_final;
                }
            }
            _ => {}
        }
    } else if name_str_lc == "self" {
        if let Some(self_class_name) = options.self_class
            && let TObject::Named(named_object) = named_object
        {
            named_object.name = self_class_name;
        }
    } else if name_str_lc == "parent"
        && let Some(self_class_name) = options.self_class
        && let Some(class_metadata) = get_class_like(codebase, &self_class_name)
        && let Some(parent_name) = class_metadata.direct_parent_class
        && let TObject::Named(named_object) = named_object
    {
        named_object.name = parent_name;
    }

    let TObject::Named(named_object) = named_object else {
        return;
    };

    if named_object.type_parameters.is_none()
        && let Some(class_like_metadata) = get_class_like(codebase, &named_object.name)
        && !class_like_metadata.template_types.is_empty()
    {
        let default_params: Vec<TUnion> = class_like_metadata
            .template_types
            .iter()
            .map(|(_, template_map)| template_map.iter().map(|(_, t)| t).next().cloned().unwrap_or_else(get_mixed))
            .collect();

        if !default_params.is_empty() {
            named_object.type_parameters = Some(default_params);
        }
    }
}

pub fn get_signature_of_function_like_identifier(
    function_like_identifier: &FunctionLikeIdentifier,
    codebase: &CodebaseMetadata,
) -> Option<TCallableSignature> {
    Some(match function_like_identifier {
        FunctionLikeIdentifier::Function(name) => {
            let function_like_metadata = get_function(codebase, name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Closure(file_id, position) => {
            let function_like_metadata = get_closure(codebase, file_id, position)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Method(classlike_name, method_name) => {
            let function_like_metadata = get_declaring_method(codebase, classlike_name, method_name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions {
                    self_class: Some(*classlike_name),
                    static_class_type: StaticClassType::Name(*classlike_name),
                    ..Default::default()
                },
            )
        }
    })
}

pub fn get_atomic_of_function_like_identifier(
    function_like_identifier: &FunctionLikeIdentifier,
    codebase: &CodebaseMetadata,
) -> Option<TAtomic> {
    let signature = get_signature_of_function_like_identifier(function_like_identifier, codebase)?;

    Some(TAtomic::Callable(TCallable::Signature(signature)))
}

pub fn get_signature_of_function_like_metadata(
    function_like_identifier: &FunctionLikeIdentifier,
    function_like_metadata: &FunctionLikeMetadata,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> TCallableSignature {
    let parameters: Vec<_> = function_like_metadata
        .parameters
        .iter()
        .map(|parameter_metadata| {
            let type_signature = if let Some(t) = parameter_metadata.get_type_metadata() {
                let mut t = t.type_union.clone();
                expand_union(codebase, &mut t, options);
                Some(Box::new(t))
            } else {
                None
            };

            TCallableParameter::new(
                type_signature,
                parameter_metadata.flags.is_by_reference(),
                parameter_metadata.flags.is_variadic(),
                parameter_metadata.flags.has_default(),
            )
        })
        .collect();

    let return_type = if let Some(type_metadata) = function_like_metadata.return_type_metadata.as_ref() {
        let mut return_type = type_metadata.type_union.clone();
        expand_union(codebase, &mut return_type, options);
        Some(Box::new(return_type))
    } else {
        None
    };

    let mut signature = TCallableSignature::new(function_like_metadata.flags.is_pure(), true)
        .with_parameters(parameters)
        .with_return_type(return_type)
        .with_source(Some(*function_like_identifier));

    if let FunctionLikeIdentifier::Closure(file_id, closure_position) = function_like_identifier {
        signature = signature.with_closure_location(Some((*file_id, *closure_position)));
    }

    signature
}

fn expand_key_of(
    return_type_key_of: &TKeyOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut type_atomics = vec![];

    let mut target_type = return_type_key_of.get_target_type().clone();
    let mut new_atomics = vec![];
    let mut remove_target_atomic = false;
    expand_atomic(&mut target_type, codebase, options, &mut remove_target_atomic, &mut new_atomics);
    type_atomics.extend(new_atomics);
    if !remove_target_atomic {
        type_atomics.push(target_type);
    }

    let Some(new_return_types) = TKeyOf::get_key_of_targets(&type_atomics, codebase, false) else {
        return vec![TAtomic::Derived(TDerived::KeyOf(return_type_key_of.clone()))];
    };

    new_return_types.types.into_owned()
}

fn expand_value_of(
    return_type_value_of: &TValueOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut type_atomics = vec![];

    let mut target_type = return_type_value_of.get_target_type().clone();
    let mut new_atomics = vec![];
    let mut remove_target_atomic = false;
    expand_atomic(&mut target_type, codebase, options, &mut remove_target_atomic, &mut new_atomics);
    type_atomics.extend(new_atomics);
    if !remove_target_atomic {
        type_atomics.push(target_type);
    }

    let Some(new_return_types) = TValueOf::get_value_of_targets(&type_atomics, codebase, false) else {
        return vec![TAtomic::Derived(TDerived::ValueOf(return_type_value_of.clone()))];
    };

    new_return_types.types.into_owned()
}
