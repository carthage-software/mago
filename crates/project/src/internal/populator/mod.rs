use mago_interner::ThreadedInterner;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::identifier::ClassLikeName;
use mago_reflection::identifier::FunctionLikeName;
use mago_reflection::identifier::Name;
use mago_reflection::CodebaseReflection;
use mago_reflection::Reflection;

mod report;

#[inline(always)]
pub fn populate(interner: &ThreadedInterner, codebase: &mut CodebaseReflection, populate_non_user_defined: bool) {
    if codebase.populated {
        return;
    }

    populate_all_class_like_reflections(interner, codebase, populate_non_user_defined);
    populate_all_function_like_reflections(codebase, populate_non_user_defined);
    populate_all_constant_reflections(codebase, populate_non_user_defined);

    codebase.populated = true;
}

#[inline]
fn populate_all_class_like_reflections(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    populate_non_user_defined: bool,
) {
    let unpopulated_classlike_names = codebase
        .class_like_reflections
        .iter()
        .filter_map(|(name, reflection)| {
            if !populate_non_user_defined && !reflection.is_user_defined() {
                return None;
            }

            if !reflection.is_populated {
                Some(*name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for classlike_name in &unpopulated_classlike_names {
        if let Some(reflection) = codebase.class_like_reflections.get_mut(classlike_name) {
            reflection.properties.declaring_members = Default::default();
            reflection.properties.appering_members = Default::default();
            reflection.methods.declaring_members = Default::default();
            reflection.methods.appering_members = Default::default();
        }
    }

    for classlike_name in &unpopulated_classlike_names {
        populate_class_like_reflection(interner, codebase, *classlike_name, populate_non_user_defined);
    }

    for (classlike_name, classlike_reflection) in &codebase.class_like_reflections {
        let Some(classlike_name) = classlike_name.inner().map(|v| v.value).map(|s| interner.lowered(&s)) else {
            continue;
        };

        if let Some(parent_class) = &classlike_reflection.inheritance.direct_extended_class {
            let parent_class = interner.lowered(&parent_class.value);

            codebase.direct_classlike_descendants.entry(parent_class).or_default().insert(classlike_name);
        }

        for parent_interface in &classlike_reflection.inheritance.direct_implemented_interfaces {
            let parent_interface = interner.lowered(&parent_interface.value);

            codebase.direct_classlike_descendants.entry(parent_interface).or_default().insert(classlike_name);
        }

        for parent_class in &classlike_reflection.inheritance.all_extended_classes {
            let parent_class = interner.lowered(&parent_class.value);

            codebase.all_classlike_descendants.entry(parent_class).or_default().insert(classlike_name);
        }

        for parent_interface in &classlike_reflection.inheritance.all_extended_interfaces {
            let parent_interface = interner.lowered(&parent_interface.value);

            codebase.all_classlike_descendants.entry(parent_interface).or_default().insert(classlike_name);
        }

        for used_trait in &classlike_reflection.used_traits {
            codebase.all_classlike_descendants.entry(used_trait.value).or_default().insert(classlike_name);
        }
    }
}

#[inline]
fn populate_all_function_like_reflections(codebase: &mut CodebaseReflection, populate_non_user_defined: bool) {
    let unpopulated_function_like_names = codebase
        .function_like_reflections
        .iter()
        .filter_map(|(name, reflection)| {
            if !populate_non_user_defined && !reflection.is_user_defined() {
                return None;
            }

            if !reflection.is_populated {
                Some(*name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for function_like_name in &unpopulated_function_like_names {
        populate_function_like_reflection(codebase, *function_like_name);
    }
}

#[inline]
fn populate_all_constant_reflections(codebase: &mut CodebaseReflection, populate_non_user_defined: bool) {
    let unpopulated_constant_names = codebase
        .constant_reflections
        .iter()
        .filter_map(|(name, reflection)| {
            if !populate_non_user_defined && !reflection.is_user_defined() {
                return None;
            }

            if !reflection.is_populated {
                Some(*name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for constant_name in &unpopulated_constant_names {
        populate_constant_reflection(codebase, *constant_name);
    }
}

#[inline]
fn populate_class_like_reflection(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    class_like_name: ClassLikeName,
    populate_non_user_defined: bool,
) {
    let Some(mut reflection) = codebase.class_like_reflections.remove(&class_like_name) else {
        return;
    };

    if !populate_non_user_defined && !reflection.is_user_defined() {
        codebase.class_like_reflections.insert(class_like_name, reflection);

        return;
    }

    if reflection.is_populated {
        codebase.class_like_reflections.insert(class_like_name, reflection);

        return;
    }

    implement_magic_interfaces(interner, codebase, &mut reflection);

    for property_id in reflection.properties.members.keys() {
        reflection.properties.appering_members.insert(*property_id, reflection.name);
        reflection.properties.declaring_members.insert(*property_id, reflection.name);
    }

    for method_id in reflection.methods.members.keys() {
        reflection.methods.appering_members.insert(*method_id, reflection.name);
        reflection.methods.declaring_members.insert(*method_id, reflection.name);
    }

    for trait_name in reflection.used_traits.clone() {
        populate_data_from_trait(interner, codebase, &mut reflection, trait_name, populate_non_user_defined);
    }

    if let Some(parent_classname) = reflection.inheritance.direct_extended_class {
        populate_data_from_parent_class(
            interner,
            codebase,
            &mut reflection,
            &parent_classname,
            populate_non_user_defined,
        );
    }

    for parent_interface in reflection.inheritance.direct_extended_interfaces.clone() {
        populate_data_from_parent_interface(
            interner,
            codebase,
            &mut reflection,
            &parent_interface,
            false,
            populate_non_user_defined,
        );
    }

    for parent_interface in reflection.inheritance.direct_implemented_interfaces.clone() {
        populate_data_from_parent_interface(
            interner,
            codebase,
            &mut reflection,
            &parent_interface,
            true,
            populate_non_user_defined,
        );
    }

    reflection.is_populated = true;

    codebase.class_like_reflections.insert(class_like_name, reflection);
}

#[inline]
fn implement_magic_interfaces(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
) {
    const UNIT_ENUM_INTERFACE: &str = "unitenum";
    const BACKED_ENUM_INTERFACE: &str = "backedenum";
    const STRINGABLE_INTERFACE: &str = "stringable";
    const TO_STRING_METHOD: &str = "__toString";

    let implement_interface = |reflection: &mut ClassLikeReflection, interface_name| {
        // Check if the interface is already implemented.
        if reflection.inheritance.implements_interface_with_name(interner, &interface_name) {
            return;
        }

        // The interface does not exist in the codebase, ignore it.
        let Some(interface) = codebase.get_interface(interner, &interface_name) else {
            return;
        };

        let interface = *interface.name.inner_unchecked();
        reflection.inheritance.direct_implemented_interfaces.insert(interface);
        reflection.inheritance.all_implemented_interfaces.insert(interface);
        reflection.inheritance.names.insert(interface.value, interface);
        reflection.inheritance.names.insert(interface_name, interface);
    };

    // Add auto-implemented interfaces for enums
    'enum_interface: {
        if !reflection.is_enum() {
            break 'enum_interface;
        }

        implement_interface(
            reflection,
            if reflection.backing_type.is_some() {
                interner.intern(BACKED_ENUM_INTERFACE)
            } else {
                interner.intern(UNIT_ENUM_INTERFACE)
            },
        );
    }

    'stringable_interface: {
        let to_string_method = interner.intern(TO_STRING_METHOD);

        if !reflection.methods.appering_members.contains_key(&to_string_method) {
            break 'stringable_interface;
        }

        implement_interface(reflection, interner.intern(STRINGABLE_INTERFACE));
    }
}

#[inline]
fn populate_data_from_parent_interface(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
    parent_name: &Name,
    implemented: bool,
    populate_non_user_defined: bool,
) {
    let parent_name_id = interner.lowered(&parent_name.value);
    let Some(interface_name) = codebase.class_like_names.get(&parent_name_id).cloned() else {
        report::report_missing_parent_interface(interner, reflection, parent_name, implemented);

        return;
    };

    populate_class_like_reflection(interner, codebase, interface_name, populate_non_user_defined);

    let Some(parent_reflection) = codebase.class_like_reflections.get_mut(&interface_name) else {
        report::report_missing_parent_interface(interner, reflection, parent_name, implemented);

        return;
    };

    if !parent_reflection.is_interface() {
        report::report_parent_not_interface(interner, reflection, parent_name, implemented);

        return;
    }

    if parent_reflection.inheritance.implements_interface(interner, reflection) {
        report::report_parent_interface_circular_reference(interner, reflection, parent_name, implemented);
    }

    for (constant_name, constant) in parent_reflection.constants.iter() {
        if reflection.constants.contains_key(constant_name) {
            continue;
        }

        reflection.constants.insert(*constant_name, constant.clone());
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
fn populate_data_from_parent_class(
    interner: &ThreadedInterner,
    codebase: &mut CodebaseReflection,
    reflection: &mut ClassLikeReflection,
    parent_name: &Name,
    populate_non_user_defined: bool,
) {
    let parent_name_id = interner.lowered(&parent_name.value);
    let Some(parent_classname) = codebase.class_like_names.get(&parent_name_id).cloned() else {
        report::report_missing_parent_class(interner, reflection, parent_name);

        return;
    };

    populate_class_like_reflection(interner, codebase, parent_classname, populate_non_user_defined);

    let Some(parent_reflection) = codebase.class_like_reflections.get_mut(&parent_classname) else {
        report::report_missing_parent_class(interner, reflection, parent_name);

        return;
    };

    if !parent_reflection.is_class() {
        report::report_parent_not_class(interner, reflection, parent_name);

        return;
    }

    if parent_reflection.is_final {
        report::report_parent_class_is_final(interner, reflection, parent_name);
    }

    if parent_reflection.is_readonly && !reflection.is_readonly {
        report::report_parent_class_is_readonly(interner, reflection, parent_name);
    } else if !parent_reflection.is_readonly && reflection.is_readonly {
        report::report_parent_class_is_not_readonly(interner, reflection, parent_name);
    }

    if parent_reflection.inheritance.extends_class(interner, reflection) {
        report::report_parent_class_circular_reference(interner, reflection, parent_name);

        return;
    }

    for extended_class in &parent_reflection.inheritance.all_extended_classes {
        if reflection.inheritance.all_extended_classes.contains(extended_class) {
            continue;
        }

        let identifier = extended_class.value;

        reflection.inheritance.all_extended_classes.insert(*extended_class);
        reflection.inheritance.names.insert(interner.lowered(&identifier), *extended_class);
    }

    for implemented_interface in &parent_reflection.inheritance.all_implemented_interfaces {
        if reflection.inheritance.all_implemented_interfaces.contains(implemented_interface) {
            continue;
        }

        let identifier = implemented_interface.value;

        reflection.inheritance.all_implemented_interfaces.insert(*implemented_interface);
        reflection.inheritance.names.insert(interner.lowered(&identifier), *implemented_interface);
    }

    for used_trait in &parent_reflection.used_traits {
        if reflection.used_traits.contains(used_trait) {
            continue;
        }

        reflection.used_traits.insert(*used_trait);
        reflection.used_trait_names.insert(used_trait.value, *used_trait);
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
    trait_name: Name,
    populate_non_user_defined: bool,
) {
    let Some(trait_class_like_name) = codebase.class_like_names.get(&trait_name.value).cloned() else {
        report::report_missing_trait(interner, reflection, &trait_name);

        return;
    };

    populate_class_like_reflection(interner, codebase, trait_class_like_name, populate_non_user_defined);

    let Some(trait_reflection) = codebase.class_like_reflections.get(&trait_class_like_name) else {
        report::report_missing_trait(interner, reflection, &trait_name);

        return;
    };

    if !trait_reflection.is_trait() {
        report::report_not_trait(interner, reflection, &trait_name);

        return;
    }

    if reflection.is_trait() {
        let class_like_name_id = &reflection.name.inner_unchecked().value;
        if trait_reflection.used_trait_names.contains_key(class_like_name_id) {
            report::report_trait_circular_reference(interner, reflection, &trait_name);
        }
    }

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
            if let Some(parent_property_storage) = parent_reflection.properties.members.get(property_name) {
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
            if let Some(overriding_property) = reflection.properties.members.get_mut(property_name) {
                overriding_property.is_overriding = true;
            }

            continue;
        }

        if !parent_is_trait {
            if let Some(parent_property_storage) = parent_reflection.properties.members.get(property_name) {
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
            if let Some(parent_property_storage) = parent_reflection.properties.members.get(property_name) {
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

        if reflection.methods.declaring_members.contains_key(method_name) {
            if let Some(overriding_method) = reflection.methods.members.get_mut(method_name) {
                overriding_method.is_overriding = true;
            }

            continue;
        }

        reflection.methods.declaring_members.insert(*method_name, *declaring_class);

        if !class_is_trait
            || !reflection.inheritance.require_extensions.contains(&parent_reflection.name.inner_unchecked().value)
        {
            reflection.methods.inheritable_members.insert(*method_name, *declaring_class);
        }
    }
}

#[inline]
fn populate_function_like_reflection(codebase: &mut CodebaseReflection, function_like_name: FunctionLikeName) {
    let Some(reflection) = codebase.function_like_reflections.get_mut(&function_like_name) else {
        return;
    };

    reflection.is_populated = true;
}

#[inline]
fn populate_constant_reflection(codebase: &mut CodebaseReflection, constant_name: Name) {
    let Some(reflection) = codebase.constant_reflections.get_mut(&constant_name) else {
        return;
    };

    reflection.is_populated = true;
}
