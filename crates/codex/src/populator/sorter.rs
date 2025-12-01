use mago_atom::Atom;
use mago_atom::AtomSet;

use crate::metadata::CodebaseMetadata;

pub fn sort_class_likes(codebase: &CodebaseMetadata, class_likes_to_repopulate: &AtomSet) -> Vec<Atom> {
    let mut sorted = Vec::with_capacity(class_likes_to_repopulate.len());
    let mut visited = AtomSet::default();
    let mut visiting = AtomSet::default();

    for class_like in class_likes_to_repopulate {
        visit(class_like, codebase, class_likes_to_repopulate, &mut visited, &mut visiting, &mut sorted);
    }

    sorted
}

fn visit(
    class_like: &Atom,
    codebase: &CodebaseMetadata,
    class_likes_to_repopulate: &AtomSet,
    visited: &mut AtomSet,
    visiting: &mut AtomSet,
    sorted: &mut Vec<Atom>,
) {
    if visited.contains(class_like) {
        return;
    }

    if visiting.contains(class_like) {
        return;
    }

    visiting.insert(*class_like);

    if let Some(metadata) = codebase.class_likes.get(class_like) {
        if let Some(parent) = metadata.direct_parent_class
            && class_likes_to_repopulate.contains(&parent)
        {
            visit(&parent, codebase, class_likes_to_repopulate, visited, visiting, sorted);
        }

        for trait_name in &metadata.used_traits {
            if class_likes_to_repopulate.contains(trait_name) {
                visit(trait_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for interface_name in &metadata.direct_parent_interfaces {
            if class_likes_to_repopulate.contains(interface_name) {
                visit(interface_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for required in &metadata.require_extends {
            if class_likes_to_repopulate.contains(required) {
                visit(required, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for required in &metadata.require_implements {
            if class_likes_to_repopulate.contains(required) {
                visit(required, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }

        for (source_class_name, _, _) in metadata.imported_type_aliases.values() {
            if class_likes_to_repopulate.contains(source_class_name) {
                visit(source_class_name, codebase, class_likes_to_repopulate, visited, visiting, sorted);
            }
        }
    }

    visiting.remove(class_like);
    visited.insert(*class_like);
    sorted.push(*class_like);
}
