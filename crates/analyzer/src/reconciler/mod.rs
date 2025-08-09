use std::borrow::Cow;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::LazyLock;

use ahash::HashSet;
use regex::Regex;

use mago_codex::assertion::Assertion;
use mago_codex::class_like_exists;
use mago_codex::class_or_interface_exists;
use mago_codex::get_class_constant_type;
use mago_codex::get_declaring_class_for_property;
use mago_codex::get_property;
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
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_iterable_value_parameter;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_collector::Collector;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::Code;
use crate::context::block::BlockContext;
use crate::context::scope::var_has_root;

pub mod assertion_reconciler;
pub mod negated_assertion_reconciler;
pub mod simple_assertion_reconciler;
pub mod simple_negated_assertion_reconciler;

mod macros;

#[derive(Debug)]
pub struct ReconciliationContext<'a, 's> {
    pub interner: &'a ThreadedInterner,
    pub codebase: &'a CodebaseMetadata,
    pub collector: &'a mut Collector<'s>,
}

impl<'a, 's> ReconciliationContext<'a, 's> {
    pub fn new(
        interner: &'a ThreadedInterner,
        codebase: &'a CodebaseMetadata,
        collector: &'a mut Collector<'s>,
    ) -> Self {
        Self { interner, codebase, collector }
    }
}

pub fn reconcile_keyed_types(
    context: &mut ReconciliationContext<'_, '_>,
    new_types: &BTreeMap<String, Vec<Vec<Assertion>>>,
    mut active_new_types: BTreeMap<String, HashSet<usize>>,
    block_context: &mut BlockContext<'_>,
    changed_var_ids: &mut HashSet<String>,
    referenced_var_ids: &HashSet<String>,
    span: &Span,
    can_report_issues: bool,
    negated: bool,
) {
    if new_types.is_empty() {
        return;
    }

    let inside_loop = block_context.inside_loop;
    let old_new_types = new_types.clone();
    let mut new_types = new_types.clone();

    add_nested_assertions(&mut new_types, &mut active_new_types, block_context);

    for (key, new_type_parts) in &new_types {
        if key.contains("::") && !key.contains('$') && !key.contains('[') {
            continue;
        }

        let mut has_negation = false;
        let mut has_isset = false;
        let mut has_inverted_isset = false;
        let mut has_inverted_key_exists = false;
        let mut has_truthy_or_falsy_or_empty = false;
        let mut has_count_check = false;
        let mut has_empty = false;
        let is_real = old_new_types.get(key).unwrap_or(&Vec::new()).eq(new_type_parts);
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
            }
        }

        let did_type_exist = block_context.locals.contains_key(key);
        let mut has_object_array_access = false;
        let mut possibly_undefined = false;

        let mut result_type = block_context.locals.get(key).map(|t| t.as_ref().clone()).or_else(|| {
            get_value_for_key(
                context,
                key.clone(),
                block_context,
                &new_types,
                has_isset,
                has_inverted_isset,
                has_inverted_key_exists,
                false,
                inside_loop,
                &mut has_object_array_access,
                &mut possibly_undefined,
            )
        });

        let before_adjustment = result_type.clone();
        for (i, new_type_part_parts) in new_type_parts.iter().enumerate() {
            let mut orred_type: Option<TUnion> = None;

            for assertion in new_type_part_parts {
                let result_type_candidate = assertion_reconciler::reconcile(
                    context,
                    assertion,
                    result_type.as_ref(),
                    possibly_undefined,
                    Some(key),
                    inside_loop,
                    Some(span),
                    can_report_issues
                        && if referenced_var_ids.contains(key) && active_new_types.contains_key(key) {
                            active_new_types.get(key).is_some_and(|active_new_type| active_new_type.get(&i).is_some())
                        } else {
                            false
                        },
                    negated,
                );

                orred_type = Some(add_optional_union_type(
                    result_type_candidate,
                    orred_type.as_ref(),
                    context.codebase,
                    context.interner,
                ));
            }

            result_type = orred_type;
        }

        let result_type = result_type.unwrap_or_else(get_never);

        if !did_type_exist && result_type.is_never() {
            continue;
        }

        let type_changed =
            if let Some(before_adjustment) = &before_adjustment { &result_type != before_adjustment } else { true };

        let key_parts = break_up_path_into_parts(key);
        if type_changed {
            changed_var_ids.insert(key.clone());

            if key.ends_with(']') && !has_inverted_isset && !has_inverted_key_exists && !has_empty && !is_equality {
                adjust_array_type(key_parts, block_context, changed_var_ids, &result_type);
            } else if key != "$this" {
                let mut removable_keys = Vec::new();
                for (new_key, _) in block_context.locals.iter() {
                    if new_key.eq(key) {
                        continue;
                    }

                    if is_real && !new_types.contains_key(new_key) && var_has_root(new_key, key) {
                        removable_keys.push(new_key.clone());
                    }
                }

                for new_key in removable_keys {
                    block_context.locals.remove(&new_key);
                }
            }
        } else if !has_negation && !has_truthy_or_falsy_or_empty && !has_isset {
            changed_var_ids.insert(key.clone());
        }

        if !has_object_array_access {
            block_context.locals.insert(key.clone(), Rc::new(result_type));
        }
    }
}

fn adjust_array_type(
    mut key_parts: Vec<String>,
    context: &mut BlockContext<'_>,
    changed_var_ids: &mut HashSet<String>,
    result_type: &TUnion,
) {
    key_parts.pop();
    let Some(array_key) = key_parts.pop() else {
        return;
    };
    key_parts.pop();

    if array_key.starts_with('$') {
        return;
    }

    let mut has_string_offset = false;

    let arraykey_offset = if array_key.starts_with('\'') || array_key.starts_with('\"') {
        has_string_offset = true;
        array_key[1..(array_key.len() - 1)].to_string()
    } else {
        array_key.clone()
    };

    let base_key = key_parts.join("");

    let mut existing_type = if let Some(existing_type) = context.locals.get(&base_key) {
        (**existing_type).clone()
    } else {
        return;
    };

    for base_atomic_type in existing_type.types.iter_mut() {
        match base_atomic_type {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) => {
                let dictkey = if has_string_offset {
                    ArrayKey::String(arraykey_offset.clone())
                } else if let Ok(arraykey_value) = arraykey_offset.parse::<i64>() {
                    ArrayKey::Integer(arraykey_value)
                } else {
                    continue;
                };

                if let Some(known_items) = known_items {
                    known_items.insert(dictkey, (false, result_type.clone()));
                } else {
                    *known_items = Some(BTreeMap::from([(dictkey, (false, result_type.clone()))]));
                }
            }
            TAtomic::Array(TArray::List(TList { known_elements, .. })) => {
                if let Ok(arraykey_offset) = arraykey_offset.parse::<usize>() {
                    if let Some(known_elements) = known_elements {
                        known_elements.insert(arraykey_offset, (false, result_type.clone()));
                    } else {
                        *known_elements = Some(BTreeMap::from([(arraykey_offset, (false, result_type.clone()))]));
                    }
                }
            }
            _ => {
                continue;
            }
        }

        changed_var_ids.insert(format!("{}[{}]", base_key, array_key.clone()));

        if let Some(last_part) = key_parts.last()
            && last_part == "]"
        {
            adjust_array_type(key_parts.clone(), context, changed_var_ids, &wrap_atomic(base_atomic_type.clone()));
        }
    }

    context.locals.insert(base_key, Rc::new(existing_type));
}

fn refine_array_key(key_type: &TUnion) -> TUnion {
    fn refine_array_key_inner(key_type: &TUnion) -> Option<TUnion> {
        let mut refined = false;
        let mut types = vec![];

        for cat in &key_type.types {
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

        if refined { Some(TUnion::new(types)) } else { None }
    }

    refine_array_key_inner(key_type).unwrap_or_else(|| key_type.clone())
}

static INTEGER_REGEX: LazyLock<Regex> = LazyLock::new(|| unsafe {
    // SAFETY: `unwrap_unchecked` is safe here because the regex is valid and will not panic.
    Regex::new(r"^[0-9]+$").unwrap_unchecked()
});

fn add_nested_assertions(
    new_types: &mut BTreeMap<String, Vec<Vec<Assertion>>>,
    active_new_types: &mut BTreeMap<String, HashSet<usize>>,
    context: &BlockContext<'_>,
) {
    let mut keys_to_remove = vec![];

    'outer: for (nk, new_type) in new_types.clone() {
        if (nk.contains('[') || nk.contains("->"))
            && (new_type[0][0] == Assertion::IsEqualIsset || new_type[0][0] == Assertion::IsIsset)
        {
            let mut key_parts = break_up_path_into_parts(&nk);
            key_parts.reverse();

            let mut nesting = 0;
            let mut base_key;

            unsafe {
                // SAFETY: `pop` will always return a value because we checked that the key contains either `[` or `->`.
                base_key = key_parts.pop().unwrap_unchecked();

                if !&base_key.starts_with('$') && key_parts.len() > 2 && key_parts.last().unwrap_unchecked() == "::$" {
                    base_key += key_parts.pop().unwrap_unchecked().as_str();
                    base_key += key_parts.pop().unwrap_unchecked().as_str();
                }
            };

            let base_key_set = if let Some(base_key_type) = context.locals.get(&base_key) {
                !base_key_type.is_nullable()
            } else {
                false
            };

            if !base_key_set {
                new_types.insert(
                    base_key.clone(),
                    if let Some(mut existing_entry) = new_types.get(&base_key).cloned() {
                        existing_entry.push(vec![Assertion::IsEqualIsset]);
                        existing_entry
                    } else {
                        vec![vec![Assertion::IsEqualIsset]]
                    },
                );
            }

            while let Some(divider) = key_parts.pop() {
                if divider == "[" {
                    let array_key = unsafe {
                        // SAFETY: we know that after `[` there is always an array key, so `pop` will not panic.
                        key_parts.pop().unwrap_unchecked()
                    };

                    key_parts.pop();

                    let new_base_key = base_key.clone() + "[" + array_key.as_str() + "]";

                    let entry = new_types.entry(base_key.clone()).or_default();

                    let new_key = if array_key.starts_with('\'') {
                        Some(ArrayKey::String(array_key[1..(array_key.len() - 1)].to_string()))
                    } else if array_key.starts_with('$') {
                        None
                    } else if let Ok(arraykey_value) = array_key.parse::<i64>() {
                        Some(ArrayKey::Integer(arraykey_value))
                    } else {
                        continue 'outer;
                    };

                    if let Some(new_key) = new_key {
                        entry.push(vec![Assertion::HasNonnullEntryForKey(new_key)]);

                        if key_parts.is_empty() {
                            keys_to_remove.push(nk.clone());

                            if nesting == 0 && base_key_set && active_new_types.remove(&nk).is_some() {
                                active_new_types.entry(base_key.clone()).or_default().insert(entry.len() - 1);
                            }

                            break 'outer;
                        }
                    } else {
                        entry.push(vec![if array_key.contains('\'') {
                            Assertion::HasStringArrayAccess
                        } else {
                            Assertion::HasIntOrStringArrayAccess
                        }]);
                    }

                    base_key = new_base_key;
                    nesting += 1;
                    continue;
                }

                if divider == "->" {
                    let property_name = unsafe {
                        // SAFETY: we know that after `->` there is always a property name, so `pop` will not panic.
                        key_parts.pop().unwrap_unchecked()
                    };

                    let new_base_key = base_key.clone() + "->" + property_name.as_str();

                    if !new_types.contains_key(&base_key) {
                        new_types.insert(base_key.clone(), vec![vec![Assertion::IsIsset]]);
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
    }

    new_types.retain(|k, _| !keys_to_remove.contains(k));
}

pub fn break_up_path_into_parts(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec!["".to_string()];
    }

    let mut parts: Vec<String> = Vec::with_capacity(path.len() / 4 + 1);
    parts.push(String::with_capacity(16));

    let mut chars = path.chars().peekable();

    let mut string_char: Option<char> = None;
    let mut escape_char = false;
    let mut brackets: i32 = 0;

    while let Some(c) = chars.next() {
        if let Some(quote) = string_char {
            // SAFETY: the `parts` vector will always contain at least 1 string.
            unsafe {
                parts.last_mut().unwrap_unchecked().push(c);
            }

            if c == quote && !escape_char {
                string_char = None;
            }

            escape_char = c == '\\' && !escape_char;
        } else {
            let mut token_found: Option<&'static str> = None;
            match c {
                '[' => {
                    brackets += 1;
                    token_found = Some("[");
                }
                ']' => {
                    brackets -= 1;
                    token_found = Some("]");
                }
                '\'' | '"' => {
                    string_char = Some(c);
                    unsafe {
                        // SAFETY: the `parts` vector will always contain at least 1 string.
                        parts.last_mut().unwrap_unchecked().push(c);
                    }
                }
                ':' if brackets == 0 && chars.peek() == Some(&':') => {
                    let mut lookahead = chars.clone();
                    lookahead.next();
                    if lookahead.peek() == Some(&'$') {
                        chars.next();
                        chars.next();
                        token_found = Some("::$");
                    } else {
                        unsafe {
                            // SAFETY: the `parts` vector will always contain at least 1 string.
                            parts.last_mut().unwrap_unchecked().push(c);
                        }
                    }
                }
                '-' if brackets == 0 && chars.peek() == Some(&'>') => {
                    chars.next();
                    token_found = Some("->");
                }
                _ => {
                    unsafe {
                        // SAFETY: the `parts` vector will always contain at least 1 string.
                        parts.last_mut().unwrap_unchecked().push(c);
                    }
                }
            }

            if let Some(token) = token_found {
                if let Some(last_part) = parts.last_mut()
                    && last_part.is_empty()
                {
                    *last_part = token.to_string();
                } else {
                    parts.push(token.to_string());
                }

                parts.push(String::new());
            }
        }
    }

    // If the path does not end with a token, the last added empty string will be unused.
    // We remove it before returning.
    if let Some(last_part) = parts.last()
        && last_part.is_empty()
    {
        parts.pop();
    }

    parts
}

fn get_value_for_key(
    context: &mut ReconciliationContext<'_, '_>,
    key: String,
    block_context: &mut BlockContext<'_>,
    new_assertions: &BTreeMap<String, Vec<Vec<Assertion>>>,
    has_isset: bool,
    has_inverted_isset: bool,
    has_inverted_key_exists: bool,
    has_empty: bool,
    inside_loop: bool,
    has_object_array_access: &mut bool,
    possibly_undefined: &mut bool,
) -> Option<TUnion> {
    let mut key_parts = break_up_path_into_parts(&key);
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

    let mut base_key;

    unsafe {
        // SAFETY: `pop` will always return a value because we checked that the key has more than one part.
        base_key = key_parts.pop().unwrap_unchecked();

        if !base_key.starts_with('$')
            && key_parts.len() > 2
            && key_parts.last().is_some_and(|part| part.starts_with("::$"))
        {
            // SAFETY: `pop` will always return a value because we checked that the key has more than two parts.
            base_key += key_parts.pop().unwrap_unchecked().as_str();
            base_key += key_parts.pop().unwrap_unchecked().as_str();
        }
    };

    if !block_context.locals.contains_key(&base_key) {
        if base_key.contains("::") {
            let base_key_parts = &base_key.split("::").collect::<Vec<&str>>();
            let fq_class_name = base_key_parts[0].to_string();
            let const_name = base_key_parts[1].to_string();

            let fq_class_name = context.interner.intern(fq_class_name.as_str());
            if !class_like_exists(context.codebase, context.interner, &fq_class_name) {
                return None;
            }

            let class_constant = if let Some(const_name) = context.interner.get(&const_name) {
                get_class_constant_type(context.codebase, context.interner, &fq_class_name, &const_name)
            } else {
                None
            };

            if let Some(class_constant) = class_constant {
                let class_constant = Rc::new(match class_constant {
                    Cow::Borrowed(t) => t.clone(),
                    Cow::Owned(t) => t,
                });

                block_context.locals.insert(base_key.clone(), class_constant);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    while let Some(divider) = key_parts.pop() {
        let base_key_type = block_context.locals.get(&base_key)?;

        if divider == "[" {
            let array_key = key_parts.pop()?;

            key_parts.pop();

            let array_key_offset = if INTEGER_REGEX.is_match(&array_key)
                && let Ok(integer) = array_key.parse::<usize>()
            {
                Some(integer)
            } else {
                None
            };

            let array_key_type = if let Some(array_key_offset) = array_key_offset {
                ArrayKey::Integer(array_key_offset as i64)
            } else {
                ArrayKey::String(array_key.replace('\'', "").to_string())
            };

            let new_base_key = base_key.clone() + "[" + array_key.as_str() + "]";

            if !block_context.locals.contains_key(&new_base_key) {
                let mut new_base_type: Option<Rc<TUnion>> = None;
                let mut atomic_types = base_key_type.types.clone();

                atomic_types.reverse();
                while let Some(existing_key_type_part) = atomic_types.pop() {
                    if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = existing_key_type_part {
                        atomic_types.extend(constraint.types.clone());
                        continue;
                    }

                    let mut new_base_type_candidate;

                    if let TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) = &existing_key_type_part {
                        if has_empty {
                            return None;
                        }

                        let known_item = if !array_key.starts_with('$')
                            && let Some(known_items) = known_items
                        {
                            known_items.get(&array_key_type)
                        } else {
                            None
                        };

                        if let Some(known_item) = known_item {
                            let known_item = known_item.clone();

                            new_base_type_candidate = known_item.1.clone();

                            if known_item.0 {
                                *possibly_undefined = true;
                            }
                        } else {
                            if has_empty {
                                return None;
                            }

                            new_base_type_candidate = get_iterable_value_parameter(
                                &existing_key_type_part,
                                context.codebase,
                                context.interner,
                            )?;

                            if new_base_type_candidate.is_mixed()
                                && !has_isset
                                && !has_inverted_isset
                                && !has_inverted_key_exists
                            {
                                return Some(new_base_type_candidate);
                            }

                            if (has_isset || has_inverted_isset || has_inverted_key_exists)
                                && new_assertions.contains_key(&new_base_key)
                            {
                                if has_inverted_isset && new_base_key.eq(&key) {
                                    new_base_type_candidate = add_union_type(
                                        new_base_type_candidate,
                                        &get_null(),
                                        context.codebase,
                                        context.interner,
                                        false,
                                    );
                                }

                                *possibly_undefined = true;
                            }
                        }
                    } else if let TAtomic::Array(TArray::List(TList { known_elements, .. })) = &existing_key_type_part {
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
                                *possibly_undefined = true;
                            }
                        } else {
                            new_base_type_candidate = get_iterable_value_parameter(
                                &existing_key_type_part,
                                context.codebase,
                                context.interner,
                            )?;

                            if (has_isset || has_inverted_isset || has_inverted_key_exists)
                                && new_assertions.contains_key(&new_base_key)
                            {
                                if has_inverted_isset && new_base_key.eq(&key) {
                                    new_base_type_candidate = add_union_type(
                                        new_base_type_candidate,
                                        &get_null(),
                                        context.codebase,
                                        context.interner,
                                        false,
                                    );
                                }

                                *possibly_undefined = true;
                            }
                        }
                    } else if matches!(existing_key_type_part, TAtomic::Scalar(TScalar::String(_))) {
                        return Some(get_string());
                    } else if existing_key_type_part.is_never() || existing_key_type_part.is_mixed_isset_from_loop() {
                        return Some(get_mixed_maybe_from_loop(inside_loop));
                    } else if let TAtomic::Object(TObject::Named(_named_object)) = &existing_key_type_part {
                        if has_isset || has_inverted_isset || has_inverted_key_exists {
                            *has_object_array_access = true;
                            block_context.locals.remove(&new_base_key);

                            return None;
                        }

                        return Some(get_mixed());
                    } else {
                        return Some(get_mixed());
                    }

                    let resulting_type = Rc::new(if let Some(new_base_type) = &new_base_type {
                        add_union_type(
                            new_base_type_candidate,
                            new_base_type,
                            context.codebase,
                            context.interner,
                            false,
                        )
                    } else {
                        new_base_type_candidate.clone()
                    });

                    new_base_type = Some(resulting_type.clone());
                    block_context.locals.insert(new_base_key.clone(), resulting_type);
                }
            }

            base_key = new_base_key;
        } else if divider == "->" || divider == "::$" {
            let property_name = key_parts.pop()?;
            let new_base_key = base_key.clone() + "->" + property_name.as_str();

            if !block_context.locals.contains_key(&new_base_key) {
                let mut new_base_type: Option<Rc<TUnion>> = None;
                let mut atomic_types = base_key_type.types.clone();

                while let Some(existing_key_type_part) = atomic_types.pop() {
                    if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = existing_key_type_part {
                        atomic_types.extend(constraint.types.clone());
                        continue;
                    }

                    let class_property_type: TUnion;

                    if let TAtomic::Null = existing_key_type_part {
                        class_property_type = get_null();
                        // TODO(azjezz): maybe we should exclude mixed from isset in loop?
                    } else if let TAtomic::Mixed(_) | TAtomic::GenericParameter(_) | TAtomic::Object(TObject::Any) =
                        existing_key_type_part
                    {
                        class_property_type = get_mixed();
                    } else if let TAtomic::Object(TObject::Named(named_object)) = existing_key_type_part {
                        let fq_class_name = named_object.get_name_ref();
                        if context.interner.lookup(fq_class_name).eq_ignore_ascii_case("stdClass")
                            || !class_or_interface_exists(context.codebase, context.interner, fq_class_name)
                        {
                            class_property_type = get_mixed();
                        } else {
                            class_property_type = get_property_type(context, fq_class_name, &property_name)?;
                        }
                    } else {
                        class_property_type = get_mixed();
                    }

                    let resulting_type = Rc::new(add_optional_union_type(
                        class_property_type,
                        new_base_type.as_deref(),
                        context.codebase,
                        context.interner,
                    ));

                    new_base_type = Some(resulting_type.clone());
                    block_context.locals.insert(new_base_key.clone(), resulting_type);
                }
            }

            base_key = new_base_key;
        } else {
            return None;
        }
    }

    block_context.locals.get(&base_key).map(|t| (**t).clone())
}

fn get_property_type(
    context: &ReconciliationContext<'_, '_>,
    classlike_name: &StringIdentifier,
    property_name_str: &str,
) -> Option<TUnion> {
    // Add `$` prefix
    let prefixed_property_name = "$".to_owned() + property_name_str;
    let property_name = context.interner.intern(&prefixed_property_name);

    let declaring_property_class =
        get_declaring_class_for_property(context.codebase, context.interner, classlike_name, &property_name)?;
    let property_metadata = get_property(context.codebase, context.interner, classlike_name, &property_name)?;
    let property_type = property_metadata.type_metadata.as_ref().map(|metadata| metadata.type_union.clone());

    let property_type = if let Some(mut property_type) = property_type {
        expander::expand_union(
            context.codebase,
            context.interner,
            &mut property_type,
            &TypeExpansionOptions {
                self_class: Some(&declaring_property_class),
                static_class_type: StaticClassType::Name(declaring_property_class),
                ..Default::default()
            },
        );

        property_type
    } else {
        get_mixed()
    };

    Some(property_type)
}

pub(crate) fn trigger_issue_for_impossible(
    context: &mut ReconciliationContext<'_, '_>,
    old_var_type_string: &String,
    key: &String,
    assertion: &Assertion,
    redundant: bool,
    negated: bool,
    span: &Span,
) {
    let mut assertion_string = assertion.as_string(Some(context.interner));
    let mut not_operator = assertion_string.starts_with('!');

    if not_operator {
        assertion_string = assertion_string[1..].to_string();
    }

    let mut redundant = redundant;
    if negated {
        not_operator = !not_operator;
        redundant = !redundant;
    }

    if redundant {
        if not_operator {
            if assertion_string == "falsy" {
                not_operator = false;
                assertion_string = "truthy".to_string();
            } else if assertion_string == "truthy" {
                not_operator = false;
                assertion_string = "falsy".to_string();
            }
        }

        if not_operator {
            report_impossible_issue(context, assertion, &assertion_string, key, span, old_var_type_string)
        } else {
            report_redundant_issue(context, assertion, &assertion_string, key, span, old_var_type_string)
        }
    } else if not_operator {
        report_redundant_issue(context, assertion, &assertion_string, key, span, old_var_type_string)
    } else {
        report_impossible_issue(context, assertion, &assertion_string, key, span, old_var_type_string)
    }
}

fn report_impossible_issue(
    context: &mut ReconciliationContext<'_, '_>,
    assertion: &Assertion,
    assertion_string: &String,
    key: &String,
    span: &Span,
    old_var_type_string: &String,
) {
    let subject_desc = if old_var_type_string.is_empty() || old_var_type_string.len() > 50 {
        format!("`{key}`")
    } else {
        format!("`{key}` (type `{old_var_type_string}`)")
    };

    let (issue_kind, main_message_verb, specific_note, specific_help) = match assertion {
        Assertion::Truthy => (
            Code::IMPOSSIBLE_CONDITION,
            "will always evaluate to false".to_owned(),
            format!("Variable {subject_desc} is always falsy and can never satisfy a truthiness check."),
            "Review the logic or type of the variable; this condition will never pass.".to_string(),
        ),
        Assertion::Falsy => (
            Code::IMPOSSIBLE_CONDITION,
            "will always evaluate to false".to_owned(),
            format!("Variable {subject_desc} is always truthy, so asserting it is falsy will always be false."),
            "Review the logic or type of the variable; this condition will never pass.".to_string(),
        ),
        Assertion::IsType(TAtomic::Null) => (
            Code::IMPOSSIBLE_NULL_TYPE_COMPARISON,
            "can never be `null`".to_owned(),
            format!("Variable {subject_desc} does not include `null`."),
            format!(
                "The condition checking if `{key}` is `null` will always be false. Remove or refactor the condition.",
            ),
        ),
        Assertion::IsNotType(TAtomic::Null) => (
            Code::IMPOSSIBLE_NULL_TYPE_COMPARISON,
            "will always be `null`".to_owned(),
            format!("Variable {subject_desc} is already known to be `null`, so asserting it's not `null` is impossible."),
            format!("The condition checking if `{key}` is not `null` will always be false. Review the variable's state or condition."),
        ),
        Assertion::HasArrayKey(array_key_assertion) => (
            Code::IMPOSSIBLE_KEY_CHECK,
            format!("can never have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to not contain the key `{array_key_assertion}`. This check will always be false."),
            "Ensure the array structure and key are correct, or remove this condition.".to_owned(),
        ),
        Assertion::DoesNotHaveArrayKey(array_key_assertion) => (
            Code::IMPOSSIBLE_KEY_CHECK,
            format!("will always have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to always contain the key `{array_key_assertion}`. Asserting it doesn't have this key will always be false."),
            "Review the logic; this negative key check will always fail.".to_owned(),
        ),
        Assertion::HasNonnullEntryForKey(dict_key_name) => (
            Code::IMPOSSIBLE_NONNULL_ENTRY_CHECK,
            format!("can never have a non-null entry for key `{dict_key_name}`"),
            format!("Variable {subject_desc} is known to either not have the key `{dict_key_name}` or its value is always `null`. This check for a non-null entry will always be false."),
            "Verify the array/object structure or remove this `!empty()` style check.".to_owned(),
        ),
        _ => (
            Code::IMPOSSIBLE_TYPE_COMPARISON,
            format!("can never be `{assertion_string}`"),
            format!("The type of variable {subject_desc} is incompatible with the assertion that it is `{assertion_string}`."),
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

fn report_redundant_issue(
    context: &mut ReconciliationContext<'_, '_>,
    assertion: &Assertion,
    assertion_string: &String,
    key: &String,
    span: &Span,
    old_var_type_string: &String,
) {
    let subject_desc = if old_var_type_string.is_empty() || old_var_type_string.len() > 50 {
        format!("`{key}`")
    } else {
        format!("`{key}` (type `{old_var_type_string}`)")
    };

    let (issue_kind, main_message_verb, specific_note, specific_help) = match assertion {
        Assertion::IsIsset | Assertion::IsEqualIsset => (
            Code::REDUNDANT_ISSET_CHECK,
            "is always considered set (not null)".to_owned(),
            format!("Variable {subject_desc} is already known to be non-null, making the `isset()` check redundant."),
            "Remove the redundant `isset()` check.".to_owned()
        ),
        Assertion::Truthy => (
            Code::REDUNDANT_CONDITION,
            "will always evaluate to true".to_owned(),
            format!("Variable {subject_desc} is always truthy. This condition is redundant and the code block will always execute if reached."),
            "Simplify or remove the redundant condition if the guarded code should always run.".to_owned()
        ),
        Assertion::Falsy => (
            Code::REDUNDANT_CONDITION,
            "will always evaluate to true".to_owned(),
            format!("Variable {subject_desc} is always falsy, so asserting it's falsy is always true and redundant."),
            "Simplify or remove the redundant condition if the guarded code should always run.".to_owned()
        ),
        Assertion::HasArrayKey(array_key_assertion) => (
            Code::REDUNDANT_KEY_CHECK,
            format!("will always have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to always contain the key `{array_key_assertion}`. This check is redundant."),
            "Remove the redundant `array_key_exists()` or key check.".to_owned()
        ),
        Assertion::DoesNotHaveArrayKey(array_key_assertion) => (
            Code::REDUNDANT_KEY_CHECK,
            format!("will never have the key `{array_key_assertion}`"),
            format!("Variable {subject_desc} is known to never contain the key `{array_key_assertion}`. This negative check is redundant."),
            "Remove the redundant negative key check.".to_owned()
        ),
        Assertion::HasNonnullEntryForKey(dict_key_name) => (
            Code::REDUNDANT_NONNULL_ENTRY_CHECK,
            format!("will always have a non-null entry for key `{dict_key_name}`"),
            format!("Variable {subject_desc} is known to always have a non-null value for key `{dict_key_name}`. This `!empty()` style check is redundant."),
            "Remove the redundant non-null entry check.".to_owned()
        ),
        Assertion::IsType(TAtomic::Mixed(mixed)) if mixed.is_non_null() => (
            Code::REDUNDANT_NONNULL_TYPE_COMPARISON,
            "is already known to be non-null".to_owned(),
            format!("Variable {subject_desc} is already non-null. Checking against `mixed (not null)` is redundant."),
            "Remove the redundant non-null check.".to_owned()
        ),
        Assertion::IsNotType(TAtomic::Mixed(mixed)) if mixed.is_non_null() => (
            Code::REDUNDANT_TYPE_COMPARISON,
            "comparison with `mixed (not null)` is redundant".to_owned(),
            format!("The check against `mixed (not null)` for variable {subject_desc} might be overly broad or redundant depending on context."),
            "Verify if a more specific type check is needed.".to_owned()
        ),
        _ => (
            Code::REDUNDANT_TYPE_COMPARISON,
            format!("is already known to be `{assertion_string}`"),
            format!("The type of variable {subject_desc} already satisfies the condition that it is `{assertion_string}`. This check is redundant."),
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
    let parameter = if generic_parameter.constraint.is_mixed() {
        generic_parameter.clone()
    } else {
        generic_parameter.with_constraint(f(&generic_parameter.constraint))
    };

    if parameter.constraint.is_never() { None } else { Some(TAtomic::GenericParameter(parameter)) }
}

fn map_generic_constraint_or_else<F, D>(generic_parameter: &TGenericParameter, d: D, f: F) -> Option<TAtomic>
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
        let path = "$service_name->prop[0]->foo::$prop";
        let expected: Vec<&str> = vec!["$service_name", "->", "prop", "[", "0", "]", "->", "foo", "::$", "prop"];
        let result = break_up_path_into_parts(path);
        assert_eq!(result, expected);
    }
}
