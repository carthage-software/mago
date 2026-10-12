use foldhash::HashMap;
use foldhash::HashSet;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::flags::MetadataFlags;

pub(super) struct InheritedPatchMethod {
    pub(super) target: MethodIdentifier,
    pub(super) source: Option<MethodIdentifier>,
}

pub(super) fn reset_patch_methods(codebase: &mut CodebaseMetadata) {
    let mut methods: Vec<_> = codebase
        .patch_function_likes
        .keys()
        .copied()
        .filter(|key| {
            !key.0.is_empty()
                && codebase.function_likes.get(key).is_some_and(|method| method.flags.is_patch())
                && codebase.class_likes.get(&key.0).is_some_and(|class| !class.flags.is_user_defined())
        })
        .collect();

    let constructor = word("__construct");
    methods.extend(codebase.patch_class_likes.values().filter_map(|patch| {
        let class = codebase.class_likes.get(&patch.name)?;
        if !patch.flags.has_consistent_constructor()
            || class.flags.is_user_defined()
            || class.explicit_docblock_flags.contains(MetadataFlags::CONSISTENT_CONSTRUCTOR)
        {
            return None;
        }

        let key = (class.name, constructor);
        codebase
            .function_likes
            .get(&key)
            .filter(|method| method.name_span.is_none() && method.span == class.span)
            .map(|_| key)
    }));

    for (class, method) in methods {
        codebase.function_likes.remove(&(class, method));
        if let Some(class) = codebase.class_likes.get_mut(&class) {
            class.methods.remove(&method);
            class.declaring_method_ids.remove(&method);
            class.appearing_method_ids.remove(&method);
            class.inheritable_method_ids.remove(&method);
            class.overridden_method_ids.remove(&method);
            class.flags.remove(MetadataFlags::POPULATED);
        }
    }
}

/// Resolves all patch classes against the original declarations with one shared cache.
pub(super) fn collect_inherited_patch_methods(codebase: &CodebaseMetadata) -> WordMap<WordSet> {
    let mut resolver = InheritedMethodResolver::new(codebase, HashSet::default());
    codebase
        .patch_class_likes
        .values()
        .filter(|patch| codebase.class_likes.get(&patch.name).is_some_and(|target| !target.flags.is_user_defined()))
        .map(|patch| {
            let methods = patch
                .methods
                .iter()
                .copied()
                .chain(patch.flags.has_consistent_constructor().then(|| word("__construct")))
                .filter(|name| {
                    resolver.find_source(MethodIdentifier::new(patch.name, *name), &mut HashSet::default()).0.is_some()
                })
                .collect();
            (patch.name, methods)
        })
        .collect()
}

/// Resolves all inherited overrides with a separate shared cache and orders ancestors first.
/// Planned overrides count as declarations, so sources stay stable while their flags change.
pub(super) fn resolve_inherited_patch_methods(
    codebase: &CodebaseMetadata,
    mut methods: Vec<MethodIdentifier>,
) -> Vec<InheritedPatchMethod> {
    let mut depths = WordMap::default();
    methods.sort_unstable_by_key(|method| {
        (inheritance_depth(method.get_class_name(), codebase, &mut depths, &mut Vec::new()), *method)
    });

    let mut resolver = InheritedMethodResolver::new(codebase, methods.iter().copied().collect());
    methods
        .into_iter()
        .map(|target| InheritedPatchMethod { target, source: resolver.find_source(target, &mut HashSet::default()).0 })
        .collect()
}

struct InheritedMethodResolver<'metadata> {
    codebase: &'metadata CodebaseMetadata,
    overrides: HashSet<MethodIdentifier>,
    sources: HashMap<MethodIdentifier, Option<MethodIdentifier>>,
}

impl<'metadata> InheritedMethodResolver<'metadata> {
    fn new(codebase: &'metadata CodebaseMetadata, overrides: HashSet<MethodIdentifier>) -> Self {
        Self { codebase, overrides, sources: HashMap::default() }
    }

    fn find_source(
        &mut self,
        method: MethodIdentifier,
        path: &mut HashSet<MethodIdentifier>,
    ) -> (Option<MethodIdentifier>, bool) {
        if let Some(source) = self.sources.get(&method) {
            return (*source, false);
        }
        if !path.insert(method) {
            return (None, true);
        }

        let mut source: Option<MethodIdentifier> = None;
        let mut cyclic = false;
        for ancestor in self.direct_method_sources(method) {
            if source
                .is_some_and(|current| !self.codebase.can_replace_inherited_method(current, ancestor.get_class_name()))
            {
                continue;
            }
            if path.contains(&ancestor) {
                cyclic = true;
                continue;
            }
            let (inherited, ancestor_cyclic) = if self.overrides.contains(&ancestor)
                || self.codebase.function_likes.contains_key(&ancestor.get_key())
            {
                (Some(ancestor), false)
            } else {
                self.find_source(ancestor, path)
            };
            cyclic |= ancestor_cyclic;
            if inherited.is_some() {
                source = inherited;
            }
        }

        path.remove(&method);
        // A cycle can make resolution depend on the active path, so do not memoize it.
        if !cyclic {
            self.sources.insert(method, source);
        }
        (source, cyclic)
    }

    fn direct_method_sources(&self, method: MethodIdentifier) -> Vec<MethodIdentifier> {
        let Some(class) = self.codebase.class_likes.get(&method.get_class_name()) else { return Vec::new() };
        let name = method.get_method_name();
        let trait_method = class.trait_alias_map.get(&name).copied().unwrap_or(name);
        let mut traits: Vec<_> = class.direct_trait_names().collect();
        traits.sort_unstable();

        let mut sources: Vec<_> = traits
            .into_iter()
            .filter(|source| class.trait_alias_applies(*source, trait_method, name, self.codebase))
            .map(|source| MethodIdentifier::new(source, trait_method))
            .collect();
        sources.extend(class.direct_parent_class.map(|parent| MethodIdentifier::new(parent, name)));

        let mut interfaces: Vec<_> = class.direct_parent_interfaces.iter().copied().collect();
        interfaces.sort_unstable();
        sources.extend(interfaces.into_iter().map(|parent| MethodIdentifier::new(parent, name)));
        sources
    }
}

pub(super) fn inheritance_depth(
    class: Word,
    codebase: &CodebaseMetadata,
    depths: &mut WordMap<usize>,
    path: &mut Vec<Word>,
) -> usize {
    if let Some(depth) = depths.get(&class) {
        return *depth;
    }
    if path.contains(&class) {
        return 0;
    }

    let Some(metadata) = codebase.class_likes.get(&class) else { return 0 };
    path.push(class);
    let depth = metadata
        .direct_trait_names()
        .chain(metadata.direct_parent_class)
        .chain(metadata.direct_parent_interfaces.iter().copied())
        .map(|ancestor| inheritance_depth(ancestor, codebase, depths, path).saturating_add(1))
        .max()
        .unwrap_or(0);
    path.pop();
    depths.insert(class, depth);
    depth
}
