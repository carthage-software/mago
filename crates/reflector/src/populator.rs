use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::identifier::ClassLikeName;
use fennec_reflection::CodebaseReflection;

#[inline(always)]
pub fn populate(interner: &ThreadedInterner, codebase: &mut CodebaseReflection) {
    if codebase.populated {
        return;
    }

    let new_class_like_names = codebase
        .class_like_reflections
        .iter()
        .filter_map(|(name, reflection)| if !reflection.is_populated { Some(*name) } else { None })
        .collect::<Vec<_>>();

    for name in &new_class_like_names {
        if let Some(reflection) = codebase.class_like_reflections.get_mut(name) {
            reflection.properties.declaring_members = Default::default();
            reflection.properties.appering_members = Default::default();
            reflection.methods.declaring_members = Default::default();
            reflection.methods.appering_members = Default::default();
        }
    }

    for class_like_name in &new_class_like_names {
        populate_class_like_reflection(interner, codebase, *class_like_name);
    }

    for (classlike_name, classlike_reflection) in &codebase.class_like_reflections {
        let Some(classlike_name) = classlike_name.inner().map(|v| v.value) else {
            continue;
        };

        if let Some(parent_class) = &classlike_reflection.inheritance.direct_extended_class {
            codebase.direct_classlike_descendants.entry(parent_class.value).or_default().insert(classlike_name);
        }

        for parent_class in &classlike_reflection.inheritance.all_extended_classes {
            codebase.all_classlike_descendants.entry(parent_class.value).or_default().insert(classlike_name);
        }

        for parent_interface in &classlike_reflection.inheritance.direct_implemented_interfaces {
            codebase.direct_classlike_descendants.entry(parent_interface.value).or_default().insert(classlike_name);
        }

        for parent_interface in &classlike_reflection.inheritance.all_extended_interfaces {
            codebase.all_classlike_descendants.entry(parent_interface.value).or_default().insert(classlike_name);
        }

        for used_trait in &classlike_reflection.used_traits {
            codebase.all_classlike_descendants.entry(*used_trait).or_default().insert(classlike_name);
        }
    }

    codebase.all_classlike_descendants.shrink_to_fit();
    codebase.direct_classlike_descendants.shrink_to_fit();
    codebase.populated = true;
}

#[inline]
fn populate_class_like_reflection(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    class_like_name: ClassLikeName,
) {
    let Some(mut reflection) = codebase.class_like_reflections.remove(&class_like_name) else {
        return;
    };

    if reflection.is_populated {
        codebase.class_like_reflections.insert(class_like_name, reflection);

        return;
    }

    for property_id in reflection.properties.members.keys() {
        reflection.properties.appering_members.insert(*property_id, reflection.name);
        reflection.properties.declaring_members.insert(*property_id, reflection.name);
    }

    for method_id in reflection.methods.members.keys() {
        reflection.methods.appering_members.insert(*method_id, reflection.name);
        reflection.methods.declaring_members.insert(*method_id, reflection.name);
    }

    for trait_name in reflection.used_traits.clone() {
        populate_data_from_trait(interner, codebase, &mut reflection, trait_name);
    }

    if let Some(parent_classname) = reflection.inheritance.direct_extended_class {
        populate_data_from_parent_classlike(interner, codebase, &mut reflection, parent_classname.value);
    }

    for parent_interface in reflection.inheritance.direct_extended_interfaces.clone() {
        populate_interface_data_from_parent_interface(interner, codebase, &mut reflection, parent_interface.value);
    }

    reflection.inheritance.all_extended_classes.shrink_to_fit();
    reflection.inheritance.all_implemented_interfaces.shrink_to_fit();
    reflection.inheritance.names.shrink_to_fit();
    reflection.constants.shrink_to_fit();
    reflection.properties.members.shrink_to_fit();
    reflection.properties.appering_members.shrink_to_fit();
    reflection.properties.declaring_members.shrink_to_fit();
    reflection.methods.members.shrink_to_fit();
    reflection.methods.appering_members.shrink_to_fit();
    reflection.methods.declaring_members.shrink_to_fit();

    reflection.is_populated = true;

    codebase.class_like_reflections.insert(class_like_name, reflection);
}

#[inline]
fn populate_interface_data_from_parent_interface(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
    parent_name_id: StringIdentifier,
) {
    let Some(parent_name) = codebase.class_like_names.get(&parent_name_id).cloned() else {
        return;
    };

    populate_class_like_reflection(interner, codebase, parent_name);

    let Some(parent_reflection) = codebase.class_like_reflections.get_mut(&parent_name) else {
        return;
    };

    for (constant_name, constant) in parent_reflection.constants.iter() {
        if reflection.constants.contains_key(constant_name) {
            continue;
        }

        reflection.constants.insert(constant_name.clone(), constant.clone());
    }

    inherit_methods_from_parent(reflection, parent_reflection);

    for parent_interface_name in parent_reflection.inheritance.all_extended_interfaces.clone() {
        if reflection.inheritance.all_extended_interfaces.contains(&parent_interface_name) {
            continue;
        }

        reflection.inheritance.all_extended_interfaces.insert(parent_interface_name);
    }
}

#[inline]
fn populate_data_from_parent_classlike(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
    parent_name_id: StringIdentifier,
) {
    let Some(parent_name) = codebase.class_like_names.get(&parent_name_id).cloned() else {
        return;
    };

    populate_class_like_reflection(interner, codebase, parent_name);

    let Some(parent_reflection) = codebase.class_like_reflections.get_mut(&parent_name) else {
        return;
    };

    for extended_class in &parent_reflection.inheritance.all_extended_classes {
        if reflection.inheritance.all_extended_classes.contains(extended_class) {
            continue;
        }

        reflection.inheritance.all_extended_classes.insert(*extended_class);
        reflection.inheritance.names.insert(extended_class.value, *extended_class);
    }

    for implemented_interface in &parent_reflection.inheritance.all_implemented_interfaces {
        if reflection.inheritance.all_implemented_interfaces.contains(implemented_interface) {
            continue;
        }

        reflection.inheritance.all_implemented_interfaces.insert(*implemented_interface);
        reflection.inheritance.names.insert(implemented_interface.value, *implemented_interface);
    }

    for (used_trait, used_trait_name) in &parent_reflection.used_trait_names {
        if reflection.used_traits.contains(used_trait) {
            continue;
        }

        reflection.used_traits.insert(*used_trait);
        reflection.used_trait_names.insert(*used_trait, *used_trait_name);
    }

    for (constant_name, constant) in &parent_reflection.constants {
        if reflection.constants.contains_key(constant_name) {
            continue;
        }

        reflection.constants.insert(*constant_name, constant.clone());
    }

    inherit_properties_from_parent(reflection, parent_reflection);
    inherit_methods_from_parent(reflection, parent_reflection);

    parent_reflection.inheritance.children.insert(reflection.name);
}

#[inline]
fn populate_data_from_trait(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
    trait_name_id: StringIdentifier,
) {
    let Some(trait_name) = codebase.class_like_names.get(&trait_name_id).cloned() else {
        return;
    };

    populate_class_like_reflection(interner, codebase, trait_name);

    let Some(trait_reflection) = codebase.class_like_reflections.get(&trait_name) else {
        return;
    };

    inherit_properties_from_parent(reflection, trait_reflection);
    inherit_methods_from_parent(reflection, trait_reflection);
}

#[inline]
fn inherit_properties_from_parent(reflection: &mut ClassLikeReflection, parent_reflection: &ClassLikeReflection) {
    let class_name = reflection.name;
    let class_is_trait = reflection.is_trait();
    let parent_is_trait = parent_reflection.is_trait();

    for (property_name, appearing_classlike) in &parent_reflection.properties.appering_members {
        if reflection.properties.appering_members.contains_key(property_name) {
            continue;
        }

        if !parent_is_trait {
            if let Some(parent_property_storage) = parent_reflection.get_property(property_name) {
                if parent_property_storage.write_visibility_reflection.map(|v| v.is_private()).unwrap_or(false) {
                    continue;
                }
            }
        }

        reflection
            .properties
            .appering_members
            .insert(*property_name, if class_is_trait { class_name } else { *appearing_classlike });
    }

    for (property_name, declaring_classlike) in &parent_reflection.properties.declaring_members {
        if reflection.properties.declaring_members.contains_key(property_name) {
            continue;
        }

        if !parent_is_trait {
            if let Some(parent_property_storage) = parent_reflection.get_property(property_name) {
                if parent_property_storage.write_visibility_reflection.map(|v| v.is_private()).unwrap_or(false) {
                    continue;
                }
            }
        }

        reflection.properties.declaring_members.insert(*property_name, *declaring_classlike);
    }

    // register inheritance
    for (property_name, inheritable_classlike) in &parent_reflection.properties.inheritable_members {
        if !parent_is_trait {
            if let Some(parent_property_storage) = parent_reflection.get_property(property_name) {
                if parent_property_storage.write_visibility_reflection.map(|v| v.is_private()).unwrap_or(false) {
                    continue;
                }
            }

            reflection.properties.overriden_members.entry(*property_name).or_default().insert(*inheritable_classlike);
        }

        reflection.properties.inheritable_members.insert(*property_name, *inheritable_classlike);
    }
}

#[inline]
fn inherit_methods_from_parent(reflection: &mut ClassLikeReflection, parent_reflection: &ClassLikeReflection) {
    let class_name = reflection.name;
    let class_is_trait = reflection.is_trait();

    for (method_name, appering_class_like) in &parent_reflection.methods.appering_members {
        if reflection.methods.appering_members.contains_key(method_name) {
            continue;
        }

        reflection
            .methods
            .appering_members
            .insert(*method_name, if class_is_trait { class_name } else { *appering_class_like });
    }

    for (method_name, declaring_class) in &parent_reflection.methods.inheritable_members {
        reflection.methods.overriden_members.entry(*method_name).or_default().insert(*declaring_class);

        if let Some(map) = reflection.methods.overriden_members.get_mut(method_name) {
            map.extend(parent_reflection.methods.overriden_members.get(method_name).cloned().unwrap_or_default())
        }

        if let Some(_) = reflection.methods.declaring_members.get(method_name) {
            continue;
        }

        reflection.methods.declaring_members.insert(*method_name, *declaring_class);

        if !reflection.is_trait()
            || !reflection.inheritance.require_extensions.contains(&parent_reflection.name.inner().unwrap().value)
        {
            reflection.methods.inheritable_members.insert(*method_name, *declaring_class);
        }
    }
}
