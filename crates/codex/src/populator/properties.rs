use crate::metadata::class_like::ClassLikeMetadata;

/// Inherits property declarations and appearances from a parent class-like.
/// Updates `declaring_property_ids`, `appearing_property_ids`, etc.
pub fn inherit_properties_from_parent(metadata: &mut ClassLikeMetadata, parent_metadata: &ClassLikeMetadata) {
    let classlike_name = metadata.name;
    let is_trait = metadata.kind.is_trait();
    let parent_is_trait = parent_metadata.kind.is_trait();

    for (property_name, appearing_classlike) in &parent_metadata.appearing_property_ids {
        if metadata.has_appearing_property(property_name) {
            continue;
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata
            .appearing_property_ids
            .insert(*property_name, if is_trait { classlike_name } else { *appearing_classlike });

        if parent_is_trait && let Some(parent_property) = parent_metadata.properties.get(property_name) {
            metadata.properties.entry(*property_name).or_insert_with(|| parent_property.clone());
        }
    }

    for (property_name, declaring_classlike) in &parent_metadata.declaring_property_ids {
        if metadata.declaring_property_ids.contains_key(property_name) {
            if !parent_is_trait && metadata.properties.get(property_name).is_some_and(|p| p.flags.is_magic_property()) {
                metadata.properties.remove(property_name);
            } else {
                continue;
            }
        }

        if !parent_is_trait
            && let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
            && parent_property_metadata.is_final()
        {
            continue;
        }

        metadata.declaring_property_ids.insert(*property_name, *declaring_classlike);
    }

    for (property_name, inheritable_classlike) in &parent_metadata.inheritable_property_ids {
        let mut is_overridable = true;
        if !parent_is_trait {
            if let Some(parent_property_metadata) = parent_metadata.properties.get(property_name)
                && parent_property_metadata.is_final()
            {
                is_overridable = false;
            }

            if is_overridable {
                metadata.overridden_property_ids.entry(*property_name).or_default().insert(*inheritable_classlike);
            }
        }

        if is_overridable {
            metadata.inheritable_property_ids.insert(*property_name, *inheritable_classlike);
        }
    }
}
