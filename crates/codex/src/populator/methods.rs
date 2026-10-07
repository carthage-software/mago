use foldhash::HashMap;
use indexmap::IndexMap;

use mago_word::Word;
use mago_word::word;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;

/// Inherits method declarations and appearances from a parent class-like.
/// Updates `declaring_method_ids`, `appearing_method_ids`, etc.
pub fn inherit_methods_from_parent(
    metadata: &mut ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) {
    let class_like_name = metadata.name;
    let parent_is_trait = parent_metadata.kind.is_trait();
    let constructor_name = word("__construct");
    let clone_name = word("__clone");

    let reverse_alias_map: Option<HashMap<Word, Vec<Word>>> = if parent_is_trait && !metadata.trait_alias_map.is_empty()
    {
        let mut map: HashMap<Word, Vec<Word>> = HashMap::default();
        for (original, alias) in metadata.get_trait_alias_map() {
            map.entry(*original).or_default().push(*alias);
        }
        Some(map)
    } else {
        None
    };

    for (method_name_lc, appearing_method_id) in &parent_metadata.appearing_method_ids {
        let mut process_name = |aliased_method_name: Word| {
            metadata.appearing_method_ids.entry(aliased_method_name).or_insert_with(|| {
                if parent_is_trait {
                    MethodIdentifier::new(class_like_name, aliased_method_name)
                } else {
                    *appearing_method_id
                }
            });
        };

        process_name(*method_name_lc);

        if let Some(reverse_map) = reverse_alias_map.as_ref()
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            for alias in aliases.iter().copied() {
                process_name(alias);
            }
        }
    }

    for (method_name_lc, declaring_method_id) in &parent_metadata.inheritable_method_ids {
        if *method_name_lc != constructor_name || parent_metadata.flags.has_consistent_constructor() {
            let add_parent = !parent_is_trait
                || codebase
                    .function_likes
                    .get(&(declaring_method_id.get_class_name(), *method_name_lc))
                    .and_then(|meta| meta.method_metadata.as_ref())
                    .is_some_and(|method| method.is_abstract);
            let parent_overridden = parent_metadata.overridden_method_ids.get(method_name_lc);
            let existing_overridden = if add_parent {
                let overridden = metadata.overridden_method_ids.entry(*method_name_lc).or_insert_with(|| {
                    let mut parents = IndexMap::default();
                    parents.reserve(1 + parent_overridden.map_or(0, IndexMap::len));
                    parents
                });
                overridden.insert(declaring_method_id.get_class_name(), *declaring_method_id);
                Some(overridden)
            } else {
                metadata.overridden_method_ids.get_mut(method_name_lc)
            };

            if let (Some(existing_overridden), Some(parent_overridden)) = (existing_overridden, parent_overridden) {
                existing_overridden.extend(parent_overridden.iter().map(|(k, v)| (*k, *v)));
            }
        }

        let process_name = |aliased_method_name: Word, metadata: &mut ClassLikeMetadata| {
            if let Some(implementing_method_id) = metadata.declaring_method_ids.get(&aliased_method_name) {
                let implementing_class = implementing_method_id.get_class_name();
                let implementing_method_name = implementing_method_id.get_method_name();

                let is_existing_pseudo_from_trait = !parent_is_trait
                    && codebase.class_likes.get(&implementing_class).is_some_and(|c| c.kind.is_trait())
                    && codebase
                        .function_likes
                        .get(&(implementing_class, implementing_method_name))
                        .is_some_and(|m| m.flags.is_magic_method());

                if !is_existing_pseudo_from_trait
                    && (!codebase
                        .method_is_abstract(implementing_class.as_bytes(), implementing_method_name.as_bytes())
                        || implementing_class == class_like_name)
                {
                    return;
                }
            }

            metadata.declaring_method_ids.insert(aliased_method_name, *declaring_method_id);

            let is_ctor_or_clone = aliased_method_name == constructor_name || aliased_method_name == clone_name;
            let is_inheritable = !parent_is_trait
                || metadata.kind.is_trait()
                || is_ctor_or_clone
                || metadata
                    .trait_visibility_map
                    .get(&aliased_method_name)
                    .copied()
                    .or_else(|| {
                        codebase.get_method_visibility(parent_metadata.name.as_bytes(), aliased_method_name.as_bytes())
                    })
                    .is_none_or(|visibility| !visibility.is_private());

            if is_inheritable {
                metadata.inheritable_method_ids.insert(aliased_method_name, *declaring_method_id);
            }
        };

        process_name(*method_name_lc, metadata);

        if let Some(reverse_map) = reverse_alias_map.as_ref()
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            for alias in aliases.iter().copied() {
                process_name(alias, metadata);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mago_span::Span;

    use super::*;
    use crate::metadata::flags::MetadataFlags;
    use crate::metadata::function_like::FunctionLikeKind;
    use crate::metadata::function_like::FunctionLikeMetadata;
    use crate::metadata::function_like::MethodMetadata;
    use crate::symbol::SymbolKind;
    use crate::visibility::Visibility;

    fn class(name: &str) -> ClassLikeMetadata {
        let name = word(name);
        ClassLikeMetadata::new(name, name, Span::dummy(0, 0), None, MetadataFlags::empty())
    }

    fn add_method(metadata: &mut ClassLikeMetadata, name: Word) -> MethodIdentifier {
        let id = MethodIdentifier::new(metadata.name, name);
        metadata.appearing_method_ids.insert(name, id);
        metadata.inheritable_method_ids.insert(name, id);
        id
    }

    fn overridden(metadata: &ClassLikeMetadata, method: Word) -> Option<Vec<(Word, MethodIdentifier)>> {
        metadata.overridden_method_ids.get(&method).map(|parents| parents.iter().map(|(k, v)| (*k, *v)).collect())
    }

    #[test]
    fn inherited_overrides_keep_entry_order_and_parent_update_precedence() {
        let method = word("run");
        let mut parent = class("Parent");
        let parent_id = add_method(&mut parent, method);
        let ancestor = MethodIdentifier::new(word("Ancestor"), method);
        let replacement = MethodIdentifier::new(parent.name, word("original"));
        parent.add_overridden_method_parent(method, ancestor);
        parent.add_overridden_method_parent(method, replacement);

        let mut child = class("Child");
        inherit_methods_from_parent(&mut child, &parent, &CodebaseMetadata::new());
        assert_eq!(
            overridden(&child, method),
            Some(vec![(parent.name, replacement), (ancestor.get_class_name(), ancestor)])
        );
        assert_eq!(child.declaring_method_ids.get(&method), Some(&parent_id));

        let mut child = class("OtherChild");
        let old_ancestor = MethodIdentifier::new(ancestor.get_class_name(), word("old"));
        let existing = MethodIdentifier::new(word("Existing"), method);
        child.add_overridden_method_parent(method, old_ancestor);
        child.add_overridden_method_parent(method, existing);
        inherit_methods_from_parent(&mut child, &parent, &CodebaseMetadata::new());
        assert_eq!(
            overridden(&child, method),
            Some(vec![
                (ancestor.get_class_name(), ancestor),
                (existing.get_class_name(), existing),
                (parent.name, replacement)
            ])
        );
    }

    #[test]
    fn trait_overrides_create_maps_only_for_abstract_methods() {
        let method = word("run");
        let mut parent = class("ParentTrait");
        parent.kind = SymbolKind::Trait;
        let parent_id = add_method(&mut parent, method);
        let ancestor = MethodIdentifier::new(word("Ancestor"), method);
        parent.add_overridden_method_parent(method, ancestor);

        for is_abstract in [false, true] {
            for has_existing in [false, true] {
                let mut child = class("Child");
                let existing = MethodIdentifier::new(word("Existing"), method);
                if has_existing {
                    child.add_overridden_method_parent(method, existing);
                }
                let mut codebase = CodebaseMetadata::new();
                let mut function = FunctionLikeMetadata::new(
                    FunctionLikeKind::Method,
                    method,
                    method,
                    Span::dummy(0, 0),
                    MetadataFlags::empty(),
                );
                function.method_metadata = Some(MethodMetadata { is_abstract, ..MethodMetadata::default() });
                codebase.function_likes.insert((parent.name, method), function);
                inherit_methods_from_parent(&mut child, &parent, &codebase);

                let mut expected = Vec::new();
                if has_existing {
                    expected.push((existing.get_class_name(), existing));
                }
                if is_abstract {
                    expected.push((parent.name, parent_id));
                }
                if is_abstract || has_existing {
                    expected.push((ancestor.get_class_name(), ancestor));
                    assert_eq!(overridden(&child, method), Some(expected));
                } else {
                    assert_eq!(overridden(&child, method), None);
                }
            }
        }
    }

    #[test]
    fn constructor_overrides_require_a_consistent_parent() {
        let method = word("__construct");
        for consistent in [false, true] {
            let mut parent = class("Parent");
            let parent_id = add_method(&mut parent, method);
            if consistent {
                parent.flags.insert(MetadataFlags::CONSISTENT_CONSTRUCTOR);
            }
            let mut child = class("Child");
            inherit_methods_from_parent(&mut child, &parent, &CodebaseMetadata::new());
            assert_eq!(overridden(&child, method), consistent.then(|| vec![(parent.name, parent_id)]));
        }
    }

    #[test]
    fn trait_aliases_keep_existing_appearances_and_private_magic_methods() {
        let method = word("run");
        let alias = word("alias");
        let constructor = word("__construct");
        let clone_method = word("__clone");
        let mut parent = class("ParentTrait");
        parent.kind = SymbolKind::Trait;
        let method_id = add_method(&mut parent, method);
        add_method(&mut parent, constructor);
        add_method(&mut parent, clone_method);
        let mut child = class("Child");
        let own_alias = MethodIdentifier::new(child.name, alias);
        child.appearing_method_ids.insert(alias, own_alias);
        child.trait_alias_map.insert(method, alias);
        for name in [method, alias, constructor, clone_method] {
            child.trait_visibility_map.insert(name, Visibility::Private);
        }

        inherit_methods_from_parent(&mut child, &parent, &CodebaseMetadata::new());
        assert_eq!(child.appearing_method_ids.get(&alias), Some(&own_alias));
        assert_eq!(child.appearing_method_ids.get(&method), Some(&MethodIdentifier::new(child.name, method)));
        assert_eq!(child.declaring_method_ids.get(&alias), Some(&method_id));
        assert!(!child.inheritable_method_ids.contains_key(&method));
        assert!(!child.inheritable_method_ids.contains_key(&alias));
        assert!(child.inheritable_method_ids.contains_key(&constructor));
        assert!(child.inheritable_method_ids.contains_key(&clone_method));
    }
}
