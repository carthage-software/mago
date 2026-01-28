use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;

use crate::metadata::CodebaseMetadata;

/// A graph of class-like dependencies, grouped by level.
///
/// This structure computes the dependency depth of each class-like and groups them
/// by level, allowing class-likes at the same level to be processed in parallel
/// since they don't depend on each other.
///
/// # Levels
///
/// - Level 0: Class-likes with no dependencies in the input set
/// - Level N: Class-likes whose deepest dependency is at level N-1
///
/// # Example
///
/// ```text
/// // Given these classes:
/// // - A (no parent)
/// // - B extends A
/// // - C extends B
/// // - D (no parent)
/// //
/// // The levels would be:
/// // Level 0: [A, D]  - can be processed in parallel
/// // Level 1: [B]     - depends on A
/// // Level 2: [C]     - depends on B
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    levels: Vec<Vec<Atom>>,
    count: usize,
}

impl DependencyGraph {
    /// Build a dependency graph from a set of classes.
    ///
    /// This analyzes the inheritance relationships between the given classes
    /// and groups them by dependency depth. Only dependencies within the
    /// provided set are considered.
    ///
    /// # Arguments
    ///
    /// * `codebase` - The codebase containing class metadata
    /// * `classes` - The set of classes to include in the graph
    ///
    /// # Returns
    ///
    /// A `DependencyGraph` with classes grouped by level.
    pub fn build(codebase: &CodebaseMetadata, classes: &AtomSet) -> Self {
        if classes.is_empty() {
            return Self { levels: Vec::new(), count: 0 };
        }

        let mut depths: AtomMap<usize> = AtomMap::default();
        let mut visiting = AtomSet::default();

        for class in classes {
            compute_depth(*class, codebase, classes, &mut depths, &mut visiting);
        }

        let max_depth = depths.values().copied().max().unwrap_or(0);
        let mut levels: Vec<Vec<Atom>> = vec![Vec::new(); max_depth + 1];
        for (class, depth) in &depths {
            levels[*depth].push(*class);
        }

        Self { levels, count: classes.len() }
    }

    /// Get classes grouped by level for parallel iteration.
    ///
    /// Classes at the same level have no dependencies on each other and can
    /// be processed in parallel. Levels should be processed in order (0, 1, 2, ...)
    /// to ensure dependencies are resolved before dependents.
    #[inline]
    pub fn levels(&self) -> &[Vec<Atom>] {
        &self.levels
    }

    /// Get the total number of classes in the graph.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the graph is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the number of levels in the graph.
    #[inline]
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// Iterate over levels, yielding (level_index, classes_at_level).
    #[inline]
    pub fn iter_levels(&self) -> impl Iterator<Item = (usize, &[Atom])> {
        self.levels.iter().enumerate().map(|(i, v)| (i, v.as_slice()))
    }
}

/// Compute the depth of a class using memoized DFS.
///
/// The depth is defined as:
/// - 0 if the class has no dependencies in the input set
/// - max(depth of dependencies) + 1 otherwise
///
/// Cycles are handled by returning 0 when a class is already being visited.
fn compute_depth(
    class: Atom,
    codebase: &CodebaseMetadata,
    classes: &AtomSet,
    depths: &mut AtomMap<usize>,
    visiting: &mut AtomSet,
) -> usize {
    if let Some(&depth) = depths.get(&class) {
        return depth;
    }

    if visiting.contains(&class) {
        return 0;
    }

    visiting.insert(class);
    let deps = get_dependencies(class, codebase, classes);
    let max_dep_depth =
        deps.into_iter().map(|dep| compute_depth(dep, codebase, classes, depths, visiting)).max().unwrap_or(0);
    let depth = if max_dep_depth > 0 || has_any_dependency(class, codebase, classes) { max_dep_depth + 1 } else { 0 };

    visiting.remove(&class);
    depths.insert(class, depth);

    depth
}

/// Check if a class has any dependency in the input set.
fn has_any_dependency(class: Atom, codebase: &CodebaseMetadata, classes: &AtomSet) -> bool {
    let Some(metadata) = codebase.class_likes.get(&class) else {
        return false;
    };

    if let Some(parent) = metadata.direct_parent_class
        && classes.contains(&parent)
    {
        return true;
    }

    for trait_name in &metadata.used_traits {
        if classes.contains(trait_name) {
            return true;
        }
    }

    for interface in &metadata.direct_parent_interfaces {
        if classes.contains(interface) {
            return true;
        }
    }

    for required in &metadata.require_extends {
        if classes.contains(required) {
            return true;
        }
    }

    for required in &metadata.require_implements {
        if classes.contains(required) {
            return true;
        }
    }

    for (source_class, _, _) in metadata.imported_type_aliases.values() {
        if classes.contains(source_class) {
            return true;
        }
    }

    false
}

/// Get all dependencies of a class that are in the input set.
fn get_dependencies(class: Atom, codebase: &CodebaseMetadata, classes: &AtomSet) -> Vec<Atom> {
    let Some(metadata) = codebase.class_likes.get(&class) else {
        return Vec::new();
    };

    let mut deps = Vec::new();

    if let Some(parent) = metadata.direct_parent_class
        && classes.contains(&parent)
    {
        deps.push(parent);
    }

    for trait_name in &metadata.used_traits {
        if classes.contains(trait_name) {
            deps.push(*trait_name);
        }
    }

    for interface in &metadata.direct_parent_interfaces {
        if classes.contains(interface) {
            deps.push(*interface);
        }
    }

    for required in &metadata.require_extends {
        if classes.contains(required) {
            deps.push(*required);
        }
    }

    for required in &metadata.require_implements {
        if classes.contains(required) {
            deps.push(*required);
        }
    }

    for (source_class, _, _) in metadata.imported_type_aliases.values() {
        if classes.contains(source_class) {
            deps.push(*source_class);
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let codebase = CodebaseMetadata::default();
        let classes = AtomSet::default();

        let graph = DependencyGraph::build(&codebase, &classes);

        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert_eq!(graph.depth(), 0);
        assert!(graph.levels().is_empty());
    }
}
