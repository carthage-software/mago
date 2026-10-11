use foldhash::HashMap;
use foldhash::HashSet;
use mago_word::Word;
use mago_word::WordMap;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::visibility::Visibility;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct TraitMethodAdaptations {
    pub alias_sources: WordMap<Word>,
    pub excluded_methods: HashSet<MethodIdentifier>,
    pub modifiers: HashMap<MethodIdentifier, TraitMethodModifiers>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct TraitMethodModifiers {
    pub visibility: Option<Visibility>,
    pub is_final: bool,
}

impl ClassLikeMetadata {
    pub(crate) fn excludes_trait_method(&self, source: Word, method: Word, codebase: &CodebaseMetadata) -> bool {
        self.trait_method_adaptations.as_ref().is_some_and(|adaptations| {
            adaptations.excluded_methods.iter().any(|excluded| {
                excluded.get_method_name() == method
                    && codebase.resolve_class_like_word(excluded.get_class_name()) == Some(source)
            })
        })
    }

    pub(crate) fn trait_alias_applies(
        &self,
        source: Word,
        method: Word,
        alias: Word,
        codebase: &CodebaseMetadata,
    ) -> bool {
        let qualified =
            self.trait_method_adaptations.as_ref().and_then(|adaptations| adaptations.alias_sources.get(&alias));
        qualified.map_or_else(
            || !self.excludes_trait_method(source, method, codebase),
            |qualified| codebase.resolve_class_like_word(*qualified) == Some(source),
        )
    }

    pub(crate) fn trait_method_modifiers(
        &self,
        name: Word,
        declaring: MethodIdentifier,
        codebase: &CodebaseMetadata,
    ) -> TraitMethodModifiers {
        let mut result = TraitMethodModifiers {
            visibility: self.trait_visibility_map.get(&name).copied(),
            is_final: self.trait_final_map.contains(&name),
        };

        let Some(adaptations) = &self.trait_method_adaptations else { return result };
        for (source, modifiers) in &adaptations.modifiers {
            if source.get_method_name() != name {
                continue;
            }

            let Some(source_class) = codebase.get_class_like_by_word(source.get_class_name()) else { continue };
            if !self.excludes_trait_method(source_class.name, name, codebase)
                && source_class.declaring_method_ids.get(&name) == Some(&declaring)
            {
                result.visibility = modifiers.visibility.or(result.visibility);
                result.is_final |= modifiers.is_final;
            }
        }

        result
    }
}

#[derive(Default)]
struct TraitModifierPath {
    modifiers: TraitMethodModifiers,
    visibility_source: Option<MethodIdentifier>,
    conflicting_visibility: bool,
}

impl CodebaseMetadata {
    pub(super) fn effective_trait_method_modifiers(
        &self,
        class: &ClassLikeMetadata,
        name: Word,
        declaring: MethodIdentifier,
    ) -> TraitMethodModifiers {
        self.trait_modifiers_on_path(class, name, declaring, &mut Vec::new()).modifiers
    }

    fn trait_modifiers_on_path(
        &self,
        class: &ClassLikeMetadata,
        name: Word,
        declaring: MethodIdentifier,
        path: &mut Vec<MethodIdentifier>,
    ) -> TraitModifierPath {
        let owner = class
            .appearing_method_ids
            .get(&name)
            .and_then(|appearing| self.get_class_like_by_word(appearing.get_class_name()))
            .unwrap_or(class);
        if declaring.get_class_name() == owner.name
            || !self.get_class_like_by_word(declaring.get_class_name()).is_some_and(|class| class.kind.is_trait())
        {
            return TraitModifierPath::default();
        }

        let current = MethodIdentifier::new(owner.name, name);
        if path.contains(&current) {
            return TraitModifierPath { conflicting_visibility: true, ..Default::default() };
        }

        path.push(current);
        let local = owner.trait_method_modifiers(name, declaring, self);
        let mut result = TraitModifierPath {
            modifiers: local,
            visibility_source: local.visibility.map(|_| current),
            conflicting_visibility: false,
        };

        for (source, source_name) in self.trait_method_sources(owner, name, declaring) {
            let inherited = self.trait_modifiers_on_path(source, source_name, declaring, path);
            result.modifiers.is_final |= inherited.modifiers.is_final;
            if local.visibility.is_some() {
                continue;
            }

            result.conflicting_visibility |= inherited.conflicting_visibility;
            match (result.visibility_source, inherited.visibility_source) {
                (None, Some(source)) => {
                    result.modifiers.visibility = inherited.modifiers.visibility;
                    result.visibility_source = Some(source);
                }
                (Some(first), Some(next)) if first != next => result.conflicting_visibility = true,
                _ => {}
            }
        }

        path.pop();
        if result.conflicting_visibility {
            result.modifiers.visibility = None;
        }

        result
    }

    fn trait_method_sources<'metadata>(
        &'metadata self,
        owner: &'metadata ClassLikeMetadata,
        name: Word,
        declaring: MethodIdentifier,
    ) -> impl Iterator<Item = (&'metadata ClassLikeMetadata, Word)> + 'metadata {
        let source_name = owner.trait_alias_map.get(&name).copied().unwrap_or(name);
        owner
            .direct_trait_names()
            .filter_map(move |source| self.get_class_like_by_word(source))
            .filter(move |source| {
                source.declaring_method_ids.get(&source_name) == Some(&declaring)
                    && owner.trait_alias_applies(source.name, source_name, name, self)
            })
            .map(move |source| (source, source_name))
    }
}
