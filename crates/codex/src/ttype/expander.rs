use itertools::Itertools;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_source::SourceIdentifier;

use crate::get_class_like;
use crate::get_closure;
use crate::get_declaring_method;
use crate::get_function;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
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
use crate::ttype::extend_dataflow_uniquely;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum StaticClassType {
    #[default]
    None,
    Name(StringIdentifier),
    Object(TObject),
}

#[derive(Debug)]
pub struct TypeExpansionOptions<'a> {
    pub self_class: Option<&'a StringIdentifier>,
    pub static_class_type: StaticClassType,
    pub parent_class: Option<&'a StringIdentifier>,
    pub file_path: Option<&'a SourceIdentifier>,
    pub evaluate_class_constants: bool,
    pub evaluate_conditional_types: bool,
    pub function_is_final: bool,
    pub expand_generic: bool,
    pub expand_templates: bool,
}

impl Default for TypeExpansionOptions<'_> {
    fn default() -> Self {
        Self {
            file_path: None,
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

pub fn expand_union(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    return_type: &mut TUnion,
    options: &TypeExpansionOptions,
) {
    let previous_types = std::mem::take(&mut return_type.types);
    return_type.types = combiner::combine(previous_types, codebase, interner, false);

    let mut new_return_type_parts = vec![];

    let mut skipped_keys = vec![];

    for (i, return_type_part) in return_type.types.iter_mut().enumerate() {
        let mut skip_key = false;
        expand_atomic(return_type_part, codebase, interner, options, &mut skip_key, &mut new_return_type_parts);

        if skip_key {
            skipped_keys.push(i);
        }
    }

    if !skipped_keys.is_empty() {
        let mut i = 0;

        return_type.types.retain(|_| {
            let to_retain = !skipped_keys.contains(&i);
            i += 1;
            to_retain
        });

        new_return_type_parts.extend(return_type.types.drain(..).collect_vec());

        if new_return_type_parts.is_empty() {
            new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
        }

        if new_return_type_parts.len() > 1 {
            return_type.types = combiner::combine(new_return_type_parts, codebase, interner, false)
        } else {
            return_type.types = new_return_type_parts;
        }
    }

    extend_dataflow_uniquely(&mut return_type.parent_nodes, vec![]);
}

fn expand_atomic(
    return_type_part: &mut TAtomic,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    options: &TypeExpansionOptions,
    skip_key: &mut bool,
    new_return_type_parts: &mut Vec<TAtomic>,
) {
    match return_type_part {
        TAtomic::Array(array_type) => match array_type {
            TArray::Keyed(keyed_data) => {
                if let Some((key_parameter, value_parameter)) = &mut keyed_data.parameters {
                    expand_union(codebase, interner, key_parameter, options);
                    expand_union(codebase, interner, value_parameter, options);
                }

                if let Some(known_items) = &mut keyed_data.known_items {
                    for (_, item_type) in known_items.values_mut() {
                        expand_union(codebase, interner, item_type, options);
                    }
                }
            }
            TArray::List(list_data) => {
                expand_union(codebase, interner, &mut list_data.element_type, options);

                if let Some(known_elements) = &mut list_data.known_elements {
                    for (_, element_type) in known_elements.values_mut() {
                        expand_union(codebase, interner, element_type, options);
                    }
                }
            }
        },
        TAtomic::Object(TObject::Named(named_object)) => {
            if named_object.is_this()
                && let StaticClassType::Object(obj) = &options.static_class_type
                && let TObject::Named(new_this_object) = &obj
                && is_instance_of(codebase, interner, &new_this_object.name, &named_object.name)
            {
                *skip_key = true;
                new_return_type_parts.push(TAtomic::Object(obj.clone()));
                return;
            }

            if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                for parameter in type_parameters {
                    expand_union(codebase, interner, parameter, options);
                }
            }
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            if let Some(return_type) = signature.get_return_type_mut() {
                expand_union(codebase, interner, return_type, options);
            }

            for param in signature.get_parameters_mut() {
                if let Some(param_type) = param.get_type_signature_mut() {
                    expand_union(codebase, interner, param_type, options);
                }
            }
        }
        TAtomic::GenericParameter(parameter) => {
            expand_union(codebase, interner, parameter.constraint.as_mut(), options);
        }
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
            let mut atomic_return_type_parts = vec![];
            expand_atomic(constraint, codebase, interner, options, &mut false, &mut atomic_return_type_parts);

            if !atomic_return_type_parts.is_empty() {
                *constraint = Box::new(atomic_return_type_parts.remove(0));
            }
        }
        TAtomic::Reference(TReference::Member { class_like_name, member_selector }) => {
            *skip_key = true;

            match member_selector {
                TReferenceMemberSelector::Wildcard => {
                    let Some(class_like) = get_class_like(codebase, interner, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));

                        return;
                    };

                    for constant in class_like.constants.values() {
                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                interner,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, interner, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types);
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::StartsWith(prefix) => {
                    let Some(class_like) = get_class_like(codebase, interner, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));

                        return;
                    };

                    let prefix_str = interner.lookup(prefix);

                    for (constant_name, constant) in class_like.constants.iter() {
                        let constant_name_str = interner.lookup(constant_name);

                        if !constant_name_str.starts_with(prefix_str) {
                            continue;
                        }

                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                interner,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, interner, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types);
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        let enum_case_name_str = interner.lookup(enum_case_name);

                        if !enum_case_name_str.starts_with(prefix_str) {
                            continue;
                        }

                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::EndsWith(suffix) => {
                    let Some(class_like) = get_class_like(codebase, interner, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));

                        return;
                    };

                    let suffix_str = interner.lookup(suffix);

                    for (constant_name, constant) in class_like.constants.iter() {
                        let constant_name_str = interner.lookup(constant_name);

                        if !constant_name_str.ends_with(suffix_str) {
                            continue;
                        }

                        if let Some(inferred_type) = constant.inferred_type.as_ref() {
                            let mut inferred_type = inferred_type.clone();

                            let mut skip_inferred_type = false;
                            expand_atomic(
                                &mut inferred_type,
                                codebase,
                                interner,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, interner, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types);
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
                        }
                    }

                    for enum_case_name in class_like.enum_cases.keys() {
                        let enum_case_name_str = interner.lookup(enum_case_name);

                        if !enum_case_name_str.ends_with(suffix_str) {
                            continue;
                        }

                        new_return_type_parts
                            .push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
                    }
                }
                TReferenceMemberSelector::Identifier(member_name) => {
                    let Some(class_like) = get_class_like(codebase, interner, class_like_name) else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));

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
                                interner,
                                options,
                                &mut skip_inferred_type,
                                new_return_type_parts,
                            );

                            if !skip_inferred_type {
                                new_return_type_parts.push(inferred_type);
                            }
                        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
                            let mut constant_type = type_metadata.type_union.clone();

                            expand_union(codebase, interner, &mut constant_type, options);

                            new_return_type_parts.extend(constant_type.types);
                        } else {
                            new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
                        }
                    } else {
                        new_return_type_parts.push(TAtomic::Mixed(TMixed::vanilla()));
                    }
                }
            }
        }
        TAtomic::Callable(TCallable::Alias(id)) => {
            if let Some(value) = get_atomic_of_function_like_identifier(id, codebase, interner) {
                *skip_key = true;
                new_return_type_parts.push(value);
            }
        }
        TAtomic::Conditional(conditional) => {
            *skip_key = true;

            let mut then = conditional.then.clone();
            let mut otherwise = conditional.otherwise.clone();

            expand_union(codebase, interner, &mut then, options);
            expand_union(codebase, interner, &mut otherwise, options);

            new_return_type_parts.extend(then.types);
            new_return_type_parts.extend(otherwise.types);
        }
        TAtomic::Derived(derived) => match derived {
            TDerived::KeyOf(key_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_key_of(key_of, codebase, interner, options));
            }
            TDerived::ValueOf(value_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_value_of(value_of, codebase, interner, options));
            }
            TDerived::PropertiesOf(_) => todo!("expand_properties_of"),
        },
        _ => {}
    }
}

pub fn get_signature_of_function_like_identifier(
    function_like_identifier: &FunctionLikeIdentifier,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> Option<TCallableSignature> {
    Some(match function_like_identifier {
        FunctionLikeIdentifier::Function(name) => {
            let function_like_metadata = get_function(codebase, interner, name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                interner,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Closure(position) => {
            let function_like_metadata = get_closure(codebase, interner, position)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                interner,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Method(classlike_name, method_name) => {
            let function_like_metadata = get_declaring_method(codebase, interner, classlike_name, method_name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                interner,
                &TypeExpansionOptions {
                    self_class: Some(classlike_name),
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
    interner: &ThreadedInterner,
) -> Option<TAtomic> {
    let signature = get_signature_of_function_like_identifier(function_like_identifier, codebase, interner)?;

    Some(TAtomic::Callable(TCallable::Signature(signature)))
}

pub fn get_signature_of_function_like_metadata(
    function_like_identifier: &FunctionLikeIdentifier,
    function_like_metadata: &FunctionLikeMetadata,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    options: &TypeExpansionOptions,
) -> TCallableSignature {
    let parameters: Vec<_> = function_like_metadata
        .get_parameters()
        .iter()
        .map(|parameter_metadata| {
            let type_signature = if let Some(t) = parameter_metadata.get_type_metadata() {
                let mut t = t.type_union.clone();
                expand_union(codebase, interner, &mut t, options);
                Some(Box::new(t))
            } else {
                None
            };

            TCallableParameter::new(
                type_signature,
                parameter_metadata.is_by_reference(),
                parameter_metadata.is_variadic(),
                parameter_metadata.has_default(),
            )
        })
        .collect();

    let return_type = if let Some(type_metadata) = function_like_metadata.return_type_metadata.as_ref() {
        let mut return_type = type_metadata.type_union.clone();
        expand_union(codebase, interner, &mut return_type, options);
        Some(Box::new(return_type))
    } else {
        None
    };

    let mut signature = TCallableSignature::new(function_like_metadata.is_pure, true)
        .with_parameters(parameters)
        .with_return_type(return_type)
        .with_source(Some(*function_like_identifier));

    if let FunctionLikeIdentifier::Closure(closure_position) = function_like_identifier {
        signature = signature.with_closure_position(Some(*closure_position));
    }

    signature
}

fn expand_key_of(
    return_type_key_of: &TKeyOf,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut type_atomics = vec![];

    let mut target_type = return_type_key_of.get_target_type().clone();
    let mut new_atomics = vec![];
    let mut remove_target_atomic = false;
    expand_atomic(&mut target_type, codebase, interner, options, &mut remove_target_atomic, &mut new_atomics);
    type_atomics.extend(new_atomics);
    if !remove_target_atomic {
        type_atomics.push(target_type);
    }

    let Some(new_return_types) = TKeyOf::get_key_of_targets(type_atomics, codebase, interner, false) else {
        return vec![TAtomic::Derived(TDerived::KeyOf(return_type_key_of.clone()))];
    };

    new_return_types.types
}

fn expand_value_of(
    return_type_value_of: &TValueOf,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut type_atomics = vec![];

    let mut target_type = return_type_value_of.get_target_type().clone();
    let mut new_atomics = vec![];
    let mut remove_target_atomic = false;
    expand_atomic(&mut target_type, codebase, interner, options, &mut remove_target_atomic, &mut new_atomics);
    type_atomics.extend(new_atomics);
    if !remove_target_atomic {
        type_atomics.push(target_type);
    }

    let Some(new_return_types) = TValueOf::get_value_of_targets(type_atomics, codebase, interner, false) else {
        return vec![TAtomic::Derived(TDerived::ValueOf(return_type_value_of.clone()))];
    };

    new_return_types.types
}
