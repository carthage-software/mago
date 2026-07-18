//! Short-circuiting deep walkers for boolean queries on a [`Type`].
//!
//! Where transformation rebuilds a type by applying a closure at every atom
//! position, inspection *queries*: the closure is a predicate, and the
//! walker stops as soon as the answer is known.
//!
//! - [`any`] - `true` iff at least one atom (at any depth) satisfies the
//!   predicate. Stops at the first `true`.
//! - [`all`] - `true` iff every atom (at every depth) satisfies the
//!   predicate. Stops at the first `false`. Vacuously `true` for the empty
//!   type.
//!
//! The walk descends through every nested-type carrier: object type
//! arguments, intersections, list elements and known elements, keyed-array
//! parameters and known items, iterable key/value, object-shape known
//! properties, class-like-string constraints, generic-parameter constraints,
//! reference type arguments, conditional operands, every derived variant,
//! and callable signatures (return / parameters / throws). The predicate is
//! called at every level and never twice on the same atom.

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;

/// `true` iff at least one atom in `ty` (at any depth) satisfies
/// `predicate`. Stops walking on the first match.
#[inline]
pub fn any<'arena, F>(ty: Type<'arena>, mut predicate: F) -> bool
where
    F: FnMut(Atom<'arena>) -> bool,
{
    any_with(ty, &mut predicate)
}

/// `true` iff every atom in `ty` (at every depth) satisfies `predicate`.
/// Stops walking on the first failure.
#[inline]
pub fn all<'arena, F>(ty: Type<'arena>, mut predicate: F) -> bool
where
    F: FnMut(Atom<'arena>) -> bool,
{
    !any(ty, |atom| !predicate(atom))
}

fn any_with<'arena, F>(ty: Type<'arena>, predicate: &mut F) -> bool
where
    F: FnMut(Atom<'arena>) -> bool,
{
    ty.atoms.iter().any(|atom| visit(*atom, predicate))
}

fn visit<'arena, F>(atom: Atom<'arena>, predicate: &mut F) -> bool
where
    F: FnMut(Atom<'arena>) -> bool,
{
    if predicate(atom) {
        return true;
    }

    descend(atom, predicate)
}

fn descend<'arena, F>(atom: Atom<'arena>, predicate: &mut F) -> bool
where
    F: FnMut(Atom<'arena>) -> bool,
{
    match atom {
        Atom::Object(payload) => payload
            .type_arguments
            .is_some_and(|arguments| arguments.iter().any(|argument| any_with(*argument, predicate))),
        Atom::List(payload) => {
            any_with(payload.element_type, predicate)
                || payload
                    .known_elements
                    .is_some_and(|entries| entries.iter().any(|entry| any_with(entry.value, predicate)))
        }
        Atom::Array(payload) => {
            payload.key_param.is_some_and(|key| any_with(key, predicate))
                || payload.value_param.is_some_and(|value| any_with(value, predicate))
                || payload
                    .known_items
                    .is_some_and(|entries| entries.iter().any(|entry| any_with(entry.value, predicate)))
        }
        Atom::Iterable(payload) => any_with(payload.key_type, predicate) || any_with(payload.value_type, predicate),
        Atom::ObjectShape(payload) => {
            payload.known_properties.is_some_and(|entries| entries.iter().any(|entry| any_with(entry.value, predicate)))
        }
        Atom::ClassLikeString(payload) => match payload.specifier {
            ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
                any_with(constraint, predicate)
            }
            ClassLikeStringSpecifier::Any | ClassLikeStringSpecifier::Literal { .. } => false,
        },
        Atom::GenericParameter(payload) => any_with(payload.constraint, predicate),
        Atom::Reference(payload) => payload
            .type_arguments
            .is_some_and(|arguments| arguments.iter().any(|argument| any_with(*argument, predicate))),
        Atom::Conditional(payload) => {
            any_with(payload.subject, predicate)
                || any_with(payload.target, predicate)
                || any_with(payload.then, predicate)
                || any_with(payload.otherwise, predicate)
        }
        Atom::Derived(payload) => match *payload {
            DerivedAtom::KeyOf(target)
            | DerivedAtom::ValueOf(target)
            | DerivedAtom::IntMaskOf(target)
            | DerivedAtom::New(target)
            | DerivedAtom::PropertiesOf { target, .. } => any_with(target, predicate),
            DerivedAtom::IndexAccess { target, index } => any_with(target, predicate) || any_with(index, predicate),
            DerivedAtom::IntMask(members) => members.iter().any(|member| any_with(*member, predicate)),
            DerivedAtom::TemplateType { object, class_name, template_name } => {
                any_with(object, predicate) || any_with(class_name, predicate) || any_with(template_name, predicate)
            }
        },
        Atom::Callable(payload) => {
            let (CallableAtom::Signature(signature) | CallableAtom::Closure(signature)) = payload else {
                return false;
            };

            any_with(signature.return_type, predicate)
                || signature.throws.is_some_and(|throws| any_with(throws, predicate))
                || signature
                    .parameters
                    .is_some_and(|parameters| parameters.iter().any(|parameter| any_with(parameter.r#type, predicate)))
        }
        Atom::Intersected(payload) => {
            visit(*payload.head, predicate) || payload.conjuncts.iter().any(|conjunct| visit(*conjunct, predicate))
        }
        _ => false,
    }
}
