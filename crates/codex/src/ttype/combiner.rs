use std::collections::BTreeMap;

use ahash::HashSet;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_atom::concat_atom;

use crate::get_class_like;
use crate::inherits_class;
use crate::inherits_interface;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::symbol::SymbolKind;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::resource::TResource;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::combination::TypeCombination;
use crate::ttype::combine_union_types;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::object_comparator;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::utils::str_is_numeric;

pub fn combine(types: Vec<TAtomic>, codebase: &CodebaseMetadata, overwrite_empty_array: bool) -> Vec<TAtomic> {
    if types.len() == 1 || types.is_empty() {
        return types;
    }

    let mut combination = TypeCombination::new();

    for atomic in types {
        if let TAtomic::Derived(derived) = atomic {
            combination.derived_types.insert(derived);
            continue;
        }

        scrape_type_properties(atomic, &mut combination, codebase, overwrite_empty_array);
    }

    let is_falsy_mixed = combination.falsy_mixed.unwrap_or(false);
    let is_truthy_mixed = combination.truthy_mixed.unwrap_or(false);
    let is_nonnull_mixed = combination.nonnull_mixed.unwrap_or(false);

    if is_falsy_mixed || is_nonnull_mixed || combination.generic_mixed || is_truthy_mixed {
        return vec![TAtomic::Mixed(TMixed::new().with_is_non_null(is_nonnull_mixed).with_truthiness(
            if is_truthy_mixed && !is_falsy_mixed {
                TMixedTruthiness::Truthy
            } else if is_falsy_mixed && !is_truthy_mixed {
                TMixedTruthiness::Falsy
            } else {
                TMixedTruthiness::Undetermined
            },
        ))];
    } else if combination.has_mixed {
        return vec![TAtomic::Mixed(TMixed::new())];
    }

    if combination.is_simple() {
        if combination.value_types.contains_key(&atom("false")) {
            return vec![TAtomic::Scalar(TScalar::r#false())];
        }

        if combination.value_types.contains_key(&atom("true")) {
            return vec![TAtomic::Scalar(TScalar::r#true())];
        }

        return combination.value_types.into_values().collect();
    }

    if combination.value_types.contains_key(&atom("void")) {
        combination.value_types.remove(&atom("void"));

        if combination.value_types.contains_key(&atom("null")) {
            combination.value_types.insert(atom("null"), TAtomic::Null);
        }
    }

    if combination.value_types.contains_key(&atom("false")) && combination.value_types.contains_key(&atom("true")) {
        combination.value_types.remove(&atom("false"));
        combination.value_types.remove(&atom("true"));
        combination.value_types.insert(atom("bool"), TAtomic::Scalar(TScalar::bool()));
    }

    let mut new_types = Vec::new();
    for derived_type in combination.derived_types {
        new_types.push(TAtomic::Derived(derived_type));
    }

    if combination.resource {
        new_types.push(TAtomic::Resource(TResource { closed: None }));
    } else {
        match (combination.open_resource, combination.closed_resource) {
            (true, true) => {
                new_types.push(TAtomic::Resource(TResource { closed: None }));
            }
            (true, false) => {
                new_types.push(TAtomic::Resource(TResource { closed: Some(false) }));
            }
            (false, true) => {
                new_types.push(TAtomic::Resource(TResource { closed: Some(true) }));
            }
            _ => {
                // No resource type, do nothing
            }
        }
    }

    let mut added_array = false;
    if combination.has_keyed_array {
        added_array = true;

        new_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: if combination.keyed_array_entries.is_empty() {
                None
            } else {
                Some(combination.keyed_array_entries)
            },
            parameters: if let Some((k, v)) = combination.keyed_array_parameters {
                Some((Box::new(k), Box::new(v)))
            } else {
                None
            },
            non_empty: combination.keyed_array_always_filled,
        })));
    }

    if let Some(list_parameter) = combination.list_array_parameter {
        added_array = true;

        new_types.push(TAtomic::Array(TArray::List(TList {
            known_elements: if combination.list_array_entries.is_empty() {
                None
            } else {
                Some(combination.list_array_entries)
            },
            element_type: Box::new(list_parameter),
            non_empty: combination.list_array_always_filled,
            known_count: None,
        })));
    }

    if !added_array && combination.has_empty_array {
        new_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: None,
            parameters: None,
            non_empty: false,
        })));
    }

    for (_, (generic_type, generic_type_parameters)) in combination.object_type_params {
        let generic_object = TAtomic::Object(TObject::Named(
            TNamedObject::new(generic_type)
                .with_is_this(*combination.object_static.get(&generic_type).unwrap_or(&false))
                .with_type_parameters(Some(generic_type_parameters)),
        ));

        new_types.push(generic_object);
    }

    new_types.extend(
        combination
            .literal_strings
            .into_iter()
            .map(|s| TAtomic::Scalar(TScalar::literal_string(s)))
            .collect::<Vec<_>>(),
    );

    if combination.value_types.contains_key(&atom("string"))
        && combination.value_types.contains_key(&atom("float"))
        && combination.value_types.contains_key(&atom("bool"))
        && combination.integers.iter().any(|integer| integer.is_unspecified())
    {
        combination.integers.clear();
        combination.value_types.remove(&atom("string"));
        combination.value_types.remove(&atom("float"));
        combination.value_types.remove(&atom("bool"));

        new_types.push(TAtomic::Scalar(TScalar::Generic));
    }

    new_types.extend(combine_integers(combination.integers));

    new_types.extend(
        combination
            .literal_floats
            .into_iter()
            .map(|f| TAtomic::Scalar(TScalar::literal_float(f.into())))
            .collect::<Vec<_>>(),
    );

    for (enum_name, enum_case) in combination.enum_names {
        if combination.value_types.contains_key(&enum_name) {
            continue;
        }

        let enum_object = match enum_case {
            Some(case) => TAtomic::Object(TObject::new_enum_case(enum_name, case)),
            None => TAtomic::Object(TObject::new_enum(enum_name)),
        };

        combination.value_types.insert(enum_object.get_id(), enum_object);
    }

    let mut has_never = combination.value_types.contains_key(&atom("never"));

    let combination_value_type_count = combination.value_types.len();

    for (_, atomic) in combination.value_types {
        let tc = if has_never { 1 } else { 0 };
        if atomic.is_mixed()
            && combination.mixed_from_loop_isset.unwrap_or(false)
            && (combination_value_type_count > (tc + 1) || new_types.len() > tc)
        {
            continue;
        }

        if (atomic.is_never() || atomic.is_templated_as_never())
            && (combination_value_type_count > 1 || !new_types.is_empty())
        {
            has_never = true;
            continue;
        }

        new_types.push(atomic);
    }

    if new_types.is_empty() {
        if !has_never {
            unreachable!("No types to return, but no 'never' type found in combination.");
        }

        return vec![TAtomic::Never];
    }

    new_types
}

fn scrape_type_properties(
    atomic: TAtomic,
    combination: &mut TypeCombination,
    codebase: &CodebaseMetadata,
    overwrite_empty_array: bool,
) {
    if let TAtomic::Mixed(mixed) = atomic {
        if mixed.is_isset_from_loop() {
            // If we already have a broader mixed type, this specific one adds no info.
            if combination.generic_mixed {
                return; // Exit early, existing state is sufficient or broader
            }

            // If we haven't specifically recorded 'mixed_from_loop_isset' before, mark it now.
            if combination.mixed_from_loop_isset.is_none() {
                combination.mixed_from_loop_isset = Some(true);
            }

            combination.value_types.insert(atom("mixed"), atomic);

            return;
        }

        combination.has_mixed = true;

        if mixed.is_vanilla() {
            combination.falsy_mixed = Some(false);
            combination.truthy_mixed = Some(false);
            combination.mixed_from_loop_isset = Some(false);
            combination.generic_mixed = true;

            return;
        }

        if mixed.is_truthy() {
            if combination.generic_mixed {
                return;
            }

            combination.mixed_from_loop_isset = Some(false);

            if combination.falsy_mixed.unwrap_or(false) {
                combination.generic_mixed = true;
                combination.falsy_mixed = Some(false);
                return;
            }

            if combination.truthy_mixed.is_some() {
                return;
            }

            for existing_value_type in combination.value_types.values() {
                if !existing_value_type.is_truthy() {
                    combination.generic_mixed = true;
                    return;
                }
            }

            combination.truthy_mixed = Some(true);
        } else {
            combination.truthy_mixed = Some(false);
        }

        if mixed.is_falsy() {
            if combination.generic_mixed {
                return;
            }

            combination.mixed_from_loop_isset = Some(false);

            if combination.truthy_mixed.unwrap_or(false) {
                combination.generic_mixed = true;
                combination.truthy_mixed = Some(false);
                return;
            }

            if combination.falsy_mixed.is_some() {
                return;
            }

            for existing_value_type in combination.value_types.values() {
                if !existing_value_type.is_falsy() {
                    combination.generic_mixed = true;
                    return;
                }
            }

            combination.falsy_mixed = Some(true);
        } else {
            combination.falsy_mixed = Some(false);
        }

        if mixed.is_non_null() {
            if combination.generic_mixed {
                return;
            }

            combination.mixed_from_loop_isset = Some(false);

            if combination.value_types.contains_key(&atom("null")) {
                combination.generic_mixed = true;
                return;
            }

            if combination.falsy_mixed.unwrap_or(false) {
                combination.falsy_mixed = Some(false);
                combination.generic_mixed = true;
                return;
            }

            if combination.nonnull_mixed.is_some() {
                return;
            }

            combination.mixed_from_loop_isset = Some(false);
            combination.nonnull_mixed = Some(true);
        } else {
            combination.nonnull_mixed = Some(false);
        }

        return;
    }

    if combination.falsy_mixed.unwrap_or(false) {
        if !atomic.is_falsy() {
            combination.falsy_mixed = Some(false);
            combination.generic_mixed = true;
        }

        return;
    } else if combination.truthy_mixed.unwrap_or(false) {
        if !atomic.is_truthy() {
            combination.truthy_mixed = Some(false);
            combination.generic_mixed = true;
        }

        return;
    } else if combination.nonnull_mixed.unwrap_or(false) {
        if let TAtomic::Null = atomic {
            combination.nonnull_mixed = Some(false);
            combination.generic_mixed = true;
        }

        return;
    } else if combination.has_mixed {
        return;
    }

    // bool|false = bool
    if matches!(&atomic, TAtomic::Scalar(TScalar::Bool(bool)) if !bool.is_general())
        && combination.value_types.contains_key(&atom("bool"))
    {
        return;
    }

    if let TAtomic::Resource(TResource { closed }) = atomic {
        match closed {
            Some(closed) => match closed {
                true => {
                    combination.closed_resource = true;
                }
                false => {
                    combination.open_resource = true;
                }
            },
            None => {
                combination.resource = true;
            }
        }

        return;
    }

    // false|bool = bool
    if matches!(&atomic, TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general()) {
        combination.value_types.remove(&atom("false"));
        combination.value_types.remove(&atom("true"));
    }

    if let TAtomic::Array(array) = atomic {
        if overwrite_empty_array && array.is_empty() {
            combination.has_empty_array = true;

            return;
        }

        match array {
            TArray::List(TList { element_type, known_elements, non_empty, known_count }) => {
                if non_empty {
                    if let Some(ref mut existing_counts) = combination.list_array_counts {
                        if let Some(known_count) = known_count {
                            existing_counts.insert(known_count);
                        } else {
                            combination.list_array_counts = None;
                        }
                    }

                    combination.list_array_sometimes_filled = true;
                } else {
                    combination.list_array_always_filled = false;
                }

                if let Some(known_elements) = known_elements {
                    let mut has_defined_keys = false;

                    for (candidate_element_index, (candidate_optional, candidate_element_type)) in known_elements {
                        let existing_entry = combination.list_array_entries.get(&candidate_element_index);

                        let new_entry = if let Some((existing_optional, existing_type)) = existing_entry {
                            (
                                *existing_optional || candidate_optional,
                                combine_union_types(
                                    existing_type,
                                    &candidate_element_type,
                                    codebase,
                                    overwrite_empty_array,
                                ),
                            )
                        } else {
                            (
                                candidate_optional,
                                if let Some(ref mut existing_value_parameter) = combination.list_array_parameter {
                                    if !existing_value_parameter.is_never() {
                                        *existing_value_parameter = combine_union_types(
                                            existing_value_parameter,
                                            &candidate_element_type,
                                            codebase,
                                            overwrite_empty_array,
                                        );

                                        continue;
                                    }

                                    candidate_element_type
                                } else {
                                    candidate_element_type
                                },
                            )
                        };

                        combination.list_array_entries.insert(candidate_element_index, new_entry);

                        if !candidate_optional {
                            has_defined_keys = true;
                        }
                    }

                    if !has_defined_keys {
                        combination.list_array_always_filled = false;
                    }
                } else if !overwrite_empty_array {
                    if element_type.is_never() {
                        for (_, (pu, _)) in combination.list_array_entries.iter_mut() {
                            *pu = true;
                        }
                    } else {
                        for (_, entry_type) in combination.list_array_entries.values() {
                            if let Some(ref mut existing_value_param) = combination.list_array_parameter {
                                *existing_value_param = combine_union_types(
                                    existing_value_param,
                                    entry_type,
                                    codebase,
                                    overwrite_empty_array,
                                );
                            }
                        }

                        combination.list_array_entries = BTreeMap::new();
                    }
                }

                combination.list_array_parameter = if let Some(ref existing_type) = combination.list_array_parameter {
                    Some(combine_union_types(existing_type, &element_type, codebase, overwrite_empty_array))
                } else {
                    Some((*element_type).clone())
                };
            }
            TArray::Keyed(TKeyedArray { parameters, known_items, non_empty, .. }) => {
                let had_previous_keyed_array = combination.has_keyed_array;
                combination.has_keyed_array = true;

                if non_empty {
                    combination.keyed_array_sometimes_filled = true;
                } else {
                    combination.keyed_array_always_filled = false;
                }

                if let Some(known_items) = known_items {
                    let has_existing_entries = !combination.keyed_array_entries.is_empty() || had_previous_keyed_array;
                    let mut possibly_undefined_entries =
                        combination.keyed_array_entries.keys().cloned().collect::<HashSet<_>>();

                    let mut has_defined_keys = false;

                    for (candidate_item_name, (cu, candidate_item_type)) in known_items {
                        if let Some((eu, existing_type)) = combination.keyed_array_entries.get_mut(&candidate_item_name)
                        {
                            if cu {
                                *eu = true;
                            }
                            if &candidate_item_type != existing_type {
                                *existing_type = combine_union_types(
                                    existing_type,
                                    &candidate_item_type,
                                    codebase,
                                    overwrite_empty_array,
                                );
                            }
                        } else {
                            let new_item_value_type =
                                if let Some((ref mut existing_key_param, ref mut existing_value_param)) =
                                    combination.keyed_array_parameters
                                {
                                    adjust_keyed_array_parameters(
                                        existing_value_param,
                                        &candidate_item_type,
                                        codebase,
                                        overwrite_empty_array,
                                        &candidate_item_name,
                                        existing_key_param,
                                    );

                                    continue;
                                } else {
                                    let new_type = candidate_item_type.clone();
                                    (has_existing_entries || cu, new_type)
                                };

                            combination.keyed_array_entries.insert(candidate_item_name, new_item_value_type);
                        };

                        possibly_undefined_entries.remove(&candidate_item_name);

                        if !cu {
                            has_defined_keys = true;
                        }
                    }

                    if !has_defined_keys {
                        combination.keyed_array_always_filled = false;
                    }

                    for possibly_undefined_type_key in possibly_undefined_entries {
                        let possibly_undefined_type =
                            combination.keyed_array_entries.get_mut(&possibly_undefined_type_key);
                        if let Some((pu, _)) = possibly_undefined_type {
                            *pu = true;
                        }
                    }
                } else if !overwrite_empty_array {
                    if match &parameters {
                        Some((_, value_param)) => value_param.is_never(),
                        None => true,
                    } {
                        for (_, (tu, _)) in combination.keyed_array_entries.iter_mut() {
                            *tu = true;
                        }
                    } else {
                        for (key, (_, entry_type)) in &combination.keyed_array_entries {
                            if let Some((ref mut existing_key_param, ref mut existing_value_param)) =
                                combination.keyed_array_parameters
                            {
                                adjust_keyed_array_parameters(
                                    existing_value_param,
                                    entry_type,
                                    codebase,
                                    overwrite_empty_array,
                                    key,
                                    existing_key_param,
                                );
                            }
                        }

                        combination.keyed_array_entries = BTreeMap::new();
                    }
                }

                combination.keyed_array_parameters = match (&combination.keyed_array_parameters, parameters) {
                    (None, None) => None,
                    (Some(existing_types), None) => Some(existing_types.clone()),
                    (None, Some(params)) => Some(((*params.0).clone(), (*params.1).clone())),
                    (Some(existing_types), Some(params)) => Some((
                        combine_union_types(&existing_types.0, &params.0, codebase, overwrite_empty_array),
                        combine_union_types(&existing_types.1, &params.1, codebase, overwrite_empty_array),
                    )),
                };
            }
        }

        return;
    }

    // this probably won't ever happen, but the object top type
    // can eliminate variants
    if let TAtomic::Object(TObject::Any) = atomic {
        combination.has_object_top_type = true;
        combination.value_types.retain(|_, t| !matches!(t, TAtomic::Object(TObject::Named(_))));
        combination.value_types.insert(atomic.get_id(), atomic);

        return;
    }

    if let TAtomic::Object(TObject::Named(named_object)) = &atomic {
        if let Some(object_static) = combination.object_static.get(named_object.get_name_ref()) {
            if *object_static && !named_object.is_this() {
                combination.object_static.insert(named_object.get_name(), false);
            }
        } else {
            combination.object_static.insert(named_object.get_name(), named_object.is_this());
        }
    }

    if let TAtomic::Object(TObject::Named(named_object)) = &atomic {
        let fq_class_name = named_object.get_name();
        if let Some(type_parameters) = named_object.get_type_parameters() {
            let object_type_key = get_combiner_key(&fq_class_name, type_parameters, codebase);

            if let Some((_, existing_type_params)) = combination.object_type_params.get(&object_type_key) {
                let mut new_type_parameters = Vec::with_capacity(type_parameters.len());
                for (i, type_param) in type_parameters.iter().enumerate() {
                    if let Some(existing_type_param) = existing_type_params.get(i) {
                        new_type_parameters.insert(
                            i,
                            combine_union_types(existing_type_param, type_param, codebase, overwrite_empty_array),
                        );
                    }
                }

                combination.object_type_params.insert(object_type_key, (fq_class_name, new_type_parameters));
            } else {
                combination.object_type_params.insert(object_type_key, (fq_class_name, type_parameters.to_vec()));
            }

            return;
        }
    }

    if let TAtomic::Object(TObject::Enum(enum_object)) = atomic {
        combination.enum_names.insert((enum_object.get_name(), enum_object.get_case()));

        return;
    }

    if let TAtomic::Object(TObject::Named(named_object)) = &atomic {
        let fq_class_name = named_object.get_name();
        let intersection_types = named_object.get_intersection_types();

        if !combination.has_object_top_type {
            if combination.value_types.contains_key(&atomic.get_id()) {
                return;
            }
        } else {
            return;
        }

        let symbol_type = if let Some(symbol_type) = codebase.symbols.get_kind(&fq_class_name) {
            symbol_type
        } else {
            combination.value_types.insert(atomic.get_id(), atomic);
            return;
        };

        if !matches!(symbol_type, SymbolKind::Class | SymbolKind::Enum | SymbolKind::Interface) {
            combination.value_types.insert(atomic.get_id(), atomic);
            return;
        }

        let is_class = matches!(symbol_type, SymbolKind::Class);
        let is_interface = matches!(symbol_type, SymbolKind::Interface);

        let mut types_to_remove: Vec<Atom> = Vec::new();

        for (key, existing_type) in &combination.value_types {
            if let TAtomic::Object(TObject::Named(existing_object)) = &existing_type {
                let existing_name = existing_object.get_name();

                if intersection_types.is_some() || existing_object.has_intersection_types() {
                    if object_comparator::is_shallowly_contained_by(
                        codebase,
                        existing_type,
                        &atomic,
                        false,
                        &mut ComparisonResult::new(),
                    ) {
                        types_to_remove.push(existing_name);
                        continue;
                    }

                    if object_comparator::is_shallowly_contained_by(
                        codebase,
                        &atomic,
                        existing_type,
                        false,
                        &mut ComparisonResult::new(),
                    ) {
                        return;
                    }

                    continue;
                }

                let Some(existing_symbol_kind) = codebase.symbols.get_kind(existing_object.get_name_ref()) else {
                    continue;
                };

                if matches!(existing_symbol_kind, SymbolKind::Class) {
                    // remove subclasses
                    if is_instance_of(codebase, &existing_name, &fq_class_name) {
                        types_to_remove.push(*key);
                        continue;
                    }

                    if is_class {
                        // if covered by a parent class
                        if inherits_class(codebase, &fq_class_name, &existing_name) {
                            return;
                        }
                    } else if is_interface {
                        // if covered by a parent class
                        if inherits_interface(codebase, &fq_class_name, &existing_name) {
                            return;
                        }
                    }
                } else if matches!(existing_symbol_kind, SymbolKind::Interface) {
                    if inherits_interface(codebase, &existing_name, &fq_class_name) {
                        types_to_remove.push(existing_name);
                        continue;
                    }

                    if (is_class || is_interface) && inherits_interface(codebase, &fq_class_name, &existing_name) {
                        return;
                    }
                }
            }
        }

        combination.value_types.insert(atomic.get_id(), atomic);

        for type_key in types_to_remove {
            combination.value_types.remove(&type_key);
        }

        return;
    }

    if let TAtomic::Scalar(TScalar::Generic) = atomic {
        combination.literal_strings = AtomSet::default();
        combination.integers = HashSet::default();
        combination.literal_floats = HashSet::default();
        combination.value_types.retain(|k, _| {
            k != "string"
                && k != "bool"
                && k != "false"
                && k != "true"
                && k != "float"
                && k != "numeric"
                && k != "array-key"
        });

        combination.value_types.insert(atomic.get_id(), atomic);
        return;
    }

    if let TAtomic::Scalar(TScalar::ArrayKey) = atomic {
        if combination.value_types.contains_key(&atom("scalar")) {
            return;
        }

        combination.literal_strings = AtomSet::default();
        combination.integers = HashSet::default();
        combination.value_types.retain(|k, _| k != "string" && k != "int");
        combination.value_types.insert(atomic.get_id(), atomic);

        return;
    }

    if let TAtomic::Scalar(TScalar::String(_) | TScalar::Integer(_)) = atomic
        && (combination.value_types.contains_key(&atom("scalar"))
            || combination.value_types.contains_key(&atom("array-key")))
    {
        return;
    }

    if let TAtomic::Scalar(TScalar::Float(_) | TScalar::Integer(_)) = atomic
        && (combination.value_types.contains_key(&atom("numeric"))
            || combination.value_types.contains_key(&atom("scalar")))
    {
        return;
    }

    if let TAtomic::Scalar(TScalar::String(mut string_scalar)) = atomic {
        if let Some(existing_string_type) = combination.value_types.get_mut(&atom("string")) {
            if let TAtomic::Scalar(TScalar::String(existing_string_type)) = existing_string_type {
                *existing_string_type = combine_string_scalars(existing_string_type, string_scalar);
            };
        } else if let Some(value) =
            string_scalar.get_known_literal_value().filter(|_| combination.literal_strings.len() <= 20)
        {
            combination.literal_strings.insert(atom(value));
        } else {
            if string_scalar.is_truthy || string_scalar.is_non_empty || string_scalar.is_numeric {
                for value in &combination.literal_strings {
                    if value.is_empty() {
                        string_scalar.is_non_empty = false;
                        string_scalar.is_truthy = false;
                        string_scalar.is_numeric = false;
                        break;
                    } else if value == "0" {
                        string_scalar.is_truthy = false;
                    }

                    string_scalar.is_numeric = string_scalar.is_numeric && str_is_numeric(value);
                }
            }

            combination.value_types.insert(atom("string"), TAtomic::Scalar(TScalar::String(string_scalar)));
            combination.literal_strings = AtomSet::default();
        }

        return;
    }

    if let TAtomic::Scalar(TScalar::Integer(integer)) = &atomic {
        combination.integers.insert(*integer);

        return;
    }

    if let TAtomic::Scalar(TScalar::Float(float_scalar)) = &atomic {
        if combination.value_types.contains_key(&atom("float")) {
            return;
        }

        match float_scalar.value.as_ref() {
            Some(literal_value) if combination.literal_floats.len() <= 20 => {
                combination.literal_floats.insert(*literal_value);
            }
            _ => {
                combination.literal_floats = HashSet::default();
                combination.value_types.insert(atom("float"), atomic);
            }
        }

        return;
    }

    combination.value_types.insert(atomic.get_id(), atomic);
}

fn adjust_keyed_array_parameters(
    existing_value_param: &mut TUnion,
    entry_type: &TUnion,
    codebase: &CodebaseMetadata,
    overwrite_empty_array: bool,
    key: &ArrayKey,
    existing_key_param: &mut TUnion,
) {
    *existing_value_param = combine_union_types(existing_value_param, entry_type, codebase, overwrite_empty_array);
    let new_key_type = key.to_union();
    *existing_key_param = combine_union_types(existing_key_param, &new_key_type, codebase, overwrite_empty_array);
}

fn get_combiner_key(name: &Atom, type_params: &[TUnion], codebase: &CodebaseMetadata) -> Atom {
    let covariants = if let Some(class_like_metadata) = get_class_like(codebase, name) {
        &class_like_metadata.template_variance
    } else {
        return *name;
    };

    concat_atom!(
        name.as_ref(),
        "<",
        type_params
            .iter()
            .enumerate()
            .map(|(i, tunion)| {
                if let Some(Variance::Covariant) = covariants.get(&i) { "*" } else { tunion.get_id().as_str() }
            },)
            .collect::<Vec<_>>()
            .join(", ")
            .as_str(),
        ">",
    )
}

fn combine_string_scalars(s1: &TString, s2: TString) -> TString {
    TString {
        literal: match (&s1.literal, s2.literal) {
            (Some(TStringLiteral::Value(v1)), Some(TStringLiteral::Value(v2))) => {
                if v1 == &v2 {
                    Some(TStringLiteral::Value(v2))
                } else {
                    Some(TStringLiteral::Unspecified)
                }
            }
            (Some(TStringLiteral::Unspecified), Some(_)) | (Some(_), Some(TStringLiteral::Unspecified)) => {
                Some(TStringLiteral::Unspecified)
            }
            _ => None,
        },
        is_numeric: s1.is_numeric && s2.is_numeric,
        is_truthy: s1.is_truthy && s2.is_truthy,
        is_non_empty: s1.is_non_empty && s2.is_non_empty,
        is_lowercase: s1.is_lowercase && s2.is_lowercase,
    }
}

fn combine_integers(types: HashSet<TInteger>) -> Vec<TAtomic> {
    let types = types.into_iter().collect::<Vec<_>>();

    TInteger::combine(&types).into_iter().map(|tint| TAtomic::Scalar(TScalar::Integer(tint))).collect()
}
