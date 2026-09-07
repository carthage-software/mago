use mago_word::Word;
use mago_word::WordSet;

use crate::metadata::CodebaseMetadata;

pub fn sort_class_likes(codebase: &CodebaseMetadata, class_likes_to_repopulate: &WordSet) -> Vec<Word> {
    let mut sorted = Vec::with_capacity(class_likes_to_repopulate.len());
    let mut visited = WordSet::default();
    let mut visiting = WordSet::default();

    for class_like in class_likes_to_repopulate {
        visit(*class_like, codebase, class_likes_to_repopulate, &mut visited, &mut visiting, &mut sorted);
    }

    sorted
}

#[inline]
fn dependency_to_repopulate(
    dependency: Word,
    codebase: &CodebaseMetadata,
    class_likes_to_repopulate: &WordSet,
) -> Option<Word> {
    if class_likes_to_repopulate.contains(&dependency) {
        return Some(dependency);
    }

    if codebase.class_like_aliases.is_empty() || codebase.class_likes.contains_key(&dependency) {
        return None;
    }

    codebase.class_like_aliases.get(&dependency).copied().filter(|actual| class_likes_to_repopulate.contains(actual))
}

fn visit(
    class_like: Word,
    codebase: &CodebaseMetadata,
    class_likes_to_repopulate: &WordSet,
    visited: &mut WordSet,
    visiting: &mut WordSet,
    sorted: &mut Vec<Word>,
) {
    if visited.contains(&class_like) {
        return;
    }

    if visiting.contains(&class_like) {
        return;
    }

    visiting.insert(class_like);

    if let Some(metadata) = codebase.class_likes.get(&class_like) {
        if let Some(parent) = metadata
            .direct_parent_class
            .and_then(|parent| dependency_to_repopulate(parent, codebase, class_likes_to_repopulate))
        {
            visit(parent, codebase, class_likes_to_repopulate, visited, visiting, sorted);
        }

        for trait_name in &metadata.used_traits {
            if let Some(trait_name) = dependency_to_repopulate(*trait_name, codebase, class_likes_to_repopulate) {
                visit(trait_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for interface_name in &metadata.direct_parent_interfaces {
            if let Some(interface_name) = dependency_to_repopulate(*interface_name, codebase, class_likes_to_repopulate)
            {
                visit(interface_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for required in &metadata.require_extends {
            if let Some(required) = dependency_to_repopulate(*required, codebase, class_likes_to_repopulate) {
                visit(required, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for required in &metadata.require_implements {
            if let Some(required) = dependency_to_repopulate(*required, codebase, class_likes_to_repopulate) {
                visit(required, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for (source_class_name, _, _) in metadata.imported_type_aliases.values() {
            if let Some(source_class_name) =
                dependency_to_repopulate(*source_class_name, codebase, class_likes_to_repopulate)
            {
                visit(source_class_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }
    }

    visiting.remove(&class_like);
    visited.insert(class_like);
    sorted.push(class_like);
}
