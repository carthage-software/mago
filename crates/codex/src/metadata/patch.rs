use mago_word::Word;
use mago_word::WordMap;

use crate::metadata::CodebaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::inherited_methods::inheritance_depth;

/// DocBlock flags that patches may change on any declaration.
pub(crate) const PATCHABLE_COMMON_DOCBLOCK_FLAGS: MetadataFlags = MetadataFlags::from_bits(
    MetadataFlags::DEPRECATED.bits()
        | MetadataFlags::INTERNAL.bits()
        | MetadataFlags::EXPERIMENTAL.bits()
        | MetadataFlags::API.bits(),
);

/// DocBlock flags that patches may change on functions and methods.
pub(crate) const PATCHABLE_FUNCTION_DOCBLOCK_FLAGS: MetadataFlags = MetadataFlags::from_bits(
    PATCHABLE_COMMON_DOCBLOCK_FLAGS.bits()
        | MetadataFlags::PURE.bits()
        | MetadataFlags::MUTATION_FREE.bits()
        | MetadataFlags::EXTERNAL_MUTATION_FREE.bits()
        | MetadataFlags::MUST_USE.bits()
        | MetadataFlags::NO_NAMED_ARGUMENTS.bits()
        | MetadataFlags::SUSPENDS_FIBER.bits(),
);

/// DocBlock flags that patches may change on classes, interfaces, traits, and enums.
pub(crate) const PATCHABLE_CLASS_DOCBLOCK_FLAGS: MetadataFlags = MetadataFlags::from_bits(
    PATCHABLE_COMMON_DOCBLOCK_FLAGS.bits()
        | MetadataFlags::CONSISTENT_CONSTRUCTOR.bits()
        | MetadataFlags::CONSISTENT_TEMPLATES.bits(),
);

pub(crate) fn merge_explicit_docblock_flags(
    target: &mut MetadataFlags,
    patch_flags: MetadataFlags,
    explicit_docblock_flags: MetadataFlags,
    allowed_flags: MetadataFlags,
) {
    let mask = explicit_docblock_flags & allowed_flags;
    target.remove(mask);
    target.insert(patch_flags & mask);
}

pub(super) fn propagate_inherited_docblock_flags(codebase: &mut CodebaseMetadata, mut classes: Vec<Word>) {
    classes.retain(|name| {
        codebase.patch_class_likes.get(name).is_some_and(|patch| {
            patch.explicit_docblock_flags.contains(MetadataFlags::CONSISTENT_TEMPLATES)
                || patch.constants.values().any(|metadata| metadata.explicit_docblock_flags != MetadataFlags::empty())
                || patch.properties.values().any(|metadata| metadata.explicit_docblock_flags != MetadataFlags::empty())
        })
    });
    let mut depths = WordMap::default();
    classes.sort_unstable_by_key(|name| (inheritance_depth(*name, codebase, &mut depths, &mut Vec::new()), *name));

    for name in classes {
        let Some(source) = codebase.class_likes.get(&name) else { continue };
        let Some(patch) = codebase.patch_class_likes.get(&name) else { continue };
        let constants: Vec<_> = patch
            .constants
            .iter()
            .filter_map(|(name, patch)| {
                source
                    .constants
                    .get(name)
                    .map(|source| (*name, source.span, patch.flags, patch.explicit_docblock_flags))
            })
            .collect();
        let properties: Vec<_> = patch
            .properties
            .iter()
            .filter_map(|(name, patch)| {
                source
                    .properties
                    .get(name)
                    .map(|source| (*name, source.span, patch.flags, patch.explicit_docblock_flags))
            })
            .collect();
        let consistent_templates = patch.explicit_docblock_flags.contains(MetadataFlags::CONSISTENT_TEMPLATES)
            && source.flags.has_consistent_templates();
        let descendants = codebase.all_class_like_descendants.get(&name).into_iter().flatten().copied();
        let targets: Vec<_> = std::iter::once(name).chain(descendants).collect();

        for target in targets {
            let Some(metadata) = codebase.class_likes.get_mut(&target) else { continue };
            if consistent_templates && metadata.all_parent_classes.contains(&name) {
                metadata.flags.insert(MetadataFlags::CONSISTENT_TEMPLATES);
            }
            for &(name, span, flags, explicit) in &constants {
                if let Some(constant) = metadata.constants.get_mut(&name)
                    && constant.span == span
                {
                    merge_explicit_docblock_flags(
                        &mut constant.flags,
                        flags,
                        explicit,
                        PATCHABLE_COMMON_DOCBLOCK_FLAGS,
                    );
                }
            }
            for &(name, span, flags, explicit) in &properties {
                if let Some(property) = metadata.properties.get_mut(&name)
                    && property.span == span
                {
                    merge_explicit_docblock_flags(
                        &mut property.flags,
                        flags,
                        explicit,
                        PATCHABLE_COMMON_DOCBLOCK_FLAGS,
                    );
                }
            }
        }
    }
}
