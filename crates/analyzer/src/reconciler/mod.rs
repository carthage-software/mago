use std::borrow::Cow;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::LazyLock;

use foldhash::HashSet;
use regex::Regex;

use mago_algebra::AssertionMap;
use mago_algebra::assertion_set::AssertionSet;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use mago_codex::assertion::Assertion;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_iterable_value_parameter;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_string;
use mago_codex::ttype::intersect_union_types;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::concat_word;
use mago_word::word;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::var_has_root;
use crate::resolver::property::resolve_declared_property;

pub mod assertion_reconciler;
pub mod negated_assertion_reconciler;
pub mod simple_assertion_reconciler;
pub mod simple_negated_assertion_reconciler;

mod macros;

#[allow(clippy::similar_names)]
pub fn reconcile_keyed_types<'ctx, A>(
    context: &mut Context<'ctx, '_, A>,
    new_types: &AssertionMap<Word, AssertionSet>,
    mut active_new_types: AssertionMap<Word, HashSet<usize>>,
    block_context: &mut BlockContext<'ctx>,
    changed_var_ids: &mut WordSet,
    referenced_var_ids: &WordSet,
    span: &Span,
    can_report_issues: bool,
    negated: bool,
) where
    A: Arena,
{
    if new_types.is_empty() {
        return;
    }

    let mut reference_graph: WordMap<WordSet> = WordMap::default();
    if !block_context.references_in_scope.is_empty() {
        // PHP behaves oddly when passing an array containing references: https://bugs.php.net/bug.php?id=20993
        // To work around the issue, if there are any references, we have to recreate the array and fix the
        // references so they're properly scoped and won't affect the caller. Starting with a new array is
        // required for some unclear reason, just cloning elements of the existing array doesn't work properly.
        let old_locals = std::mem::take(&mut block_context.locals);

        let mut cloned_references: WordSet = WordSet::default();
        for (reference, referenced) in &block_context.references_in_scope {
            if cloned_references.contains(referenced) {
                block_context.locals.insert(*referenced, Rc::clone(&old_locals[referenced]));
                cloned_references.insert(*reference);
            }
        }

        block_context.locals.extend(old_locals);
        for (reference, referenced) in &block_context.references_in_scope {
            reference_graph.entry(*reference).or_default().insert(*referenced);

            for existing_referenced in
                reference_graph.get(referenced).map(|s| s.iter().copied().collect::<Vec<_>>()).unwrap_or_default()
            {
                reference_graph.entry(existing_referenced).or_default().insert(*reference);
                reference_graph.entry(*reference).or_default().insert(existing_referenced);
            }

            reference_graph.entry(*referenced).or_default().insert(*reference);
        }
    }

    let inside_loop = block_context.flags.inside_loop();
    let old_new_types = new_types;
    let mut new_types = Cow::Borrowed(new_types);

    for (derived_local, source) in &block_context.derived_local_sources {
        if let Some(assertions) = old_new_types.get(derived_local)
            && !new_types.contains_key(source)
        {
            new_types.to_mut().insert(*source, assertions.clone());
        }
    }

    add_nested_assertions(&mut new_types, &mut active_new_types, block_context);
    if new_types.len() > 1 {
        new_types
            .to_mut()
            .sort_by_cached_key(|key, _| (old_new_types.get(key).is_none(), path_part_count(key.as_bytes())));
    }

    let original_types = can_report_issues.then(|| {
        new_types
            .keys()
            .filter_map(|key| block_context.locals.get(key).map(|ty| (*key, Rc::clone(ty))))
            .collect::<WordMap<_>>()
    });

    for (key, new_type_parts) in new_types.iter() {
        let key_str = key.as_bytes();
        if key_str.ends_with(b"()") && !block_context.locals.contains_key(key) {
            if block_context.stable_method_calls.contains(key) {
                block_context.stable_method_call_assertions.entry(*key).or_default().extend(new_type_parts.clone());
            }

            continue;
        }

        if memchr::memmem::find(key_str, b"::").is_some() && !key_str.contains(&b'$') && !key_str.contains(&b'[') {
            continue;
        }

        let mut has_negation = false;
        let mut has_isset = false;
        let mut has_inverted_isset = false;
        let mut has_inverted_key_exists = false;
        let mut has_truthy_or_falsy_or_empty = false;
        let mut has_count_check = false;
        let mut has_empty = false;
        let mut has_relational_comparison = false;
        let is_real = old_new_types.get(key).is_some_and(|v| v.eq(new_type_parts));
        let mut is_equality = is_real;

        for new_type_part_parts in new_type_parts {
            for assertion in new_type_part_parts {
                if assertion.is_negation() {
                    has_negation = true;
                }

                has_isset = has_isset || assertion.has_isset();
                has_truthy_or_falsy_or_empty = has_truthy_or_falsy_or_empty
                    || matches!(
                        assertion,
                        Assertion::Truthy | Assertion::Falsy | Assertion::Empty | Assertion::NonEmpty
                    );
                is_equality = is_equality && matches!(assertion, Assertion::IsIdentical(_));
                has_empty = has_empty || matches!(assertion, Assertion::Empty);
                has_inverted_isset = has_inverted_isset || matches!(assertion, Assertion::IsNotIsset);
                has_inverted_key_exists =
                    has_inverted_key_exists || matches!(assertion, Assertion::ArrayKeyDoesNotExist);
                has_count_check = has_count_check || matches!(assertion, Assertion::NonEmptyCountable(_));
                has_relational_comparison = has_relational_comparison
                    || matches!(
                        assertion,
                        Assertion::IsLessThanVariable(_)
                            | Assertion::IsLessThanOrEqualVariable(_)
                            | Assertion::IsGreaterThanVariable(_)
                            | Assertion::IsGreaterThanOrEqualVariable(_)
                    );
            }
        }

        if has_inverted_isset && key_str.starts_with(b"$this->") {
            block_context.definitely_uninitialized_property_ids.insert(*key);
        } else if has_isset {
            block_context.definitely_uninitialized_property_ids.remove(key);
        }

        let existing = block_context.locals.get(key).cloned();
        let did_type_exist = existing.is_some();
        let mut has_object_array_access = false;

        let before_adjustment = existing.or_else(|| {
            get_value_for_key(
                context,
                *key,
                block_context,
                &new_types,
                has_isset,
                has_inverted_isset,
                has_inverted_key_exists,
                false,
                inside_loop,
                &mut has_object_array_access,
            )
            .map(Rc::new)
        });

        let mut result_type = before_adjustment.as_deref().map(Cow::Borrowed);
        for (i, new_type_part_parts) in new_type_parts.iter().enumerate() {
            let mut orred_type: Option<TUnion> = None;

            for assertion in new_type_part_parts {
                let mut report_this_assertion = can_report_issues
                    && new_type_part_parts.len() == 1
                    && referenced_var_ids.contains(key)
                    && active_new_types.get(key).is_some_and(|active_new_type| active_new_type.contains(&i));

                if report_this_assertion
                    && let Some(original) = original_types
                        .as_ref()
                        .and_then(|types| types.get(key).map(Rc::as_ref))
                        .or(before_adjustment.as_deref())
                    && result_type.as_deref().is_some_and(|current| current != original)
                {
                    let probe_on_original = assertion_reconciler::reconcile(
                        context,
                        assertion,
                        Some(original),
                        Some(key_str),
                        inside_loop,
                        Some(span),
                        false,
                        negated,
                    );

                    if &probe_on_original != original {
                        report_this_assertion = false;
                    }
                }

                let result_type_candidate = assertion_reconciler::reconcile(
                    context,
                    assertion,
                    result_type.as_deref(),
                    Some(key_str),
                    inside_loop,
                    Some(span),
                    report_this_assertion,
                    negated,
                );

                orred_type =
                    Some(add_optional_union_type(result_type_candidate, orred_type.as_ref(), context.codebase));
            }

            result_type = orred_type.map(Cow::Owned);
        }

        let needs_refinement =
            new_type_parts.len() > 1 && result_type.as_deref().is_some_and(|current| !current.is_never());
        if needs_refinement {
            for new_type_part_parts in new_type_parts {
                let mut orred_type: Option<TUnion> = None;

                for assertion in new_type_part_parts {
                    let result_type_candidate = assertion_reconciler::reconcile(
                        context,
                        assertion,
                        result_type.as_deref(),
                        Some(key_str),
                        inside_loop,
                        Some(span),
                        false,
                        negated,
                    );

                    orred_type =
                        Some(add_optional_union_type(result_type_candidate, orred_type.as_ref(), context.codebase));
                }

                result_type = orred_type.map(Cow::Owned);
            }
        }

        let result_type = result_type.map(Cow::into_owned).unwrap_or_else(get_never);

        if !did_type_exist && result_type.is_never() {
            // Even when the type doesn't exist and result is never, we still need to
            // update parent array types for negated isset/key_exists to remove the key
            if key_str.ends_with(b"]") && (has_inverted_isset || has_inverted_key_exists) {
                adjust_array_type_remove_key(
                    borrow_path_parts(key_str),
                    block_context,
                    changed_var_ids,
                    context.codebase,
                );
            }

            continue;
        }

        let type_changed = if let Some(before_adjustment) = before_adjustment.as_deref() {
            &result_type != before_adjustment
        } else {
            true
        };

        if type_changed {
            changed_var_ids.insert(*key);
            if key_str.ends_with(b"]") && !has_inverted_isset && !has_inverted_key_exists && !has_empty {
                adjust_array_type(
                    borrow_path_parts(key_str),
                    block_context,
                    changed_var_ids,
                    &result_type,
                    context.codebase,
                    false,
                );
            } else if key_str.ends_with(b"]") && (has_inverted_isset || has_inverted_key_exists) {
                let optional_entry_type = reconcile_optional_entry_type(
                    context,
                    new_type_parts,
                    before_adjustment.as_deref(),
                    key_str,
                    inside_loop,
                    span,
                    negated,
                );

                if let Some(entry_type) = optional_entry_type {
                    adjust_array_type(
                        borrow_path_parts(key_str),
                        block_context,
                        changed_var_ids,
                        &entry_type,
                        context.codebase,
                        true,
                    );
                } else {
                    adjust_array_type_remove_key(
                        borrow_path_parts(key_str),
                        block_context,
                        changed_var_ids,
                        context.codebase,
                    );
                }
            } else if memchr::memmem::find(key_str, b"->").is_some() && !is_equality {
                adjust_object_property_type(
                    borrow_path_parts(key_str),
                    block_context,
                    changed_var_ids,
                    &result_type,
                    context,
                );
            }

            if is_real && key_str != b"$this" {
                let mut removable_keys: Vec<Word> = Vec::new();
                let local_keys = block_context.locals.keys().copied().collect::<Vec<_>>();
                for new_key in local_keys {
                    if new_key == *key {
                        continue;
                    }

                    if !new_types.contains_key(&new_key) && var_has_root(new_key, *key) {
                        if let Some(references_map) = reference_graph.get(&new_key) {
                            let ref_count = references_map.len();
                            match ref_count {
                                0 => {}
                                1 => {
                                    let Some(reference_to_fix) = references_map.iter().next().copied() else {
                                        continue;
                                    };

                                    reference_graph.remove(&reference_to_fix);
                                    if block_context.references_in_scope.contains_key(&reference_to_fix) {
                                        block_context.decrement_reference_count(reference_to_fix.as_bytes());
                                        block_context.references_in_scope.remove(&reference_to_fix);
                                    }
                                }
                                _ => {
                                    let references_to_fix = references_map.iter().copied().collect::<Vec<_>>();
                                    for reference in &references_to_fix {
                                        if let Some(inner_set) = reference_graph.get_mut(reference) {
                                            inner_set.remove(&new_key);
                                        }
                                    }

                                    if let Some(new_primary_reference) = reference_graph
                                        .get(&references_to_fix[0])
                                        .and_then(|inner_set| inner_set.iter().next().copied())
                                    {
                                        if block_context.references_in_scope.contains_key(&new_primary_reference) {
                                            block_context.decrement_reference_count(new_primary_reference.as_bytes());
                                            block_context.references_in_scope.remove(&new_primary_reference);
                                        }

                                        for referenced_value in block_context.references_in_scope.values_mut() {
                                            if *referenced_value == new_key {
                                                *referenced_value = new_primary_reference;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        reference_graph.remove(&new_key);
                        removable_keys.push(new_key);
                        if block_context.references_in_scope.contains_key(&new_key) {
                            block_context.decrement_reference_count(new_key.as_bytes());
                            block_context.references_in_scope.remove(&new_key);
                        }
                    }
                }

                for new_key in removable_keys {
                    block_context.locals.remove(&new_key);
                }
            }
        } else if !has_negation && !has_truthy_or_falsy_or_empty && !has_isset && !has_relational_comparison {
            changed_var_ids.insert(*key);
        }

        if !has_object_array_access {
            block_context.locals.insert(*key, Rc::new(result_type));
        }

        if !did_type_exist && !reference_graph.is_empty() {
            let mut key_parts = borrow_path_parts(key_str);
            let key_parts_0_atom = word(key_parts[0]);
            if let Some(existing_type) = block_context.locals.get(key).cloned()
                && let Some(references) = reference_graph.get(&key_parts_0_atom)
            {
                // Give new paths to variables that reference the same root.
                for reference in references {
                    key_parts[0] = reference.as_bytes();
                    let joined: Vec<u8> = key_parts.iter().flat_map(|part| part.iter()).copied().collect();
                    let reference_key = word(&joined);
                    block_context.locals.insert(reference_key, Rc::clone(&existing_type));
                }
            }
        }
    }
}

fn reconcile_optional_entry_type<A>(
    context: &mut Context<'_, '_, A>,
    new_type_parts: &AssertionSet,
    base_type: Option<&TUnion>,
    key_str: &[u8],
    inside_loop: bool,
    span: &Span,
    negated: bool,
) -> Option<TUnion>
where
    A: Arena,
{
    let mut result = base_type.cloned();
    let mut saw_mixed_disjunction = false;

    for pass in 0..2 {
        for disjuncts in new_type_parts {
            let value_assertions = disjuncts
                .iter()
                .filter(|assertion| !matches!(assertion, Assertion::IsNotIsset | Assertion::ArrayKeyDoesNotExist))
                .collect::<Vec<_>>();

            if value_assertions.is_empty() {
                continue;
            }

            if pass == 0 && value_assertions.len() != disjuncts.len() {
                saw_mixed_disjunction = true;
            }

            let mut orred_type: Option<TUnion> = None;
            for assertion in value_assertions {
                let candidate = assertion_reconciler::reconcile(
                    context,
                    assertion,
                    result.as_ref(),
                    Some(key_str),
                    inside_loop,
                    Some(span),
                    false,
                    negated,
                );

                orred_type = Some(add_optional_union_type(candidate, orred_type.as_ref(), context.codebase));
            }

            result = orred_type;
        }

        if result.as_ref().is_none_or(TUnion::is_never) {
            break;
        }
    }

    if !saw_mixed_disjunction {
        return None;
    }

    result.filter(|entry_type| !entry_type.is_never() && !entry_type.is_mixed())
}

fn adjust_array_type(
    mut key_parts: Vec<&[u8]>,
    context: &mut BlockContext<'_>,
    changed_var_ids: &mut WordSet,
    result_type: &TUnion,
    codebase: &CodebaseMetadata,
    optional: bool,
) {
    key_parts.pop();
    let Some(array_key) = key_parts.pop() else {
        return;
    };
    key_parts.pop();

    let base_key: Vec<u8> = key_parts.iter().flat_map(|part| part.iter()).copied().collect();
    let base_key_atom = word(&base_key);

    if array_key.starts_with(b"$") {
        // When the key is a variable, we can't narrow to a specific key,
        // but we CAN remove empty array variants since isset proves the array is non-empty.
        if let Some(existing_type) = context.locals.get(&base_key_atom).cloned()
            && existing_type.types.iter().any(|t| matches!(t, TAtomic::Array(a) if a.is_empty()))
        {
            let mut narrowed = (*existing_type).clone();
            narrowed.types.to_mut().retain(|t| !matches!(t, TAtomic::Array(a) if a.is_empty()));
            if !narrowed.types.is_empty() {
                context.locals.insert(base_key_atom, Rc::new(narrowed));
                changed_var_ids.insert(concat_word!(base_key.as_slice(), b"[", array_key, b"]"));
            }
        }
        return;
    }

    let mut has_string_offset = false;

    let arraykey_offset = if array_key.starts_with(b"'") || array_key.starts_with(b"\"") {
        has_string_offset = true;
        &array_key[1..(array_key.len() - 1)]
    } else {
        array_key
    };

    let mut existing_type = if let Some(existing_type) = context.locals.get(&base_key_atom) {
        (**existing_type).clone()
    } else {
        return;
    };

    let atomic_types = std::mem::take(existing_type.types.to_mut());
    let mut compatible_types = Vec::with_capacity(atomic_types.len());

    for mut base_atomic_type in atomic_types {
        match &mut base_atomic_type {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) => {
                let dictkey = if has_string_offset {
                    ArrayKey::String(word(arraykey_offset))
                } else if let Some(arraykey_value) =
                    std::str::from_utf8(arraykey_offset).ok().and_then(|s| s.parse::<i64>().ok())
                {
                    ArrayKey::Integer(arraykey_value)
                } else {
                    compatible_types.push(base_atomic_type);
                    continue;
                };

                if let Some(known_items) = known_items {
                    if let Some((existing_optional, existing_item_type)) = known_items.get(&dictkey) {
                        match intersect_union_types(result_type, existing_item_type, codebase) {
                            Some(intersected) if !intersected.is_never() => {
                                known_items.insert(dictkey, (*existing_optional, intersected));
                            }
                            _ => {
                                continue;
                            }
                        }
                    } else {
                        known_items.insert(dictkey, (optional, result_type.clone()));
                    }
                } else {
                    *known_items = Some(BTreeMap::from([(dictkey, (optional, result_type.clone()))]));
                }
            }
            TAtomic::Array(TArray::List(TList { known_elements, .. })) => {
                if let Some(arraykey_offset) =
                    std::str::from_utf8(arraykey_offset).ok().and_then(|s| s.parse::<usize>().ok())
                {
                    if let Some(known_elements) = known_elements {
                        if let Some((_, existing_item_type)) = known_elements.get(&arraykey_offset) {
                            match intersect_union_types(result_type, existing_item_type, codebase) {
                                Some(intersected) if !intersected.is_never() => {
                                    known_elements.insert(arraykey_offset, (false, intersected));
                                }
                                _ => {
                                    continue;
                                }
                            }
                        } else {
                            known_elements.insert(arraykey_offset, (optional, result_type.clone()));
                        }
                    } else {
                        *known_elements = Some(BTreeMap::from([(arraykey_offset, (optional, result_type.clone()))]));
                    }
                }
            }
            TAtomic::Mixed(_) => {
                let key = if has_string_offset {
                    ArrayKey::String(word(arraykey_offset))
                } else if let Some(arraykey_value) =
                    std::str::from_utf8(arraykey_offset).ok().and_then(|s| s.parse::<i64>().ok())
                {
                    ArrayKey::Integer(arraykey_value)
                } else {
                    compatible_types.push(base_atomic_type);
                    continue;
                };

                base_atomic_type = TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: Some(BTreeMap::from([(key, (optional, result_type.clone()))])),
                    parameters: Some((Arc::new(get_arraykey()), Arc::new(get_mixed()))),
                    non_empty: !optional,
                    known_non_list: false,
                }));
            }
            _ => {
                if base_atomic_type.is_object_type()
                    || base_atomic_type.is_string()
                    || base_atomic_type.is_generic_parameter()
                    || matches!(base_atomic_type, TAtomic::Variable(_))
                {
                    compatible_types.push(base_atomic_type);
                }

                continue;
            }
        }

        changed_var_ids.insert(concat_word!(base_key.as_slice(), b"[", array_key, b"]"));

        if let Some(last_part) = key_parts.last()
            && *last_part == b"]"
        {
            adjust_array_type(
                key_parts.clone(),
                context,
                changed_var_ids,
                &wrap_atomic(base_atomic_type.clone()),
                codebase,
                false,
            );
        }

        compatible_types.push(base_atomic_type);
    }

    if compatible_types.is_empty() {
        return;
    }

    *existing_type.types.to_mut() = compatible_types;
    context.locals.insert(base_key_atom, Rc::new(existing_type));
}

fn adjust_array_type_remove_key(
    mut key_parts: Vec<&[u8]>,
    context: &mut BlockContext<'_>,
    changed_var_ids: &mut WordSet,
    codebase: &CodebaseMetadata,
) {
    key_parts.pop();
    let Some(array_key) = key_parts.pop() else {
        return;
    };

    key_parts.pop();

    if array_key.starts_with(b"$") {
        return;
    }

    let mut has_string_offset = false;

    let arraykey_offset = if array_key.starts_with(b"'") || array_key.starts_with(b"\"") {
        has_string_offset = true;
        &array_key[1..(array_key.len() - 1)]
    } else {
        array_key
    };

    let base_key: Vec<u8> = key_parts.iter().flat_map(|part| part.iter()).copied().collect();
    let base_key_atom = word(&base_key);

    let mut existing_type = if let Some(existing_type) = context.locals.get(&base_key_atom) {
        (**existing_type).clone()
    } else {
        return;
    };

    for base_atomic_type in existing_type.types.to_mut() {
        match base_atomic_type {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) => {
                let dictkey = if has_string_offset {
                    ArrayKey::String(word(arraykey_offset))
                } else if let Some(arraykey_value) =
                    std::str::from_utf8(arraykey_offset).ok().and_then(|s| s.parse::<i64>().ok())
                {
                    ArrayKey::Integer(arraykey_value)
                } else {
                    continue;
                };

                if let Some(known_items) = known_items {
                    known_items.remove(&dictkey);
                }
            }
            TAtomic::Array(TArray::List(TList { known_elements, .. })) => {
                if let Some(arraykey_offset) =
                    std::str::from_utf8(arraykey_offset).ok().and_then(|s| s.parse::<usize>().ok())
                    && let Some(known_elements) = known_elements
                {
                    known_elements.remove(&arraykey_offset);
                }
            }
            _ => {
                continue;
            }
        }

        changed_var_ids.insert(concat_word!(base_key.as_slice(), b"[", array_key, b"]"));

        if let Some(last_part) = key_parts.last()
            && *last_part == b"]"
        {
            adjust_array_type(
                key_parts.clone(),
                context,
                changed_var_ids,
                &wrap_atomic(base_atomic_type.clone()),
                codebase,
                false,
            );
        }
    }

    context.locals.insert(base_key_atom, Rc::new(existing_type));
}

/// Filters a union of object types based on a narrowed property type.
///
/// When `$input->myProp` is narrowed (e.g., via `is_string()`), this function
/// checks each object variant to see if its declared property type is compatible
/// with the narrowed type, and removes incompatible variants.
fn adjust_object_property_type<A>(
    mut key_parts: Vec<&[u8]>,
    block_context: &mut BlockContext<'_>,
    changed_var_ids: &mut WordSet,
    result_type: &TUnion,
    context: &Context<'_, '_, A>,
) where
    A: Arena,
{
    let Some(property_name) = key_parts.pop() else {
        return;
    };

    let Some(divider) = key_parts.pop() else {
        return;
    };

    if divider != b"->" {
        return;
    }

    let base_key: Vec<u8> = key_parts.iter().flat_map(|part| part.iter()).copied().collect();
    let base_key_atom = word(&base_key);

    let mut existing_type = if let Some(existing_type) = block_context.locals.get(&base_key_atom) {
        (**existing_type).clone()
    } else {
        return;
    };

    let atomic_types = std::mem::take(existing_type.types.to_mut());
    let original_len = atomic_types.len();
    let mut compatible_types = Vec::with_capacity(original_len);

    for base_atomic_type in atomic_types {
        let should_check = match &base_atomic_type {
            TAtomic::Object(TObject::Named(named)) => {
                let fq_class_name = named.get_name();
                if fq_class_name.as_bytes().eq_ignore_ascii_case(b"stdClass")
                    || !context.codebase.class_or_interface_exists(fq_class_name.as_bytes())
                {
                    None
                } else {
                    get_property_type(
                        context,
                        named.get_name(),
                        property_name,
                        true, // `instance_access`: assertions on `$obj->prop`
                        block_context.scope.get_class_like_name(),
                    )
                }
            }
            _ => None,
        };

        if let Some(declared_property_type) = should_check {
            match intersect_union_types(result_type, &declared_property_type, context.codebase) {
                Some(intersected) if !intersected.is_never() => {
                    compatible_types.push(base_atomic_type);
                }
                _ => {}
            }
        } else {
            compatible_types.push(base_atomic_type);
        }
    }

    if !compatible_types.is_empty() && compatible_types.len() < original_len {
        *existing_type.types.to_mut() = compatible_types;
        block_context.locals.insert(base_key_atom, Rc::new(existing_type));
        changed_var_ids.insert(base_key_atom);
    }
}

fn refine_array_key(key_type: &TUnion) -> TUnion {
    fn refine_array_key_inner(key_type: &TUnion) -> Option<TUnion> {
        let mut refined = false;
        let mut types = vec![];

        for cat in key_type.types.as_ref() {
            match cat {
                TAtomic::GenericParameter(param) => {
                    if let Some(as_type) = refine_array_key_inner(&param.constraint) {
                        refined = true;
                        types.push(TAtomic::GenericParameter(param.with_constraint(as_type)));
                    } else {
                        types.push(cat.clone());
                    }
                }
                TAtomic::Scalar(TScalar::ArrayKey | TScalar::String(_) | TScalar::Integer(_)) => {
                    types.push(cat.clone());
                }
                _ => {
                    refined = true;
                    types.push(TAtomic::Scalar(TScalar::ArrayKey));
                }
            }
        }

        if refined { Some(TUnion::from_vec(types)) } else { None }
    }

    refine_array_key_inner(key_type).unwrap_or_else(|| key_type.clone())
}

static INTEGER_REGEX: LazyLock<Regex> = LazyLock::new(|| unsafe {
    // SAFETY: `unwrap_unchecked` is safe here because the regex is valid and will not panic.
    Regex::new("^[0-9]+$").unwrap_unchecked()
});

#[allow(clippy::multiple_unsafe_ops_per_block)]
#[allow(clippy::semicolon_inside_block)]
fn add_nested_assertions(
    new_types: &mut Cow<'_, AssertionMap<Word, AssertionSet>>,
    active_new_types: &mut AssertionMap<Word, HashSet<usize>>,
    context: &BlockContext<'_>,
) {
    let nested_assertions = new_types
        .iter()
        .filter(|(key, assertions)| {
            let bytes = key.as_bytes();
            (bytes.contains(&b'[') || memchr::memmem::find(bytes, b"->").is_some())
                && matches!(assertions[0][0], Assertion::IsEqualIsset | Assertion::IsIsset)
        })
        .map(|(key, assertions)| {
            let only_isset_assertions = assertions.iter().all(|clause| {
                clause.iter().all(|assertion| matches!(assertion, Assertion::IsIsset | Assertion::IsEqualIsset))
            });

            (*key, only_isset_assertions)
        })
        .collect::<Vec<_>>();

    if nested_assertions.is_empty() {
        return;
    }

    let new_types = new_types.to_mut();
    let mut keys_to_remove = vec![];

    'outer: for (nk, only_isset_assertions) in nested_assertions {
        let nk_str = nk.as_bytes();
        let mut key_parts = borrow_path_parts(nk_str);
        key_parts.reverse();

        let mut nesting = 0;
        let mut base_key: Vec<u8>;

        unsafe {
            // SAFETY: `pop` will always return a value because we checked that the key contains either `[` or `->`.
            base_key = key_parts.pop().unwrap_unchecked().to_vec();

            if !base_key.starts_with(b"$") && key_parts.len() > 2 && *key_parts.last().unwrap_unchecked() == b"::$" {
                base_key.extend_from_slice(key_parts.pop().unwrap_unchecked());
                base_key.extend_from_slice(key_parts.pop().unwrap_unchecked());
            }
        }

        let base_key_atom = word(&base_key);
        let base_key_set = if let Some(base_key_type) = context.locals.get(&base_key_atom) {
            !base_key_type.is_nullable()
                && !base_key_type.possibly_undefined()
                && !base_key_type.possibly_undefined_from_try()
        } else {
            false
        };

        if !base_key_set {
            new_types.insert(
                base_key_atom,
                if let Some(mut existing_entry) = new_types.get(&base_key_atom).cloned() {
                    existing_entry.push(vec![Assertion::IsEqualIsset]);
                    existing_entry
                } else {
                    vec![vec![Assertion::IsEqualIsset]]
                },
            );
        }

        while let Some(divider) = key_parts.pop() {
            if divider == b"[" {
                let array_key = unsafe {
                    // SAFETY: we know that after `[` there is always an array key, so `pop` will not panic.
                    key_parts.pop().unwrap_unchecked()
                };

                key_parts.pop();

                let mut new_base_key = base_key.clone();
                new_base_key.push(b'[');
                new_base_key.extend_from_slice(array_key);
                new_base_key.push(b']');
                let base_key_atom = word(&base_key);
                let entry = new_types.entry(base_key_atom).or_default();
                let new_key = if array_key.starts_with(b"'") || array_key.starts_with(b"\"") {
                    Some(ArrayKey::String(word(&array_key[1..(array_key.len() - 1)])))
                } else if array_key.starts_with(b"$") {
                    None
                } else if let Some(arraykey_value) =
                    std::str::from_utf8(array_key).ok().and_then(|s| s.parse::<i64>().ok())
                {
                    Some(ArrayKey::Integer(arraykey_value))
                } else {
                    continue 'outer;
                };

                if let Some(new_key) = new_key {
                    entry.push(vec![Assertion::HasNonnullEntryForKey(new_key)]);

                    if key_parts.is_empty() {
                        // Only remove the nested key if it contains ONLY isset-related assertions
                        // If it has other assertions (e.g., IsType from is_string()), keep them
                        if only_isset_assertions {
                            keys_to_remove.push(nk);

                            if nesting == 0 && base_key_set && active_new_types.swap_remove(&nk).is_some() {
                                active_new_types.entry(base_key_atom).or_default().insert(entry.len() - 1);
                            }

                            continue 'outer;
                        }
                    }
                } else {
                    entry.push(vec![Assertion::HasIntOrStringArrayAccess]);
                }

                base_key = new_base_key;
                nesting += 1;
                continue;
            }

            if divider == b"->" {
                let property_name = unsafe {
                    // SAFETY: we know that after `->` there is always a property name, so `pop` will not panic.
                    key_parts.pop().unwrap_unchecked()
                };

                let mut new_base_key = base_key.clone();
                new_base_key.extend_from_slice(b"->");
                new_base_key.extend_from_slice(property_name);
                let base_key_atom = word(&base_key);

                if !new_types.contains_key(&base_key_atom) {
                    new_types.insert(base_key_atom, vec![vec![Assertion::IsIsset]]);
                }

                base_key = new_base_key;
            } else {
                break;
            }

            if key_parts.is_empty() {
                break;
            }
        }
    }

    new_types.retain(|k, _| !keys_to_remove.contains(k));
}

#[cfg(test)]
fn break_up_path_into_parts(path: &[u8]) -> Vec<Vec<u8>> {
    let mut parts = Vec::new();
    for_each_path_part(path, |part| parts.push(part.to_vec()));
    parts
}

fn borrow_path_parts(path: &[u8]) -> Vec<&[u8]> {
    let mut parts = Vec::new();
    for_each_path_part(path, |part| parts.push(part));
    parts
}

fn path_part_count(path: &[u8]) -> usize {
    let mut count = 0;
    for_each_path_part(path, |_| count += 1);
    count
}

fn for_each_path_part<'path>(path: &'path [u8], mut visit: impl FnMut(&'path [u8])) {
    if path.is_empty() {
        visit(path);
        return;
    }

    let mut quote = None;
    let mut escaped = false;
    let mut brackets = 0i32;
    let mut start = 0;
    let mut i = 0;
    while i < path.len() {
        let token_start = i;
        let byte = path[i];
        i += 1;
        if let Some(quote_char) = quote {
            if byte == quote_char && !escaped {
                quote = None;
            }
            escaped = byte == b'\\' && !escaped;
            continue;
        }

        let is_token = match byte {
            b'[' => {
                let is_token = brackets == 0;
                brackets += 1;
                is_token
            }
            b']' => {
                brackets -= 1;
                brackets == 0
            }
            b'\'' | b'"' => {
                quote = Some(byte);
                false
            }
            b':' if brackets == 0 && path.get(i..i + 2) == Some(b":$") => {
                i += 2;
                true
            }
            b'-' if brackets == 0 && path.get(i) == Some(&b'>') => {
                i += 1;
                true
            }
            _ => false,
        };

        if is_token {
            if start < token_start {
                visit(&path[start..token_start]);
            }
            visit(&path[token_start..i]);
            start = i;
        }
    }

    if start < path.len() {
        visit(&path[start..]);
    }
}

#[allow(clippy::multiple_unsafe_ops_per_block)]
#[allow(clippy::semicolon_inside_block)]
fn get_value_for_key<A>(
    context: &Context<'_, '_, A>,
    key: Word,
    block_context: &mut BlockContext<'_>,
    new_assertions: &AssertionMap<Word, AssertionSet>,
    has_isset: bool,
    has_inverted_isset: bool,
    has_inverted_key_exists: bool,
    has_empty: bool,
    inside_loop: bool,
    has_object_array_access: &mut bool,
) -> Option<TUnion>
where
    A: Arena,
{
    let key_str = key.as_bytes();
    if memchr::memchr3(b'[', b'-', b':', key_str).is_none() {
        return block_context.locals.get(&key).map(|ty| (**ty).clone());
    }

    let mut key_parts = borrow_path_parts(key_str);
    if key_parts.is_empty() {
        return None;
    }

    if key_parts.len() == 1 {
        if let Some(t) = block_context.locals.get(&key) {
            return Some((**t).clone());
        }

        return None;
    }

    key_parts.reverse();

    let mut base_key: Vec<u8>;

    unsafe {
        // SAFETY: `pop` will always return a value because we checked that the key has more than one part.
        base_key = key_parts.pop().unwrap_unchecked().to_vec();

        if !base_key.starts_with(b"$")
            && key_parts.len() > 2
            && key_parts.last().is_some_and(|part| part.starts_with(b"::$"))
        {
            // SAFETY: `pop` will always return a value because we checked that the key has more than two parts.
            base_key.extend_from_slice(key_parts.pop().unwrap_unchecked());
            base_key.extend_from_slice(key_parts.pop().unwrap_unchecked());
        }
    }

    let base_key_atom = word(&base_key);
    if let std::collections::hash_map::Entry::Vacant(e) = block_context.locals.entry(base_key_atom) {
        let sep = memchr::memmem::find(&base_key, b"::")?;
        let fq_class_name = &base_key[..sep];
        let const_name = &base_key[sep + 2..];

        if !context.codebase.class_like_exists(fq_class_name) {
            return None;
        }

        let class_constant = context.codebase.get_class_constant_type(fq_class_name, const_name);

        let class_constant = class_constant?;
        let class_constant = Rc::new(match class_constant {
            Cow::Borrowed(t) => t.clone(),
            Cow::Owned(t) => t,
        });

        e.insert(class_constant);
    }

    let mut base_key_atom = word(&base_key);
    while let Some(divider) = key_parts.pop() {
        let base_key_type = Rc::clone(block_context.locals.get(&base_key_atom)?);

        if divider == b"[" {
            let array_key = key_parts.pop()?;

            key_parts.pop();

            let array_key_offset = std::str::from_utf8(array_key)
                .ok()
                .and_then(|s| if INTEGER_REGEX.is_match(s) { s.parse::<usize>().ok() } else { None });

            let array_key_type = if let Some(array_key_offset) = array_key_offset {
                ArrayKey::Integer(array_key_offset as i64)
            } else if array_key.starts_with(b"'") || array_key.starts_with(b"\"") {
                ArrayKey::String(word(&array_key[1..(array_key.len() - 1)]))
            } else {
                ArrayKey::String(word(array_key))
            };

            let mut new_base_key = base_key.clone();
            new_base_key.push(b'[');
            new_base_key.extend_from_slice(array_key);
            new_base_key.push(b']');
            let new_base_key_atom = word(&new_base_key);

            if !block_context.locals.contains_key(&new_base_key_atom) {
                let mut new_base_type: Option<Rc<TUnion>> = None;
                let mut atomic_types: Vec<_> = base_key_type.types.iter().collect();

                atomic_types.reverse();
                while let Some(existing_key_type_part) = atomic_types.pop() {
                    if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = existing_key_type_part {
                        atomic_types.extend(constraint.types.iter());
                        continue;
                    }

                    let mut new_base_type_candidate;

                    if let TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) = existing_key_type_part {
                        if has_empty {
                            return None;
                        }

                        let known_item = if !array_key.starts_with(b"$")
                            && let Some(known_items) = known_items
                        {
                            known_items.get(&array_key_type)
                        } else {
                            None
                        };

                        if let Some(known_item) = known_item {
                            new_base_type_candidate = known_item.1.clone();

                            if known_item.0 {
                                new_base_type_candidate.set_possibly_undefined(true, None);
                            }
                        } else {
                            if has_empty {
                                return None;
                            }

                            new_base_type_candidate =
                                get_iterable_value_parameter(existing_key_type_part, context.codebase)?;

                            if new_base_type_candidate.is_mixed()
                                && !has_isset
                                && !has_inverted_isset
                                && !has_inverted_key_exists
                            {
                                return Some(new_base_type_candidate);
                            }

                            if (has_isset || has_inverted_isset || has_inverted_key_exists)
                                && new_assertions.contains_key(&new_base_key_atom)
                            {
                                if has_inverted_isset && new_base_key_atom == key {
                                    new_base_type_candidate = add_union_type(
                                        new_base_type_candidate,
                                        &get_null(),
                                        context.codebase,
                                        CombinerOptions::default(),
                                    );
                                }

                                new_base_type_candidate.set_possibly_undefined(true, None);
                            }
                        }
                    } else if let TAtomic::Array(TArray::List(TList { known_elements, .. })) = existing_key_type_part {
                        if has_empty {
                            return None;
                        }

                        let known_item = if let Some(known_items) = known_elements
                            && let Some(array_key_offset) = array_key_offset
                        {
                            known_items.get(&array_key_offset)
                        } else {
                            None
                        };

                        if let Some(known_item) = known_item {
                            new_base_type_candidate = known_item.1.clone();

                            if known_item.0 {
                                new_base_type_candidate.set_possibly_undefined(true, None);
                            }
                        } else {
                            new_base_type_candidate =
                                get_iterable_value_parameter(existing_key_type_part, context.codebase)?;

                            if (has_isset || has_inverted_isset || has_inverted_key_exists)
                                && new_assertions.contains_key(&new_base_key_atom)
                            {
                                if has_inverted_isset && new_base_key_atom == key {
                                    new_base_type_candidate = add_union_type(
                                        new_base_type_candidate,
                                        &get_null(),
                                        context.codebase,
                                        CombinerOptions::default(),
                                    );
                                }

                                new_base_type_candidate.set_possibly_undefined(true, None);
                            }
                        }
                    } else if matches!(existing_key_type_part, TAtomic::Scalar(TScalar::String(_))) {
                        new_base_type_candidate = get_string();
                    } else if existing_key_type_part.is_never() || existing_key_type_part.is_mixed_isset_from_loop() {
                        return Some(get_mixed_maybe_from_loop(inside_loop));
                    } else if let TAtomic::Object(TObject::Named(_named_object)) = existing_key_type_part {
                        if has_isset || has_inverted_isset || has_inverted_key_exists {
                            *has_object_array_access = true;
                            block_context.locals.remove(&new_base_key_atom);

                            return None;
                        }

                        new_base_type_candidate = get_mixed();
                    } else {
                        continue;
                    }

                    let resulting_type = Rc::new(if let Some(new_base_type) = &new_base_type {
                        add_union_type(
                            new_base_type_candidate,
                            new_base_type,
                            context.codebase,
                            CombinerOptions::default(),
                        )
                    } else {
                        new_base_type_candidate
                    });

                    new_base_type = Some(Rc::clone(&resulting_type));
                    block_context.locals.insert(new_base_key_atom, resulting_type);
                }
            }

            base_key = new_base_key;
            base_key_atom = new_base_key_atom;
        } else if divider == b"->" || divider == b"::$" {
            let property_name = key_parts.pop()?;
            let mut new_base_key = base_key.clone();
            new_base_key.extend_from_slice(b"->");
            new_base_key.extend_from_slice(property_name);
            let new_base_key_atom = word(&new_base_key);

            if !block_context.locals.contains_key(&new_base_key_atom) {
                let mut new_base_type: Option<Rc<TUnion>> = None;
                let mut atomic_types: Vec<_> = base_key_type.types.iter().collect();

                while let Some(existing_key_type_part) = atomic_types.pop() {
                    if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = existing_key_type_part {
                        atomic_types.extend(constraint.types.iter());
                        continue;
                    }

                    let class_property_type: TUnion;

                    if *existing_key_type_part == TAtomic::Null {
                        class_property_type = get_null();
                        // TODO(azjezz): maybe we should exclude mixed from isset in loop?
                    } else if let TAtomic::Mixed(_) | TAtomic::GenericParameter(_) | TAtomic::Object(TObject::Any) =
                        existing_key_type_part
                    {
                        class_property_type = get_mixed();
                    } else if let TAtomic::Object(TObject::Named(named_object)) = existing_key_type_part {
                        let fq_class_name = named_object.get_name();

                        if fq_class_name.as_bytes().eq_ignore_ascii_case(b"stdClass")
                            || !context.codebase.class_or_interface_exists(fq_class_name.as_bytes())
                        {
                            class_property_type = get_mixed();
                        } else {
                            class_property_type = get_property_type(
                                context,
                                fq_class_name,
                                property_name,
                                divider == b"->",
                                block_context.scope.get_class_like_name(),
                            )?;
                        }
                    } else {
                        class_property_type = get_mixed();
                    }

                    let resulting_type = Rc::new(add_optional_union_type(
                        class_property_type,
                        new_base_type.as_deref(),
                        context.codebase,
                    ));

                    new_base_type = Some(Rc::clone(&resulting_type));
                    block_context.locals.insert(new_base_key_atom, resulting_type);
                }
            }

            base_key = new_base_key;
            base_key_atom = new_base_key_atom;
        } else {
            return None;
        }
    }

    block_context.locals.get(&base_key_atom).map(|t| (**t).clone())
}

fn get_property_type<A>(
    context: &Context<'_, '_, A>,
    classlike_name: Word,
    property_name_str: &[u8],
    instance_access: bool,
    scope: Option<Word>,
) -> Option<TUnion>
where
    A: Arena,
{
    // Add `$` prefix
    let property_name = concat_word!(b"$", property_name_str);

    let class_metadata = context.codebase.get_class_like(classlike_name.as_bytes())?;
    let resolution =
        resolve_declared_property(context.codebase, class_metadata, property_name, instance_access, scope)?;
    let declaring_property_class = resolution.declaring_class.name;

    let mut property_type = resolution.declared_type(context.codebase);
    expander::expand_union(
        context.codebase,
        &mut property_type,
        &TypeExpansionOptions {
            self_class: Some(declaring_property_class),
            static_class_type: StaticClassType::Name(declaring_property_class),
            ..Default::default()
        },
    );

    Some(property_type)
}

pub(crate) fn trigger_issue_for_impossible<A>(
    context: &mut Context<'_, '_, A>,
    old_var_type_string: Word,
    key: &[u8],
    assertion: &Assertion,
    redundant: bool,
    negated: bool,
    span: &Span,
) where
    A: Arena,
{
    let mut assertion_atom = assertion.to_atom();
    let mut not_operator = assertion_atom.as_bytes().starts_with(b"!");

    if not_operator {
        assertion_atom = word(&assertion_atom.as_bytes()[1..]);
    }

    let mut redundant = redundant;
    if negated {
        not_operator = !not_operator;
        redundant = !redundant;
    }

    if redundant {
        if not_operator {
            if assertion_atom.as_bytes() == b"falsy" {
                not_operator = false;
                assertion_atom = word(b"truthy");
            } else if assertion_atom.as_bytes() == b"truthy" {
                not_operator = false;
                assertion_atom = word(b"falsy");
            }
        }

        if not_operator {
            report_impossible_issue(context, assertion, assertion_atom, key, span, old_var_type_string);
        } else {
            report_redundant_issue(context, assertion, assertion_atom, key, span, old_var_type_string);
        }
    } else if not_operator {
        report_redundant_issue(context, assertion, assertion_atom, key, span, old_var_type_string);
    } else {
        report_impossible_issue(context, assertion, assertion_atom, key, span, old_var_type_string);
    }
}

fn report_impossible_issue<A>(
    context: &mut Context<'_, '_, A>,
    assertion: &Assertion,
    assertion_atom: Word,
    key: &[u8],
    span: &Span,
    old_var_type_string: Word,
) where
    A: Arena,
{
    let key = BytesDisplay(key);
    let subject_desc = if old_var_type_string.is_empty() || old_var_type_string.len() > 50 {
        format!("`{key}`")
    } else {
        format!("`{key}` (type `{old_var_type_string}`)")
    };

    let (issue_kind, main_message_verb, specific_note, specific_help) = match assertion {
        Assertion::Truthy => (
            IssueCode::ImpossibleCondition,
            "will always evaluate to false".to_owned(),
            format!("Variable {subject_desc} is always falsy and can never satisfy a truthiness check."),
            "Review the logic or type of the variable; this condition will never pass.".to_string(),
        ),
        Assertion::Falsy => (
            IssueCode::ImpossibleCondition,
            "will always evaluate to false".to_owned(),
            format!("Variable {subject_desc} is always truthy, so asserting it is falsy will always be false."),
            "Review the logic or type of the variable; this condition will never pass.".to_string(),
        ),
        Assertion::IsType(TAtomic::Null) => (
            IssueCode::ImpossibleNullTypeComparison,
            "can never be `null`".to_owned(),
            format!("Variable {subject_desc} does not include `null`."),
            format!(
                "The condition checking if `{key}` is `null` will always be false. Remove or refactor the condition.",
            ),
        ),
        Assertion::IsNotType(TAtomic::Null) => (
            IssueCode::ImpossibleNullTypeComparison,
            "will always be `null`".to_owned(),
            format!("Variable {subject_desc} is already known to be `null`, so asserting it's not `null` is impossible."),
            format!("The condition checking if `{key}` is not `null` will always be false. Review the variable's state or condition."),
        ),
        Assertion::HasArrayKey(array_key_assertion) => (
            IssueCode::ImpossibleKeyCheck,
            format!("can never have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to not contain the key `{array_key_assertion}`. This check will always be false."),
            "Ensure the array structure and key are correct, or remove this condition.".to_owned(),
        ),
        Assertion::DoesNotHaveArrayKey(array_key_assertion) => (
            IssueCode::ImpossibleKeyCheck,
            format!("will always have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to always contain the key `{array_key_assertion}`. Asserting it doesn't have this key will always be false."),
            "Review the logic; this negative key check will always fail.".to_owned(),
        ),
        Assertion::HasNonnullEntryForKey(dict_key_name) => (
            IssueCode::ImpossibleNonnullEntryCheck,
            format!("can never have a non-null entry for key `{dict_key_name}`"),
            format!("Variable {subject_desc} is known to either not have the key `{dict_key_name}` or its value is always `null`. This check for a non-null entry will always be false."),
            "Verify the array/object structure or remove this `!empty()` style check.".to_owned(),
        ),
        _ => (
            IssueCode::ImpossibleTypeComparison,
            format!("can never be `{assertion_atom}`"),
            format!("The type of variable {subject_desc} is incompatible with the assertion that it is `{assertion_atom}`."),
            "This condition is impossible and the associated code block will never execute. Review the types and condition logic.".to_owned(),
        ),
    };

    context.collector.report_with_code(
        issue_kind,
        Issue::warning(format!("Impossible condition: variable {subject_desc} {main_message_verb}."))
            .with_annotation(
                Annotation::primary(*span).with_message("This condition always evaluates to false".to_string()),
            )
            .with_note(specific_note)
            .with_help(specific_help),
    );
}

fn report_redundant_issue<A>(
    context: &mut Context<'_, '_, A>,
    assertion: &Assertion,
    assertion_atom: Word,
    key: &[u8],
    span: &Span,
    old_var_type_string: Word,
) where
    A: Arena,
{
    let key = BytesDisplay(key);
    let subject_desc = if old_var_type_string.is_empty() || old_var_type_string.len() > 50 {
        format!("`{key}`")
    } else {
        format!("`{key}` (type `{old_var_type_string}`)")
    };

    let (issue_kind, main_message_verb, specific_note, specific_help) = match assertion {
        Assertion::IsIsset | Assertion::IsEqualIsset => (
            IssueCode::RedundantIssetCheck,
            "is always considered set (not null)".to_owned(),
            format!("Variable {subject_desc} is already known to be non-null, making the `isset()` check redundant."),
            "Remove the redundant `isset()` check.".to_owned()
        ),
        Assertion::Truthy => (
            IssueCode::RedundantCondition,
            "will always evaluate to true".to_owned(),
            format!("Variable {subject_desc} is always truthy. This condition is redundant and the code block will always execute if reached."),
            "Simplify or remove the redundant condition if the guarded code should always run.".to_owned()
        ),
        Assertion::Falsy => (
            IssueCode::RedundantCondition,
            "will always evaluate to true".to_owned(),
            format!("Variable {subject_desc} is always falsy, so asserting it's falsy is always true and redundant."),
            "Simplify or remove the redundant condition if the guarded code should always run.".to_owned()
        ),
        Assertion::HasArrayKey(array_key_assertion) => (
            IssueCode::RedundantKeyCheck,
            format!("will always have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to always contain the key `{array_key_assertion}`. This check is redundant."),
            "Remove the redundant `array_key_exists()` or key check.".to_owned()
        ),
        Assertion::DoesNotHaveArrayKey(array_key_assertion) => (
            IssueCode::RedundantKeyCheck,
            format!("will never have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to never contain the key `{array_key_assertion}`. This negative check is redundant."),
            "Remove the redundant negative key check.".to_owned()
        ),
        Assertion::HasNonnullEntryForKey(dict_key_name) => (
            IssueCode::RedundantNonnullEntryCheck,
            format!("will always have a non-null entry for key `{dict_key_name}`"),
            format!("Variable {subject_desc} is known to always have a non-null value for key `{dict_key_name}`. This `!empty()` style check is redundant."),
            "Remove the redundant non-null entry check.".to_owned()
        ),
        Assertion::IsType(TAtomic::Mixed(mixed)) if mixed.is_non_null() => (
            IssueCode::RedundantNonnullTypeComparison,
            "is already known to be non-null".to_owned(),
            format!("Variable {subject_desc} is already non-null. Checking against `mixed (not null)` is redundant."),
            "Remove the redundant non-null check.".to_owned()
        ),
        Assertion::IsNotType(TAtomic::Mixed(mixed)) if mixed.is_non_null() => (
            IssueCode::RedundantTypeComparison,
            "comparison with `mixed (not null)` is redundant".to_owned(),
            format!("The check against `mixed (not null)` for variable {subject_desc} might be overly broad or redundant depending on context."),
            "Verify if a more specific type check is needed.".to_owned()
        ),
        _ => (
            IssueCode::RedundantTypeComparison,
            format!("is already known to be `{assertion_atom}`"),
            format!("The type of variable {subject_desc} already satisfies the condition that it is `{assertion_atom}`. This check is redundant."),
            "This condition is always true and the associated code block will always execute if reached. Consider simplifying.".to_owned()
        ),
    };

    context.collector.report_with_code(
        issue_kind,
        Issue::help(format!("Redundant condition: variable {subject_desc} {main_message_verb}."))
            .with_annotation(
                Annotation::primary(*span).with_message("This condition always evaluates to true".to_string()),
            )
            .with_note(specific_note)
            .with_help(specific_help),
    );
}

fn map_generic_constraint<F>(generic_parameter: &TGenericParameter, f: F) -> Option<TAtomic>
where
    F: FnOnce(&TUnion) -> TUnion,
{
    let parameter = generic_parameter.with_constraint(f(&generic_parameter.constraint));

    if parameter.constraint.is_never() { None } else { Some(TAtomic::GenericParameter(parameter)) }
}

fn map_concrete_generic_constraint<F>(generic_parameter: &TGenericParameter, f: F) -> Option<TAtomic>
where
    F: FnOnce(&TUnion) -> TUnion,
{
    map_generic_constraint_or_else(generic_parameter, || (*generic_parameter.constraint).clone(), f)
}

pub(crate) fn map_generic_constraint_or_else<F, D>(generic_parameter: &TGenericParameter, d: D, f: F) -> Option<TAtomic>
where
    F: FnOnce(&TUnion) -> TUnion,
    D: FnOnce() -> TUnion,
{
    let parameter = if generic_parameter.constraint.is_mixed() {
        generic_parameter.with_constraint(d())
    } else {
        generic_parameter.with_constraint(f(&generic_parameter.constraint))
    };

    if parameter.constraint.is_never() { None } else { Some(TAtomic::GenericParameter(parameter)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consecutive_tokens() {
        let path = b"$service_name->prop[0]->foo::$prop";
        let expected: Vec<&[u8]> =
            vec![b"$service_name", b"->", b"prop", b"[", b"0", b"]", b"->", b"foo", b"::$", b"prop"];
        let result = break_up_path_into_parts(path);
        assert_eq!(result, expected);
    }

    #[test]
    fn path_parts_preserve_quoted_nested_and_incomplete_paths() {
        let cases: &[(&[u8], &[&[u8]])] = &[
            (b"", &[b""]),
            (b"$plain", &[b"$plain"]),
            (b"$a[0]", &[b"$a", b"[", b"0", b"]"]),
            (b"$a[$b[0]]->value", &[b"$a", b"[", b"$b[0]", b"]", b"->", b"value"]),
            (b"$a['x->y[0]']", &[b"$a", b"[", b"'x->y[0]'", b"]"]),
            (b"$a[\"x::$y\"]", &[b"$a", b"[", b"\"x::$y\"", b"]"]),
            (b"$a['it\\'s']->value", &[b"$a", b"[", b"'it\\'s'", b"]", b"->", b"value"]),
            (b"$a['x\\\\']->value", &[b"$a", b"[", b"'x\\\\'", b"]", b"->", b"value"]),
            (b"Class::$value[0]", &[b"Class", b"::$", b"value", b"[", b"0", b"]"]),
            (b"Class::VALUE", &[b"Class::VALUE"]),
            (b"[]->::$", &[b"[", b"]", b"->", b"::$"]),
            (b"$a['unterminated", &[b"$a", b"[", b"'unterminated"]),
            (b"$a[$b[0]", &[b"$a", b"[", b"$b[0]"]),
            (b"$a]->x", &[b"$a]->x"]),
            (b"$a['\xff']", &[b"$a", b"[", b"'\xff'", b"]"]),
        ];

        for (path, expected) in cases {
            assert_eq!(break_up_path_into_parts(path), *expected, "{path:?}");
            assert_eq!(borrow_path_parts(path), *expected, "{path:?}");
            assert_eq!(path_part_count(path), expected.len(), "{path:?}");
        }
    }
}
