use ahash::HashMap;
use mago_atom::Atom;
use mago_atom::atom;

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

    let reverse_alias_map: Option<HashMap<Atom, Vec<Atom>>> = if parent_is_trait && !metadata.trait_alias_map.is_empty()
    {
        let mut map: HashMap<Atom, Vec<Atom>> = HashMap::default();
        for (original, alias) in metadata.get_trait_alias_map() {
            map.entry(*original).or_default().push(*alias);
        }
        Some(map)
    } else {
        None
    };

    for (method_name_lc, appearing_method_id) in &parent_metadata.appearing_method_ids {
        let mut aliased_method_names = vec![*method_name_lc];

        if let Some(ref reverse_map) = reverse_alias_map
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            aliased_method_names.extend(aliases.iter().copied());
        }

        for aliased_method_name in aliased_method_names {
            if metadata.has_appearing_method(&aliased_method_name) {
                continue;
            }

            let implemented_method_id = MethodIdentifier::new(class_like_name, aliased_method_name);

            let final_appearing_id = if parent_is_trait { implemented_method_id } else { *appearing_method_id };

            metadata.appearing_method_ids.insert(aliased_method_name, final_appearing_id);
        }
    }

    for (method_name_lc, declaring_method_id) in &parent_metadata.inheritable_method_ids {
        if !method_name_lc.eq(&atom("__construct")) || parent_metadata.flags.has_consistent_constructor() {
            if parent_is_trait {
                let declaring_class = declaring_method_id.get_class_name();

                if codebase
                    .function_likes
                    .get(&(*declaring_class, *method_name_lc))
                    .and_then(|meta| meta.method_metadata.as_ref())
                    .is_some_and(|method| method.is_abstract)
                {
                    metadata.add_overridden_method_parent(*method_name_lc, *declaring_method_id);
                }
            } else {
                metadata.add_overridden_method_parent(*method_name_lc, *declaring_method_id);
            }

            if let Some(existing_overridden) = metadata.overridden_method_ids.get_mut(method_name_lc)
                && let Some(parent_overridden_map) = parent_metadata.overridden_method_ids.get(method_name_lc)
            {
                existing_overridden.extend(parent_overridden_map.iter().map(|(k, v)| (*k, *v)));
            }
        }

        let mut aliased_method_names = vec![*method_name_lc];

        if let Some(ref reverse_map) = reverse_alias_map
            && let Some(aliases) = reverse_map.get(method_name_lc)
        {
            aliased_method_names.extend(aliases.iter().copied());
        }

        for aliased_method_name in aliased_method_names {
            if let Some(implementing_method_id) = metadata.declaring_method_ids.get(&aliased_method_name) {
                let implementing_class = implementing_method_id.get_class_name();
                let implementing_method_name = implementing_method_id.get_method_name();

                // Don't overwrite if:
                // 1. The child has a concrete (non-abstract) method, OR
                // 2. The child declared its own version (even if abstract) - this preserves
                //    interface method overrides where a child interface narrows the return type
                if !codebase.method_is_abstract(implementing_class, implementing_method_name)
                    || *implementing_class == class_like_name
                {
                    continue;
                }
            }

            metadata.declaring_method_ids.insert(aliased_method_name, *declaring_method_id);
            metadata.inheritable_method_ids.insert(aliased_method_name, *declaring_method_id);
        }
    }
}
