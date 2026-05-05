use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::LazyLock;

use foldhash::HashSet;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;

static ATOM_FALSE: LazyLock<Atom> = LazyLock::new(|| atom("false"));
static ATOM_TRUE: LazyLock<Atom> = LazyLock::new(|| atom("true"));
static ATOM_BOOL: LazyLock<Atom> = LazyLock::new(|| atom("bool"));
static ATOM_VOID: LazyLock<Atom> = LazyLock::new(|| atom("void"));
static ATOM_NULL: LazyLock<Atom> = LazyLock::new(|| atom("null"));
static ATOM_STRING: LazyLock<Atom> = LazyLock::new(|| atom("string"));
static ATOM_FLOAT: LazyLock<Atom> = LazyLock::new(|| atom("float"));
static ATOM_INT: LazyLock<Atom> = LazyLock::new(|| atom("int"));
static ATOM_MIXED: LazyLock<Atom> = LazyLock::new(|| atom("mixed"));
static ATOM_SCALAR: LazyLock<Atom> = LazyLock::new(|| atom("scalar"));
static ATOM_ARRAY_KEY: LazyLock<Atom> = LazyLock::new(|| atom("array-key"));
static ATOM_NUMERIC: LazyLock<Atom> = LazyLock::new(|| atom("numeric"));
static ATOM_NEVER: LazyLock<Atom> = LazyLock::new(|| atom("never"));

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
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringCasing;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::combination::CombinationFlags;
use crate::ttype::combination::TypeCombination;
use crate::ttype::combine_union_types;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::array_comparator::is_array_contained_by_array;
use crate::ttype::comparator::object_comparator;
use crate::ttype::comparator::union_comparator;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::utils::str_is_numeric;

/// Default maximum number of sealed arrays to track before generalizing.
///
/// When combining array types, sealed arrays (arrays with known literal elements)
/// are accumulated for later comparison. If the number of sealed arrays exceeds
/// this threshold, they are immediately generalized to prevent O(n²) complexity
/// in `finalize_sealed_arrays` and excessive memory usage.
pub const DEFAULT_ARRAY_COMBINATION_THRESHOLD: u16 = 32;

/// Default maximum number of literal strings to track before generalizing to string.
///
/// When combining types with many different literal string values, tracking each
/// literal individually causes O(n) memory and O(n²) comparison time.
/// Once the threshold is exceeded, we generalize to the base string type.
pub const DEFAULT_STRING_COMBINATION_THRESHOLD: u16 = 128;

/// Default maximum number of literal integers to track before generalizing to int.
///
/// When combining types with many different literal integer values, tracking each
/// literal individually causes O(n) memory and O(n²) comparison time.
/// Once the threshold is exceeded, we generalize to the base int type.
pub const DEFAULT_INTEGER_COMBINATION_THRESHOLD: u16 = 128;

/// Options for controlling type combination behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombinerOptions {
    /// When true, empty arrays are overwritten by non-empty arrays during combination.
    pub overwrite_empty_array: bool,
    /// Maximum number of sealed arrays to track before generalizing.
    pub array_combination_threshold: u16,
    /// Maximum number of literal strings to track before generalizing to string.
    pub string_combination_threshold: u16,
    /// Maximum number of literal integers to track before generalizing to int.
    pub integer_combination_threshold: u16,
}

impl Default for CombinerOptions {
    fn default() -> Self {
        Self {
            overwrite_empty_array: false,
            array_combination_threshold: DEFAULT_ARRAY_COMBINATION_THRESHOLD,
            string_combination_threshold: DEFAULT_STRING_COMBINATION_THRESHOLD,
            integer_combination_threshold: DEFAULT_INTEGER_COMBINATION_THRESHOLD,
        }
    }
}

impl CombinerOptions {
    /// Create options with overwrite_empty_array set to true.
    #[inline]
    #[must_use]
    pub fn with_overwrite_empty_array(mut self) -> Self {
        self.overwrite_empty_array = true;
        self
    }

    /// Create options with a custom array combination threshold.
    #[inline]
    #[must_use]
    pub fn with_array_combination_threshold(mut self, threshold: u16) -> Self {
        self.array_combination_threshold = threshold;
        self
    }

    /// Create options with a custom string combination threshold.
    #[inline]
    #[must_use]
    pub fn with_string_combination_threshold(mut self, threshold: u16) -> Self {
        self.string_combination_threshold = threshold;
        self
    }

    /// Create options with a custom integer combination threshold.
    #[inline]
    #[must_use]
    pub fn with_integer_combination_threshold(mut self, threshold: u16) -> Self {
        self.integer_combination_threshold = threshold;
        self
    }
}

pub fn combine(types: Vec<TAtomic>, codebase: &CodebaseMetadata, options: CombinerOptions) -> Vec<TAtomic> {
    if types.is_empty() {
        debug_assert!(false, "combine() received an empty Vec; this is a caller bug");

        return vec![TAtomic::Never];
    }

    if types.len() == 1 {
        return types;
    }

    let mut combination = TypeCombination::new();

    for atomic in types {
        if let TAtomic::Derived(derived) = atomic {
            combination.derived_types.insert(derived);
            continue;
        }

        scrape_type_properties(atomic, &mut combination, codebase, options);
    }

    combination.integers.sort_unstable();
    combination.integers.dedup();
    combination.literal_floats.sort_unstable();
    combination.literal_floats.dedup();

    finalize_sealed_arrays(&mut combination.sealed_arrays, codebase);

    let is_falsy_mixed = combination.flags.falsy_mixed().unwrap_or(false);
    let is_truthy_mixed = combination.flags.truthy_mixed().unwrap_or(false);
    let is_nonnull_mixed = combination.flags.nonnull_mixed().unwrap_or(false);

    if is_falsy_mixed
        || is_nonnull_mixed
        || combination.flags.contains(CombinationFlags::GENERIC_MIXED)
        || is_truthy_mixed
    {
        return vec![TAtomic::Mixed(TMixed::new().with_is_non_null(is_nonnull_mixed).with_truthiness(
            if is_truthy_mixed && !is_falsy_mixed {
                TMixedTruthiness::Truthy
            } else if is_falsy_mixed && !is_truthy_mixed {
                TMixedTruthiness::Falsy
            } else {
                TMixedTruthiness::Undetermined
            },
        ))];
    } else if combination.flags.contains(CombinationFlags::HAS_MIXED) {
        return vec![TAtomic::Mixed(TMixed::new())];
    }

    if combination.is_simple() {
        if combination.value_types.contains_key(&*ATOM_FALSE) {
            return vec![TAtomic::Scalar(TScalar::r#false())];
        }

        if combination.value_types.contains_key(&*ATOM_TRUE) {
            return vec![TAtomic::Scalar(TScalar::r#true())];
        }

        return combination.value_types.into_values().collect();
    }

    if combination.value_types.remove(&*ATOM_VOID).is_some() {
        combination.value_types.insert(*ATOM_NULL, TAtomic::Null);
    }

    if combination.value_types.contains_key(&*ATOM_FALSE) && combination.value_types.contains_key(&*ATOM_TRUE) {
        combination.value_types.remove(&*ATOM_FALSE);
        combination.value_types.remove(&*ATOM_TRUE);
        combination.value_types.insert(*ATOM_BOOL, TAtomic::Scalar(TScalar::bool()));
    }

    let estimated_capacity = combination.derived_types.len()
        + combination.integers.len().min(10)
        + combination.literal_floats.len()
        + combination.enum_names.len()
        + combination.value_types.len()
        + combination.sealed_arrays.len()
        + 5;

    let mut new_types = Vec::with_capacity(estimated_capacity);
    for derived_type in combination.derived_types {
        new_types.push(TAtomic::Derived(derived_type));
    }

    if combination.flags.contains(CombinationFlags::RESOURCE) {
        new_types.push(TAtomic::Resource(TResource { closed: None }));
    } else {
        let open = combination.flags.contains(CombinationFlags::OPEN_RESOURCE);
        let closed = combination.flags.contains(CombinationFlags::CLOSED_RESOURCE);
        match (open, closed) {
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

    let mut arrays = vec![];

    if combination.flags.contains(CombinationFlags::HAS_KEYED_ARRAY) {
        arrays.push(TArray::Keyed(TKeyedArray {
            known_items: if combination.keyed_array_entries.is_empty() {
                None
            } else {
                Some(combination.keyed_array_entries)
            },
            parameters: if let Some((k, v)) = combination.keyed_array_parameters {
                Some((Arc::new(k), Arc::new(v)))
            } else {
                None
            },
            non_empty: combination.flags.contains(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED),
        }));
    }

    if let Some(list_parameter) = combination.list_array_parameter {
        arrays.push(TArray::List(TList {
            known_elements: if combination.list_array_entries.is_empty() {
                None
            } else {
                Some(combination.list_array_entries)
            },
            element_type: Arc::new(list_parameter),
            non_empty: combination.flags.contains(CombinationFlags::LIST_ARRAY_ALWAYS_FILLED),
            known_count: None,
        }));
    }

    for array in combination.sealed_arrays {
        arrays.push(array);
    }

    if arrays.is_empty() && combination.flags.contains(CombinationFlags::HAS_EMPTY_ARRAY) {
        arrays.push(TArray::Keyed(TKeyedArray { known_items: None, parameters: None, non_empty: false }));
    }

    new_types.extend(arrays.into_iter().map(TAtomic::Array));

    for (_, (generic_type, generic_type_parameters)) in combination.object_type_params {
        let generic_object = TAtomic::Object(TObject::Named(
            TNamedObject::new(generic_type)
                .with_is_static(*combination.object_static.get(&generic_type).unwrap_or(&false))
                .with_type_parameters(Some(generic_type_parameters)),
        ));

        new_types.push(generic_object);
    }

    new_types.extend(combination.literal_strings.into_iter().map(|s| TAtomic::Scalar(TScalar::literal_string(s))));

    if combination.value_types.contains_key(&*ATOM_STRING)
        && combination.value_types.contains_key(&*ATOM_FLOAT)
        && combination.value_types.contains_key(&*ATOM_BOOL)
        && combination.integers.iter().any(super::atomic::scalar::int::TInteger::is_unspecified)
    {
        combination.integers.clear();
        combination.value_types.remove(&*ATOM_STRING);
        combination.value_types.remove(&*ATOM_FLOAT);
        combination.value_types.remove(&*ATOM_BOOL);

        new_types.push(TAtomic::Scalar(TScalar::Generic));
    }

    new_types.extend(TInteger::combine(combination.integers));
    new_types.extend(combination.literal_floats.into_iter().map(|f| TAtomic::Scalar(TScalar::literal_float(f.into()))));

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

    let mut has_never = combination.value_types.contains_key(&*ATOM_NEVER);

    let combination_value_type_count = combination.value_types.len();
    let mixed_from_loop_isset = combination.flags.mixed_from_loop_isset().unwrap_or(false);

    for (_, atomic) in combination.value_types {
        let tc = usize::from(has_never);
        if atomic.is_mixed()
            && mixed_from_loop_isset
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
        debug_assert!(has_never, "combine(): empty result without a `never` atomic in the combination");

        return vec![TAtomic::Never];
    }

    new_types
}

fn finalize_sealed_arrays(arrays: &mut Vec<TArray>, codebase: &CodebaseMetadata) {
    if arrays.len() <= 1 {
        return;
    }

    arrays.sort_unstable_by_key(|a| match a {
        TArray::List(list) => list.known_elements.as_ref().map_or(0, std::collections::BTreeMap::len),
        TArray::Keyed(keyed) => keyed.known_items.as_ref().map_or(0, std::collections::BTreeMap::len),
    });

    let mut keep = vec![true; arrays.len()];

    for i in 0..arrays.len() {
        if !keep[i] {
            continue;
        }

        for j in (i + 1)..arrays.len() {
            if !keep[j] {
                continue;
            }

            if is_array_contained_by_array(codebase, &arrays[i], &arrays[j], false, &mut ComparisonResult::new()) {
                keep[i] = false;
                break;
            }

            if is_array_contained_by_array(codebase, &arrays[j], &arrays[i], false, &mut ComparisonResult::new()) {
                keep[j] = false;
            }
        }
    }

    let mut write = 0;
    for (read, item) in keep.iter().enumerate().take(arrays.len()) {
        if *item {
            if write != read {
                arrays.swap(write, read);
            }

            write += 1;
        }
    }

    arrays.truncate(write);
}

fn scrape_type_properties(
    atomic: TAtomic,
    combination: &mut TypeCombination,
    codebase: &CodebaseMetadata,
    options: CombinerOptions,
) {
    if combination.flags.contains(CombinationFlags::GENERIC_MIXED) {
        return;
    }

    if let TAtomic::Mixed(mixed) = atomic {
        if mixed.is_isset_from_loop() {
            if combination.flags.contains(CombinationFlags::GENERIC_MIXED) {
                return; // Exit early, existing state is sufficient or broader
            }

            if combination.flags.mixed_from_loop_isset().is_none() {
                combination.flags.set_mixed_from_loop_isset(Some(true));
            }

            combination.value_types.insert(*ATOM_MIXED, atomic);

            return;
        }

        combination.flags.insert(CombinationFlags::HAS_MIXED);

        if mixed.is_vanilla() {
            combination.flags.set_falsy_mixed(Some(false));
            combination.flags.set_truthy_mixed(Some(false));
            combination.flags.set_mixed_from_loop_isset(Some(false));
            combination.flags.insert(CombinationFlags::GENERIC_MIXED);

            return;
        }

        if mixed.is_truthy() {
            if combination.flags.contains(CombinationFlags::GENERIC_MIXED) {
                return;
            }

            combination.flags.set_mixed_from_loop_isset(Some(false));

            if combination.flags.falsy_mixed().unwrap_or(false) {
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                combination.flags.set_falsy_mixed(Some(false));
                return;
            }

            if combination.flags.truthy_mixed().is_some() {
                return;
            }

            let has_non_truthy = combination.value_types.values().any(|v| !v.is_truthy())
                || combination.literal_strings.iter().any(|s| s.is_empty() || s.as_str() == "0");

            if has_non_truthy {
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                return;
            }

            combination.flags.set_truthy_mixed(Some(true));
        } else {
            combination.flags.set_truthy_mixed(Some(false));
        }

        if mixed.is_falsy() {
            if combination.flags.contains(CombinationFlags::GENERIC_MIXED) {
                return;
            }

            combination.flags.set_mixed_from_loop_isset(Some(false));

            if combination.flags.truthy_mixed().unwrap_or(false) {
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                combination.flags.set_truthy_mixed(Some(false));
                return;
            }

            if combination.flags.falsy_mixed().is_some() {
                return;
            }

            let has_non_falsy = combination.value_types.values().any(|v| !v.is_falsy())
                || combination.literal_strings.iter().any(|s| !s.is_empty() && s.as_str() != "0");

            if has_non_falsy {
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                return;
            }

            combination.flags.set_falsy_mixed(Some(true));
        } else {
            combination.flags.set_falsy_mixed(Some(false));
        }

        if mixed.is_non_null() {
            if combination.flags.contains(CombinationFlags::GENERIC_MIXED) {
                return;
            }

            combination.flags.set_mixed_from_loop_isset(Some(false));

            if combination.value_types.contains_key(&*ATOM_NULL) {
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                return;
            }

            if combination.flags.falsy_mixed().unwrap_or(false) {
                combination.flags.set_falsy_mixed(Some(false));
                combination.flags.insert(CombinationFlags::GENERIC_MIXED);
                return;
            }

            if combination.flags.nonnull_mixed().is_some() {
                return;
            }

            combination.flags.set_mixed_from_loop_isset(Some(false));
            combination.flags.set_nonnull_mixed(Some(true));
        } else {
            combination.flags.set_nonnull_mixed(Some(false));
        }

        return;
    }

    if combination.flags.falsy_mixed().unwrap_or(false) {
        if !atomic.is_falsy() {
            combination.flags.set_falsy_mixed(Some(false));
            combination.flags.insert(CombinationFlags::GENERIC_MIXED);
        }

        return;
    }

    if combination.flags.truthy_mixed().unwrap_or(false) {
        if !atomic.is_truthy() {
            combination.flags.set_truthy_mixed(Some(false));
            combination.flags.insert(CombinationFlags::GENERIC_MIXED);
        }

        return;
    }

    if combination.flags.nonnull_mixed().unwrap_or(false) {
        if atomic == TAtomic::Null {
            combination.flags.set_nonnull_mixed(Some(false));
            combination.flags.insert(CombinationFlags::GENERIC_MIXED);
        }

        return;
    }

    if combination.flags.contains(CombinationFlags::HAS_MIXED) {
        return;
    }

    if matches!(&atomic, TAtomic::Scalar(TScalar::Bool(bool)) if !bool.is_general())
        && combination.value_types.contains_key(&*ATOM_BOOL)
    {
        return;
    }

    if let TAtomic::Resource(TResource { closed }) = atomic {
        match closed {
            Some(closed) => {
                if closed {
                    combination.flags.insert(CombinationFlags::CLOSED_RESOURCE);
                } else {
                    combination.flags.insert(CombinationFlags::OPEN_RESOURCE);
                }
            }
            None => {
                combination.flags.insert(CombinationFlags::RESOURCE);
            }
        }

        return;
    }

    if matches!(&atomic, TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general()) {
        combination.value_types.remove(&*ATOM_FALSE);
        combination.value_types.remove(&*ATOM_TRUE);
    }

    if let TAtomic::Array(array) = atomic {
        if options.overwrite_empty_array && array.is_empty() {
            combination.flags.insert(CombinationFlags::HAS_EMPTY_ARRAY);

            return;
        }

        // Accumulate sealed arrays for later comparison, but only up to a threshold.
        // Once we exceed the threshold, we let the arrays fall through to be processed
        // immediately, which generalizes them and prevents O(n²) complexity.
        if !array.is_empty()
            && array.is_sealed()
            && combination.list_array_parameter.is_some()
            && !combination.flags.contains(CombinationFlags::HAS_KEYED_ARRAY)
            && combination.sealed_arrays.len() < options.array_combination_threshold as usize
        {
            combination.sealed_arrays.push(array);
            return;
        }

        let mut sealed_arrays = vec![];
        std::mem::swap(&mut sealed_arrays, &mut combination.sealed_arrays);
        for array in std::iter::once(array).chain(sealed_arrays) {
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

                        combination.flags.insert(CombinationFlags::LIST_ARRAY_SOMETIMES_FILLED);
                    } else {
                        combination.flags.remove(CombinationFlags::LIST_ARRAY_ALWAYS_FILLED);
                    }

                    if let Some(known_elements) = known_elements {
                        let mut has_defined_keys = false;

                        for (candidate_element_index, (candidate_optional, candidate_element_type)) in known_elements {
                            let existing_entry = combination.list_array_entries.get(&candidate_element_index);

                            let new_entry = if let Some((existing_optional, existing_type)) = existing_entry {
                                (
                                    *existing_optional || candidate_optional,
                                    combine_union_types(existing_type, &candidate_element_type, codebase, options),
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
                                                options,
                                            );

                                            if !candidate_optional {
                                                has_defined_keys = true;
                                            }

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
                            combination.flags.remove(CombinationFlags::LIST_ARRAY_ALWAYS_FILLED);
                        }
                    } else if !options.overwrite_empty_array {
                        if element_type.is_never() {
                            for (pu, _) in combination.list_array_entries.values_mut() {
                                *pu = true;
                            }
                        } else {
                            for (_, entry_type) in combination.list_array_entries.values() {
                                if let Some(ref mut existing_value_param) = combination.list_array_parameter {
                                    *existing_value_param =
                                        combine_union_types(existing_value_param, entry_type, codebase, options);
                                }
                            }

                            combination.list_array_entries.clear();
                        }
                    }

                    combination.list_array_parameter =
                        if let Some(existing_type) = combination.list_array_parameter.as_ref() {
                            Some(combine_union_types(existing_type, &element_type, codebase, options))
                        } else {
                            Some((*element_type).clone())
                        };
                }
                TArray::Keyed(TKeyedArray { parameters, known_items, non_empty }) => {
                    let mut had_previous_keyed_array = combination.flags.contains(CombinationFlags::HAS_KEYED_ARRAY);
                    let sealed_budget_available = !combination.sealed_keyed_budget_exhausted
                        && combination.sealed_arrays.len() < options.array_combination_threshold as usize;

                    if !sealed_budget_available
                        && !combination.sealed_keyed_budget_exhausted
                        && !combination.sealed_arrays.is_empty()
                    {
                        flush_sealed_keyed_arrays_into_combination(combination, codebase, options);
                        combination.sealed_keyed_budget_exhausted = true;
                        had_previous_keyed_array = combination.flags.contains(CombinationFlags::HAS_KEYED_ARRAY);
                    }

                    if had_previous_keyed_array && sealed_budget_available {
                        let incoming_is_sealed = parameters.is_none();
                        let existing_is_sealed = combination.keyed_array_parameters.is_none();

                        if incoming_is_sealed && !existing_is_sealed && known_items.is_some() {
                            let known_items = widen_known_items_with_params(
                                known_items,
                                combination.keyed_array_parameters.as_ref(),
                                &combination.keyed_array_entries,
                                codebase,
                                options,
                            );

                            combination.sealed_arrays.push(TArray::Keyed(TKeyedArray {
                                known_items,
                                parameters,
                                non_empty,
                            }));

                            continue;
                        }

                        if !incoming_is_sealed && existing_is_sealed && !combination.keyed_array_entries.is_empty() {
                            let mut frozen_entries = std::mem::take(&mut combination.keyed_array_entries);
                            if let Some((key_param, value_param)) = parameters.as_ref() {
                                for (key, (_, entry_type)) in frozen_entries.iter_mut() {
                                    // If the incoming unsealed array also declares this key as a
                                    // known item, the caller is saying this key is exactly the
                                    // declared type - the generic value_param catch-all covers
                                    // *other* keys only. Widening here would turn e.g.
                                    // `array{count: int, id: int}` + `array{count: int, ...<string, mixed>}`
                                    // into `array{count: mixed, id: mixed}`, which is a false loss.
                                    if known_items.as_ref().is_some_and(|ki| ki.contains_key(key)) {
                                        continue;
                                    }

                                    let key_type = TUnion::from_atomic(key.to_atomic());

                                    if union_comparator::can_expression_types_be_identical(
                                        codebase, &key_type, key_param, false, false,
                                    ) {
                                        *entry_type = combine_union_types(entry_type, value_param, codebase, options);
                                    }
                                }
                            }

                            let frozen = TArray::Keyed(TKeyedArray {
                                known_items: Some(frozen_entries),
                                parameters: None,
                                non_empty: combination.flags.contains(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED),
                            });
                            combination.sealed_arrays.push(frozen);
                            combination.flags.remove(CombinationFlags::HAS_KEYED_ARRAY);
                            combination.flags.remove(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED);
                            combination.flags.insert(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED);
                            had_previous_keyed_array = false;
                        }

                        if incoming_is_sealed
                            && existing_is_sealed
                            && !combination.keyed_array_entries.is_empty()
                            && let Some(known_items_inner) = known_items.as_ref()
                            && combination.sealed_arrays.len() + 1 < options.array_combination_threshold as usize
                            && (!known_items_inner.keys().any(|k| combination.keyed_array_entries.contains_key(k))
                                || shapes_are_discriminated(
                                    known_items_inner,
                                    &combination.keyed_array_entries,
                                    codebase,
                                ))
                        {
                            let frozen = TArray::Keyed(TKeyedArray {
                                known_items: Some(std::mem::take(&mut combination.keyed_array_entries)),
                                parameters: None,
                                non_empty: combination.flags.contains(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED),
                            });
                            combination.sealed_arrays.push(frozen);
                            combination.sealed_arrays.push(TArray::Keyed(TKeyedArray {
                                known_items,
                                parameters,
                                non_empty,
                            }));
                            combination.flags.remove(CombinationFlags::HAS_KEYED_ARRAY);
                            combination.flags.remove(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED);
                            combination.flags.insert(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED);

                            continue;
                        }
                    }

                    combination.flags.insert(CombinationFlags::HAS_KEYED_ARRAY);

                    if non_empty {
                        combination.flags.insert(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED);
                    } else {
                        combination.flags.remove(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED);

                        if parameters.is_none()
                            && known_items.as_ref().is_none_or(|items| items.is_empty())
                            && combination.list_array_parameter.is_some()
                        {
                            combination.flags.remove(CombinationFlags::LIST_ARRAY_ALWAYS_FILLED);
                            had_previous_keyed_array = false;
                            combination.flags.remove(CombinationFlags::HAS_KEYED_ARRAY);

                            continue;
                        }
                    }

                    if let Some(known_items) = known_items {
                        let has_existing_entries =
                            !combination.keyed_array_entries.is_empty() || had_previous_keyed_array;
                        let mut possibly_undefined_entries =
                            combination.keyed_array_entries.keys().copied().collect::<HashSet<_>>();

                        let mut has_defined_keys = false;

                        for (candidate_item_name, (cu, candidate_item_type)) in known_items {
                            if let Some((eu, existing_type)) =
                                combination.keyed_array_entries.get_mut(&candidate_item_name)
                            {
                                if cu {
                                    *eu = true;
                                }
                                if &candidate_item_type != existing_type {
                                    *existing_type =
                                        combine_union_types(existing_type, &candidate_item_type, codebase, options);
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
                                            options,
                                            &candidate_item_name,
                                            existing_key_param,
                                        );

                                        continue;
                                    } else {
                                        let new_type = candidate_item_type.clone();
                                        (has_existing_entries || cu, new_type)
                                    };

                                combination.keyed_array_entries.insert(candidate_item_name, new_item_value_type);
                            }

                            possibly_undefined_entries.remove(&candidate_item_name);

                            if !cu {
                                has_defined_keys = true;
                            }
                        }

                        if !has_defined_keys {
                            combination.flags.remove(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED);
                        }

                        for possibly_undefined_type_key in possibly_undefined_entries {
                            let possibly_undefined_type =
                                combination.keyed_array_entries.get_mut(&possibly_undefined_type_key);
                            if let Some((pu, _)) = possibly_undefined_type {
                                *pu = true;
                            }
                        }
                    } else if !options.overwrite_empty_array {
                        if match &parameters {
                            Some((_, value_param)) => value_param.is_never(),
                            None => true,
                        } {
                            for (tu, _) in combination.keyed_array_entries.values_mut() {
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
                                        options,
                                        key,
                                        existing_key_param,
                                    );
                                }
                            }

                            combination.keyed_array_entries.clear();
                        }
                    }

                    combination.keyed_array_parameters = match (&combination.keyed_array_parameters, parameters) {
                        (None, None) => None,
                        (Some(existing_types), None) => Some(existing_types.clone()),
                        (None, Some(params)) => Some(((*params.0).clone(), (*params.1).clone())),
                        (Some(existing_types), Some(params)) => Some((
                            combine_union_types(&existing_types.0, &params.0, codebase, options),
                            combine_union_types(&existing_types.1, &params.1, codebase, options),
                        )),
                    };
                }
            }
        }

        return;
    }

    // this probably won't ever happen, but the object top type
    // can eliminate variants
    if atomic == TAtomic::Object(TObject::Any) {
        combination.flags.insert(CombinationFlags::HAS_OBJECT_TOP_TYPE);
        combination.value_types.retain(|_, t| !matches!(t, TAtomic::Object(TObject::Named(_))));
        combination.value_types.insert(atomic.get_id(), atomic);

        return;
    }

    if let TAtomic::Object(TObject::Named(named_object)) = &atomic {
        if let Some(object_static) = combination.object_static.get(&named_object.get_name()) {
            if *object_static && !named_object.is_static {
                combination.object_static.insert(named_object.get_name(), false);
            }
        } else {
            combination.object_static.insert(named_object.get_name(), named_object.is_static);
        }
    }

    if let TAtomic::Object(TObject::Named(named_object)) = &atomic {
        let fq_class_name = named_object.get_name();
        if let Some(type_parameters) = named_object.get_type_parameters() {
            let object_type_key = get_combiner_key(fq_class_name, type_parameters, codebase);

            if let Some((_, existing_type_params)) = combination.object_type_params.get(&object_type_key) {
                let mut new_type_parameters = Vec::with_capacity(type_parameters.len());
                for (i, type_param) in type_parameters.iter().enumerate() {
                    if let Some(existing_type_param) = existing_type_params.get(i) {
                        new_type_parameters.push(combine_union_types(
                            existing_type_param,
                            type_param,
                            codebase,
                            options,
                        ));
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

        if combination.flags.contains(CombinationFlags::HAS_OBJECT_TOP_TYPE)
            || combination.value_types.contains_key(&atomic.get_id())
        {
            return;
        }

        let Some(symbol_type) = codebase.symbols.get_kind(fq_class_name) else {
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

                let Some(existing_symbol_kind) = codebase.symbols.get_kind(existing_object.get_name()) else {
                    continue;
                };

                if matches!(existing_symbol_kind, SymbolKind::Class) {
                    // remove subclasses
                    if codebase.is_instance_of(&existing_name, &fq_class_name) {
                        types_to_remove.push(*key);
                        continue;
                    }

                    if is_class {
                        // if covered by a parent class
                        if codebase.class_extends(&fq_class_name, &existing_name) {
                            return;
                        }
                    } else if is_interface {
                        // if covered by a parent class
                        if codebase.class_implements(&fq_class_name, &existing_name) {
                            return;
                        }
                    }
                } else if matches!(existing_symbol_kind, SymbolKind::Interface) {
                    if codebase.class_implements(&existing_name, &fq_class_name) {
                        types_to_remove.push(existing_name);
                        continue;
                    }

                    if (is_class || is_interface) && codebase.class_implements(&fq_class_name, &existing_name) {
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

    if atomic == TAtomic::Scalar(TScalar::Generic) {
        combination.literal_strings.clear();
        combination.integers.clear();
        combination.literal_floats.clear();
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

    if atomic == TAtomic::Scalar(TScalar::ArrayKey) {
        if combination.value_types.contains_key(&*ATOM_SCALAR) {
            return;
        }

        combination.literal_strings.clear();
        combination.integers.clear();
        combination.value_types.retain(|k, _| k != &*ATOM_STRING && k != &*ATOM_INT);
        combination.value_types.insert(atomic.get_id(), atomic);

        return;
    }

    if let TAtomic::Scalar(TScalar::String(_) | TScalar::Integer(_)) = atomic
        && (combination.value_types.contains_key(&*ATOM_SCALAR)
            || combination.value_types.contains_key(&*ATOM_ARRAY_KEY))
    {
        return;
    }

    if let TAtomic::Scalar(TScalar::Float(_) | TScalar::Integer(_)) = atomic
        && (combination.value_types.contains_key(&*ATOM_NUMERIC) || combination.value_types.contains_key(&*ATOM_SCALAR))
    {
        return;
    }

    if let TAtomic::Scalar(TScalar::String(mut string_scalar)) = atomic {
        if let Some(existing_string_type) = combination.value_types.get_mut(&*ATOM_STRING) {
            if let TAtomic::Scalar(TScalar::String(existing_string_type)) = existing_string_type {
                if let Some(lit_atom) = string_scalar.get_known_literal_atom() {
                    let lit_value = lit_atom.as_str();
                    let is_incompatible = (existing_string_type.is_numeric && !str_is_numeric(lit_value))
                        || (existing_string_type.is_truthy && (lit_value.is_empty() || lit_value == "0"))
                        || (existing_string_type.is_non_empty && lit_value.is_empty())
                        || (existing_string_type.is_lowercase() && lit_value.chars().any(char::is_uppercase))
                        || (existing_string_type.is_uppercase() && lit_value.chars().any(char::is_lowercase));

                    if is_incompatible {
                        // Check threshold before adding literal string
                        if combination.literal_strings.len() >= options.string_combination_threshold as usize {
                            // Exceeded threshold - just merge into the base string type
                            *existing_string_type = combine_string_scalars(existing_string_type, string_scalar);
                        } else {
                            combination.literal_strings.insert(lit_atom);
                        }
                    } else {
                        *existing_string_type = combine_string_scalars(existing_string_type, string_scalar);
                    }
                } else {
                    *existing_string_type = combine_string_scalars(existing_string_type, string_scalar);
                }
            }
        } else if let Some(atom) = string_scalar.get_known_literal_atom() {
            // Check threshold before adding literal string
            if combination.literal_strings.len() >= options.string_combination_threshold as usize {
                // Exceeded threshold - generalize to base string type
                combination.literal_strings.clear();
                combination.value_types.insert(*ATOM_STRING, TAtomic::Scalar(TScalar::string()));
            } else {
                combination.literal_strings.insert(atom);
            }
        } else {
            let mut literals_to_keep = AtomSet::default();
            if !combination.literal_strings.is_empty() {
                string_scalar.is_callable = false;
            }

            if string_scalar.is_truthy
                || string_scalar.is_non_empty
                || string_scalar.is_numeric
                || !string_scalar.casing.is_unspecified()
            {
                for value in &combination.literal_strings {
                    if value.is_empty() {
                        string_scalar.is_non_empty = false;
                        string_scalar.is_truthy = false;
                        string_scalar.is_numeric = false;
                        break;
                    } else if value == "0" {
                        string_scalar.is_truthy = false;
                    }

                    if string_scalar.is_numeric && !str_is_numeric(value) {
                        literals_to_keep.insert(*value);
                    } else {
                        string_scalar.is_numeric = string_scalar.is_numeric && str_is_numeric(value);
                    }

                    string_scalar.casing = match string_scalar.casing {
                        TStringCasing::Lowercase if value.chars().all(|c| c.is_lowercase()) => TStringCasing::Lowercase,
                        TStringCasing::Uppercase if value.chars().all(|c| c.is_uppercase()) => TStringCasing::Uppercase,
                        _ => TStringCasing::Unspecified,
                    };
                }
            }

            combination.value_types.insert(*ATOM_STRING, TAtomic::Scalar(TScalar::String(string_scalar)));

            std::mem::swap(&mut combination.literal_strings, &mut literals_to_keep);
        }

        return;
    }

    if let TAtomic::Scalar(TScalar::Integer(integer)) = &atomic {
        // If we already have the base int type, no need to track literals
        if combination.value_types.contains_key(&*ATOM_INT) {
            return;
        }

        // Check if adding this integer would exceed the threshold
        if integer.is_literal() && combination.integers.len() >= options.integer_combination_threshold as usize {
            // Exceeded threshold - generalize to base int type
            combination.integers.clear();
            combination.value_types.insert(*ATOM_INT, TAtomic::Scalar(TScalar::int()));
            return;
        }

        combination.integers.push(*integer);

        return;
    }

    if let TAtomic::Scalar(TScalar::Float(float_scalar)) = &atomic {
        if let Some(stored) = combination.value_types.get(&*ATOM_FLOAT) {
            if matches!(stored, TAtomic::Scalar(TScalar::Float(TFloat::Float))) {
                return;
            }

            if matches!(float_scalar, TFloat::Float) {
                combination.literal_floats.clear();
                combination.value_types.insert(*ATOM_FLOAT, atomic);
            }

            return;
        }

        if let TFloat::Literal(literal_value) = float_scalar {
            if combination.literal_floats.len() >= options.string_combination_threshold as usize {
                combination.literal_floats.clear();
                combination.value_types.insert(*ATOM_FLOAT, TAtomic::Scalar(TScalar::float()));
                return;
            }
            combination.literal_floats.push(*literal_value);
        } else {
            combination.literal_floats.clear();
            combination.value_types.insert(*ATOM_FLOAT, atomic);
        }

        return;
    }

    combination.value_types.insert(atomic.get_id(), atomic);
}

fn shapes_are_discriminated(
    incoming: &BTreeMap<ArrayKey, (bool, TUnion)>,
    existing: &BTreeMap<ArrayKey, (bool, TUnion)>,
    codebase: &CodebaseMetadata,
) -> bool {
    let mut has_asymmetric_keys = false;
    for key in incoming.keys() {
        if !existing.contains_key(key) {
            has_asymmetric_keys = true;
            break;
        }
    }

    if !has_asymmetric_keys {
        for key in existing.keys() {
            if !incoming.contains_key(key) {
                has_asymmetric_keys = true;
                break;
            }
        }
    }

    if !has_asymmetric_keys {
        return false;
    }

    for (key, (incoming_optional, incoming_type)) in incoming {
        if *incoming_optional {
            continue;
        }

        let Some((existing_optional, existing_type)) = existing.get(key) else {
            continue;
        };

        if *existing_optional {
            continue;
        }

        if !union_comparator::can_expression_types_be_identical(codebase, incoming_type, existing_type, false, false) {
            return true;
        }
    }

    false
}

/// Widens known items in a sealed array with the generic value type from parameters.
/// This is needed when combining a sealed array with a parametric one, the parametric
/// array's generic string keys could overwrite any of the sealed array's known keys.
fn widen_known_items_with_params(
    known_items: Option<BTreeMap<ArrayKey, (bool, TUnion)>>,
    params: Option<&(TUnion, TUnion)>,
    other_known_items: &BTreeMap<ArrayKey, (bool, TUnion)>,
    codebase: &CodebaseMetadata,
    options: CombinerOptions,
) -> Option<BTreeMap<ArrayKey, (bool, TUnion)>> {
    let mut items = known_items?;

    if let Some((key_param, value_param)) = params {
        let key_param_accepts_int;
        let key_param_accepts_string;
        if key_param.has_mixed() || key_param.has_mixed_template() {
            key_param_accepts_int = true;
            key_param_accepts_string = true;
        } else {
            let mut accepts_int = false;
            let mut accepts_string = false;
            for part in key_param.types.as_ref() {
                if accepts_int && accepts_string {
                    break;
                }

                match part {
                    TAtomic::Scalar(TScalar::ArrayKey) => {
                        accepts_int = true;
                        accepts_string = true;
                    }
                    TAtomic::Scalar(TScalar::Integer(_)) => accepts_int = true,
                    TAtomic::Scalar(TScalar::String(_)) => accepts_string = true,
                    _ => {
                        accepts_int = true;
                        accepts_string = true;
                    }
                }
            }

            key_param_accepts_int = accepts_int;
            key_param_accepts_string = accepts_string;
        }

        if !key_param_accepts_int && !key_param_accepts_string {
            return Some(items);
        }

        for (key, (_, entry_type)) in items.iter_mut() {
            if entry_type == value_param {
                continue;
            }

            if other_known_items.contains_key(key) {
                continue;
            }

            let key_compatible = match key {
                ArrayKey::Integer(_) => key_param_accepts_int,
                ArrayKey::String(_) => key_param_accepts_string,
                ArrayKey::ClassLikeConstant { .. } => key_param_accepts_int || key_param_accepts_string,
            };

            if !key_compatible {
                continue;
            }

            *entry_type = combine_union_types(entry_type, value_param, codebase, options);
        }
    }

    Some(items)
}

fn adjust_keyed_array_parameters(
    existing_value_param: &mut TUnion,
    entry_type: &TUnion,
    codebase: &CodebaseMetadata,
    options: CombinerOptions,
    key: &ArrayKey,
    existing_key_param: &mut TUnion,
) {
    *existing_value_param = combine_union_types(existing_value_param, entry_type, codebase, options);
    let new_key_type = key.to_union();
    *existing_key_param = combine_union_types(existing_key_param, &new_key_type, codebase, options);
}

fn flush_sealed_keyed_arrays_into_combination(
    combination: &mut TypeCombination,
    codebase: &CodebaseMetadata,
    options: CombinerOptions,
) {
    let sealed = std::mem::take(&mut combination.sealed_arrays);
    let mut any_keyed = false;
    let mut put_back = Vec::new();

    for array in sealed {
        let TArray::Keyed(keyed) = array else {
            put_back.push(array);
            continue;
        };

        any_keyed = true;
        let TKeyedArray { known_items, parameters, non_empty } = keyed;

        if non_empty {
            combination.flags.insert(CombinationFlags::KEYED_ARRAY_SOMETIMES_FILLED);
        } else {
            combination.flags.remove(CombinationFlags::KEYED_ARRAY_ALWAYS_FILLED);
        }

        if let Some(known_items) = known_items {
            for (candidate_item_name, (candidate_optional, candidate_item_type)) in known_items {
                if let Some((existing_optional, existing_type)) =
                    combination.keyed_array_entries.get_mut(&candidate_item_name)
                {
                    if candidate_optional {
                        *existing_optional = true;
                    }
                    if &candidate_item_type != existing_type {
                        *existing_type = combine_union_types(existing_type, &candidate_item_type, codebase, options);
                    }
                } else {
                    let inserted = if let Some((ref mut existing_key_param, ref mut existing_value_param)) =
                        combination.keyed_array_parameters
                    {
                        adjust_keyed_array_parameters(
                            existing_value_param,
                            &candidate_item_type,
                            codebase,
                            options,
                            &candidate_item_name,
                            existing_key_param,
                        );
                        None
                    } else {
                        Some((true, candidate_item_type.clone()))
                    };

                    if let Some(entry) = inserted {
                        combination.keyed_array_entries.insert(candidate_item_name, entry);
                    }
                }
            }
        }

        combination.keyed_array_parameters = match (combination.keyed_array_parameters.take(), parameters) {
            (None, None) => None,
            (Some(existing_types), None) => Some(existing_types),
            (None, Some(params)) => Some(((*params.0).clone(), (*params.1).clone())),
            (Some(existing_types), Some(params)) => Some((
                combine_union_types(&existing_types.0, &params.0, codebase, options),
                combine_union_types(&existing_types.1, &params.1, codebase, options),
            )),
        };
    }

    if any_keyed {
        combination.flags.insert(CombinationFlags::HAS_KEYED_ARRAY);
    }

    combination.sealed_arrays = put_back;
}

const COMBINER_KEY_STACK_BUF: usize = 256;

fn get_combiner_key(name: Atom, type_params: &[TUnion], codebase: &CodebaseMetadata) -> Atom {
    let covariants = if let Some(class_like_metadata) = codebase.get_class_like(&name) {
        &class_like_metadata.template_variance
    } else {
        return name;
    };

    let name_str = name.as_str();
    let mut estimated_len = name_str.len() + 2; // name + "<" + ">"
    for (i, tunion) in type_params.iter().enumerate() {
        if i > 0 {
            estimated_len += 2; // ", "
        }

        if covariants.get(i) == Some(&Variance::Covariant) {
            estimated_len += 1; // "*"
        } else {
            estimated_len += tunion.get_id().len();
        }
    }

    if estimated_len <= COMBINER_KEY_STACK_BUF {
        let mut buffer = [0u8; COMBINER_KEY_STACK_BUF];
        let mut pos = 0;

        buffer[pos..pos + name_str.len()].copy_from_slice(name_str.as_bytes());
        pos += name_str.len();

        buffer[pos] = b'<';
        pos += 1;

        for (i, tunion) in type_params.iter().enumerate() {
            if i > 0 {
                buffer[pos..pos + 2].copy_from_slice(b", ");
                pos += 2;
            }
            let param_str =
                if covariants.get(i) == Some(&Variance::Covariant) { "*" } else { tunion.get_id().as_str() };
            buffer[pos..pos + param_str.len()].copy_from_slice(param_str.as_bytes());
            pos += param_str.len();
        }

        buffer[pos] = b'>';
        pos += 1;

        // SAFETY: We only write valid UTF-8 (ASCII characters and valid UTF-8 from Atom strings)
        return atom(unsafe { std::str::from_utf8_unchecked(&buffer[..pos]) });
    }

    let mut result = String::with_capacity(estimated_len);
    result.push_str(name_str);
    result.push('<');
    for (i, tunion) in type_params.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        if covariants.get(i) == Some(&Variance::Covariant) {
            result.push('*');
        } else {
            result.push_str(tunion.get_id().as_str());
        }
    }
    result.push('>');
    atom(&result)
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
        is_callable: s1.is_callable && s2.is_callable,
        casing: match (s1.casing, s2.casing) {
            (TStringCasing::Lowercase, TStringCasing::Lowercase) => TStringCasing::Lowercase,
            (TStringCasing::Uppercase, TStringCasing::Uppercase) => TStringCasing::Uppercase,
            _ => TStringCasing::Unspecified,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    use crate::ttype::atomic::TAtomic;
    use crate::ttype::atomic::array::list::TList;
    use crate::ttype::atomic::scalar::TScalar;

    #[test]
    fn test_combine_scalars() {
        let types = vec![
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Scalar(TScalar::bool()),
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        assert_eq!(combined.len(), 1);
        assert!(matches!(combined[0], TAtomic::Scalar(TScalar::Generic)));
    }

    #[test]
    fn test_combine_boolean_lists() {
        let types = vec![
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::r#false())))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::r#true())))),
            ])))),
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::r#true())))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::r#false())))),
            ])))),
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        assert_eq!(combined.len(), 2);
        assert!(matches!(combined[0], TAtomic::Array(TArray::List(_))));
        assert!(matches!(combined[1], TAtomic::Array(TArray::List(_))));
    }

    #[test]
    fn test_combine_integer_lists() {
        let types = vec![
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(1)))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(2)))))),
            ])))),
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(2)))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(1)))))),
            ])))),
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        assert_eq!(combined.len(), 2);
        assert!(matches!(combined[0], TAtomic::Array(TArray::List(_))));
        assert!(matches!(combined[1], TAtomic::Array(TArray::List(_))));
    }

    #[test]
    fn test_combine_string_lists() {
        let types = vec![
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("a".into())))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("b".into())))))),
            ])))),
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("b".into())))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("a".into())))))),
            ])))),
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        assert_eq!(combined.len(), 2);
        assert!(matches!(combined[0], TAtomic::Array(TArray::List(_))));
        assert!(matches!(combined[1], TAtomic::Array(TArray::List(_))));
    }

    #[test]
    fn test_combine_mixed_literal_lists() {
        let types = vec![
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(1)))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("a".into())))))),
            ])))),
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal("b".into())))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(2)))))),
            ])))),
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        assert_eq!(combined.len(), 2);
        assert!(matches!(combined[0], TAtomic::Array(TArray::List(_))));
        assert!(matches!(combined[1], TAtomic::Array(TArray::List(_))));
    }

    #[test]
    fn test_combine_list_with_generic_list() {
        let types = vec![
            TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(1)))))),
                (1, (false, TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(2)))))),
            ])))),
            TAtomic::Array(TArray::List(TList::new(Arc::new(TUnion::from_atomic(TAtomic::Scalar(TScalar::int())))))), // list<int>
        ];

        let combined =
            combine(types, &CodebaseMetadata::default(), CombinerOptions::default().with_overwrite_empty_array());

        // Expecting list{1,2} and list<int> = list<int>
        assert_eq!(combined.len(), 1);

        let TAtomic::Array(TArray::List(list_type)) = &combined[0] else {
            panic!("Expected a list type");
        };

        let Some(known_elements) = &list_type.known_elements else {
            panic!("Expected known elements");
        };

        assert!(!list_type.is_non_empty());
        assert!(list_type.known_count.is_none());
        assert!(list_type.element_type.is_int());

        assert_eq!(known_elements.len(), 2);
        assert!(known_elements.contains_key(&0));
        assert!(known_elements.contains_key(&1));

        let Some(first_element) = known_elements.get(&0) else {
            panic!("Expected first element");
        };

        let Some(second_element) = known_elements.get(&1) else {
            panic!("Expected second element");
        };

        assert!(first_element.1.is_int());
        assert!(second_element.1.is_int());
    }
}
