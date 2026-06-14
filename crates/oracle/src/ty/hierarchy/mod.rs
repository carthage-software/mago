//! Cross-hierarchy template-argument propagation.
//!
//! When a class extends or implements a parameterised parent, it passes
//! type-arguments to the parent - and those arguments may mention the
//! child's templates. The analyzer needs an O(1) answer to "what does
//! `child` ultimately pass to `ancestor`'s `position`-th type parameter"
//! for any direct or transitive ancestor.
//!
//! This module precomputes the closure once and exposes it through
//! [`Hierarchy::arg`] / [`Hierarchy::args`]. Plug it into a
//! [`World`] implementation's
//! [`inherited_template_argument`](crate::world::World::inherited_template_argument)
//! and the O(depth × arity) cost vanishes from every query.
//!
//! Construction is two-step: register direct edges on a
//! [`HierarchyBuilder`], then [`build`](HierarchyBuilder::build) walks them
//! and composes each transitive `(child, ancestor)` pair by substituting
//! every intermediate parent's templates with the child's actual arguments
//! to that parent. The substitution algorithm is
//! [`crate::ty::template::substitute`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use mago_allocator::Arena;

use crate::name::Name;
use crate::ty::Type;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::template::substitute;
use crate::world::World;

/// Builder collecting direct parent edges before transitive composition.
#[derive(Debug, Default, Clone)]
pub struct HierarchyBuilder<'arena> {
    edges: BTreeMap<(Name<'arena>, Name<'arena>), Vec<Type<'arena>>>,
}

impl<'arena> HierarchyBuilder<'arena> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `child extends/implements parent<args>` where `args` is
    /// expressed in `child`'s template namespace. Idempotent on
    /// `(child, parent)`; the latest call wins.
    #[inline]
    pub fn add_edge(&mut self, child: Name<'arena>, parent: Name<'arena>, arguments: Vec<Type<'arena>>) {
        self.edges.insert((child, parent), arguments);
    }

    /// Compute the transitive closure of inherited template arguments.
    /// `world` supplies template-name-to-position lookups for each
    /// intermediate class via
    /// [`template_parameter_index`](World::template_parameter_index).
    #[inline]
    pub fn build<S, A, W>(self, world: &W, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Hierarchy<'arena>
    where
        S: Arena,
        A: Arena,
        W: World<'arena>,
    {
        let mut parents_of: BTreeMap<Name<'arena>, Vec<Name<'arena>>> = BTreeMap::new();
        for &(child, parent) in self.edges.keys() {
            parents_of.entry(child).or_default().push(parent);
        }

        let mut composed: BTreeMap<(Name<'arena>, Name<'arena>), Vec<Type<'arena>>> = self.edges.clone();

        let children: Vec<Name<'arena>> = parents_of.keys().copied().collect();
        for child in children {
            let mut visiting: BTreeSet<Name<'arena>> = BTreeSet::new();
            walk(child, &self.edges, &parents_of, &mut composed, &mut visiting, world, builder);
        }

        Hierarchy { composed }
    }
}

#[inline]
fn walk<'arena, S, A, W>(
    child: Name<'arena>,
    edges: &BTreeMap<(Name<'arena>, Name<'arena>), Vec<Type<'arena>>>,
    parents_of: &BTreeMap<Name<'arena>, Vec<Name<'arena>>>,
    composed: &mut BTreeMap<(Name<'arena>, Name<'arena>), Vec<Type<'arena>>>,
    visiting: &mut BTreeSet<Name<'arena>>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if !visiting.insert(child) {
        return;
    }

    let Some(parents) = parents_of.get(&child) else {
        visiting.remove(&child);
        return;
    };

    for &parent in parents.clone().iter() {
        walk(parent, edges, parents_of, composed, visiting, world, builder);

        let Some(child_to_parent) = edges.get(&(child, parent)).cloned() else {
            continue;
        };

        let parent_entity = DefiningEntity::ClassLike(parent);

        let grandparents: Vec<Name<'arena>> = composed
            .keys()
            .filter(|(parent_child, _)| *parent_child == parent)
            .map(|(_, ancestor)| *ancestor)
            .collect();

        for grandparent in grandparents {
            if grandparent == child || grandparent == parent {
                continue;
            }

            if composed.contains_key(&(child, grandparent)) {
                continue;
            }

            let Some(parent_to_grandparent) = composed.get(&(parent, grandparent)).cloned() else {
                continue;
            };

            let composed_arguments: Vec<Type<'arena>> = parent_to_grandparent
                .into_iter()
                .map(|argument| {
                    substitute(
                        argument,
                        &|parameter: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
                            if parameter.defining_entity != parent_entity {
                                return None;
                            }

                            let position = world.template_parameter_index(parent, parameter.name)?;
                            child_to_parent.get(position).copied()
                        },
                        builder,
                    )
                })
                .collect();

            composed.insert((child, grandparent), composed_arguments);
        }
    }

    visiting.remove(&child);
}

/// Precomputed transitive closure of cross-hierarchy template arguments.
/// O(1) lookup keyed on `(child, ancestor)`.
#[derive(Debug, Clone, Default)]
pub struct Hierarchy<'arena> {
    composed: BTreeMap<(Name<'arena>, Name<'arena>), Vec<Type<'arena>>>,
}

impl<'arena> Hierarchy<'arena> {
    /// Composed type-argument list `child` passes to `ancestor`, in
    /// `ancestor`'s declaration order, expressed in `child`'s template
    /// namespace. `None` when `child` does not descend from `ancestor` or
    /// no edges were registered along the path.
    #[inline]
    pub fn args(&self, child: Name<'arena>, ancestor: Name<'arena>) -> Option<&[Type<'arena>]> {
        self.composed.get(&(child, ancestor)).map(Vec::as_slice)
    }

    /// Single positional argument; convenience for [`Hierarchy::args`]
    /// followed by `[position]`.
    #[inline]
    #[must_use]
    pub fn arg(&self, child: Name<'arena>, ancestor: Name<'arena>, position: usize) -> Option<Type<'arena>> {
        self.args(child, ancestor).and_then(|arguments| arguments.get(position).copied())
    }

    /// Iterate every `((child, ancestor), args)` triple recorded in the
    /// closure. Useful for building reverse indexes or for a wrapper
    /// [`World`] that delegates
    /// [`descends_from`](World::descends_from).
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = ((Name<'arena>, Name<'arena>), &[Type<'arena>])> {
        self.composed.iter().map(|(&pair, arguments)| (pair, arguments.as_slice()))
    }
}
