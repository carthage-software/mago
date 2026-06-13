//! `Object \ B` precision via a `Negated` conjunct on the surviving
//! object, expressed through the
//! [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected) wrapper.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;
use crate::world::World;

/// `Object \ B` records the removed side as a `Negated` conjunct of the
/// input. The removed side may be another `Object` (descendant,
/// sibling, or same-class with different args) or a structural conjunct
/// (`HasMethod` / `HasProperty` / `ObjectShape`). For the strict
/// bare-descendant case the exclusion binds to the bare descendant
/// class so the whole nominal subtree is excluded.
pub(in crate::ty::subtract) fn object_descendant_minus<'arena, S, A, W>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Atom::Object(input_payload) = input else {
        return None;
    };

    let removed_is_object = matches!(removed, Atom::Object(_));
    let removed_is_intersected = matches!(removed, Atom::Intersected(_));
    let removed_is_structural = matches!(removed, Atom::HasMethod(_) | Atom::HasProperty(_) | Atom::ObjectShape(_));

    if !removed_is_object && !removed_is_structural && !removed_is_intersected {
        return None;
    }

    let (head, exclude_atom) = if let Atom::Intersected(removed_payload) = removed {
        match *removed_payload.head {
            Atom::Object(removed_head_payload) => {
                let descends = input_payload.name != removed_head_payload.name
                    && world.descends_from(removed_head_payload.name, input_payload.name)
                    && removed_head_payload.type_arguments.is_none();

                let atom = if descends { builder.object(*removed_head_payload) } else { removed };
                (Some(*removed_payload.head), atom)
            }
            Atom::HasMethod(_) | Atom::HasProperty(_) | Atom::ObjectShape(_) => (Some(*removed_payload.head), removed),
            _ => (None, removed),
        }
    } else if let Atom::Object(removed_payload) = removed {
        let descends =
            input_payload.name != removed_payload.name && world.descends_from(removed_payload.name, input_payload.name);

        let atom = if descends && removed_payload.type_arguments.is_none() {
            builder.object(*removed_payload)
        } else {
            removed
        };
        (None, atom)
    } else {
        (None, removed)
    };

    if head.is_none() && removed_is_intersected {
        return None;
    }

    let exclude_type = builder.union_of(&[exclude_atom]);
    let negated = builder.negated(exclude_type);
    Some(vec![builder.intersected(input, &[negated])])
}
