use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_interner::StringIdentifier;

use crate::context::Context;

/// Resolves and expands template types applicable to a class member (method or property)
/// within a specific call context.
///
/// This function determines the concrete types for template parameters defined either
/// directly on the member (`existing_template_types`) or on its declaring class.
/// It considers the context of the call (`calling_class_meta`) and how the calling
/// class might extend the declaring class (`template_extended_parameters`), merging these
/// with any template arguments already resolved at the class level (`class_template_parameters`).
///
/// It handles template resolution through inheritance chains using `get_generic_parameter_for_offset`.
/// Finally, it expands the resolved types using `expander::expand_union` to resolve
/// types like `self`, `static`, etc., within the final template type definitions.
///
/// # Arguments
/// * `context` - The analysis context, providing codebase and interner access.
/// * `declaring_class_meta` - Metadata of the class where the member is originally declared.
/// * `appearing_class_name` - The name of the class through which the member is being accessed (might differ from declaring class due to inheritance). Used for `self::` resolution during final expansion.
/// * `calling_class_meta` - Metadata of the class context from which the call originates (`$this` or `static::class`). Used for `static::` resolution.
/// * `existing_template_types` - Template types defined directly on the function/method itself (e.g., `@template TMethod`). These take precedence.
/// * `class_template_parameters` - Concrete types already resolved for the *class's* template parameters in the current context (e.g., if analyzing `$obj` of type `Vec<int>`, this map would contain `TValue => int`).
///
/// # Returns
///
/// An `IndexMap` where keys are template parameter names (`StringIdentifier`) and values are
/// `HashMap`s mapping the defining entity (`GenericParent` - class or function) to the
/// fully resolved and expanded `TUnion` for that template parameter in this specific context.
pub fn get_template_types_for_class_member(
    context: &Context<'_>,
    declaring_class_meta: Option<&ClassLikeMetadata>,
    appearing_class_name: Option<&StringIdentifier>,
    calling_class_meta: Option<&ClassLikeMetadata>,
    existing_template_types: &[(StringIdentifier, Vec<(GenericParent, TUnion)>)],
    class_template_parameters: &IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState>,
) -> IndexMap<StringIdentifier, HashMap<GenericParent, TUnion>, RandomState> {
    let codebase = context.codebase;
    let interner = context.interner;

    let mut template_types: IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState> =
        IndexMap::from_iter(existing_template_types.iter().cloned());

    if let Some(declaring_class_meta) = declaring_class_meta {
        let declaring_class_name = declaring_class_meta.name;

        let calling_has_extends = calling_class_meta
            .map(|calling_meta| {
                calling_meta.name != declaring_class_name && !calling_meta.template_extended_parameters.is_empty()
            })
            .unwrap_or(false);

        if calling_has_extends {
            let calling_meta = calling_class_meta.unwrap();
            let calling_template_extended = &calling_meta.template_extended_parameters;

            for (extended_class_name, type_map) in calling_template_extended {
                if extended_class_name == &declaring_class_name {
                    for (template_name, provided_type_arc) in type_map {
                        let resolved_type = if provided_type_arc.has_template_types() {
                            let mut resolved_union = None;
                            for atomic_type in &provided_type_arc.types {
                                let resolved_atomic_type_union = if let TAtomic::GenericParameter(TGenericParameter {
                                    defining_entity: GenericParent::ClassLike(defining_entity),
                                    parameter_name,
                                    ..
                                }) = atomic_type
                                {
                                    let mut combined_parameters = class_template_parameters.clone();
                                    combined_parameters.extend(template_types.clone());

                                    get_generic_parameter_for_offset(
                                        defining_entity,
                                        parameter_name,
                                        calling_template_extended,
                                        &combined_parameters.into_iter().collect::<HashMap<_, _>>(),
                                    )
                                } else {
                                    wrap_atomic(atomic_type.clone())
                                };

                                resolved_union = Some(add_optional_union_type(
                                    resolved_atomic_type_union,
                                    resolved_union.as_ref(),
                                    codebase,
                                    interner,
                                ));
                            }

                            resolved_union.unwrap_or_else(get_mixed_any)
                        } else {
                            provided_type_arc.clone()
                        };

                        template_types
                            .entry(*template_name)
                            .or_default()
                            .push((GenericParent::ClassLike(declaring_class_name), resolved_type));
                    }
                }
            }
        } else if !declaring_class_meta.template_types.is_empty() {
            for (template_name, type_map) in &declaring_class_meta.template_types {
                for (defining_parent, default_type) in type_map {
                    let concrete_type = class_template_parameters.get(template_name).and_then(|parameters| {
                        parameters.iter().find(|(p, _)| p == defining_parent).map(|(_, t)| t.clone())
                    });

                    let resolved_type = concrete_type.unwrap_or_else(|| default_type.clone());

                    template_types.entry(*template_name).or_default().push((*defining_parent, resolved_type));
                }
            }
        }
    }

    let mut expanded_template_types = IndexMap::with_hasher(RandomState::new());
    for (template_name, type_map_vec) in template_types {
        let final_map_entry: &mut HashMap<GenericParent, TUnion> =
            expanded_template_types.entry(template_name).or_default();

        for (generic_parent, mut expanded_union) in type_map_vec {
            expander::expand_union(
                codebase,
                interner,
                &mut expanded_union,
                &TypeExpansionOptions {
                    self_class: appearing_class_name,
                    static_class_type: if let Some(calling_meta) = calling_class_meta {
                        StaticClassType::Name(calling_meta.name)
                    } else {
                        StaticClassType::None
                    },
                    parent_class: declaring_class_meta.and_then(|m| m.direct_parent_class.as_ref()),
                    function_is_final: calling_class_meta.is_some_and(|m| m.is_final),
                    file_path: Some(&context.source.identifier),
                    expand_templates: true,
                    ..Default::default()
                },
            );

            final_map_entry.insert(generic_parent, expanded_union);
        }
    }

    expanded_template_types
}

/// Recursively resolves the concrete type for a specific template parameter within a class hierarchy.
///
/// This function traces template parameter substitutions through class extensions. For example,
/// if ClassC<U> extends ClassB<U>, and ClassB<T> extends ClassA<T>, calling this function
/// to find the type for `T` in the context of `ClassC<int>` would first look up `T` in `ClassB`'s
/// context (finding `U`), and then recursively look up `U` in `ClassC`'s context, ultimately
/// resolving to `int`.
///
/// # Arguments
///
/// * `class_like_name` - The identifier of the class where the template parameter is originally defined.
/// * `template_name` - The identifier of the template parameter to resolve (e.g., `T`).
/// * `template_extended_parameters` - A map representing how classes extend others with specific template arguments
///   (e.g., `ClassB -> { ClassA -> { T -> string } }` means B extends A with T=string).
/// * `found_generic_parameters` - A map containing already resolved template types in the current specific context
///   (e.g., if we are analyzing an instance `ClassC<int>`, this might contain `U -> int`).
///
/// # Returns
///
/// An `TUnion` representing the resolved concrete type for the template parameter,
/// or `any` if it cannot be resolved.
pub fn get_generic_parameter_for_offset(
    class_like_name: &StringIdentifier,
    template_name: &StringIdentifier,
    template_extended_parameters: &HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    found_generic_parameters: &HashMap<StringIdentifier, Vec<(GenericParent, TUnion)>>,
) -> TUnion {
    if let Some(result_map) = found_generic_parameters.get(template_name)
        && let Some(found_parameter_type) = result_map
            .iter()
            .find(|(parent, _)| parent == &GenericParent::ClassLike(*class_like_name))
            .map(|(_, type_arc)| type_arc)
    {
        return found_parameter_type.clone();
    }

    for (extending_class_name, type_map) in template_extended_parameters {
        for (extended_template_name, extended_type_union) in type_map {
            for extended_atomic_type in &extended_type_union.types {
                if let TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: current_parameter_name,
                    defining_entity: GenericParent::ClassLike(current_defining_class),
                    ..
                }) = extended_atomic_type
                    && current_parameter_name == template_name
                    && current_defining_class == class_like_name
                {
                    return get_generic_parameter_for_offset(
                        extending_class_name,
                        extended_template_name,
                        template_extended_parameters,
                        found_generic_parameters,
                    );
                }
            }
        }
    }

    get_mixed_any()
}
