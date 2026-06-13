//! `Callable` family meet.
//!
//! For two callable signatures with the same parameter arity:
//!
//! - return type is **covariant**; a value satisfying both must
//!   produce a value compatible with both, so the meet narrows the
//!   return type via [`crate::ty::meet::compute`].
//! - parameter types are **contravariant**; both signatures must
//!   accept any input either accepts, so the meet *widens* each
//!   parameter via [`crate::ty::join::compute`].
//! - purity is conjunctive (`pure ∧ pure → pure`, otherwise impure).
//!
//! When either side carries no signature (the `Any` variant) the
//! subsumption rule has already accepted the more-specific side, so
//! this function never sees that case.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::callable::SignatureFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::world::World;

pub(in crate::ty::meet) fn callable_meet<'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let (Atom::Callable(CallableAtom::Signature(a_signature)), Atom::Callable(CallableAtom::Signature(b_signature))) =
        (a, b)
    else {
        return None;
    };

    let a_parameters: &[Parameter<'arena>] = a_signature.parameters.unwrap_or(&[]);
    let b_parameters: &[Parameter<'arena>] = b_signature.parameters.unwrap_or(&[]);
    if a_parameters.len() != b_parameters.len() {
        return None;
    }

    let mut merged_parameters: Vec<Parameter<'arena>> = Vec::with_capacity(a_parameters.len());
    for (a_parameter, b_parameter) in a_parameters.iter().zip(b_parameters.iter()) {
        let mut combined: Vec<Atom<'arena>> = a_parameter.r#type.atoms.to_vec();
        combined.extend_from_slice(b_parameter.r#type.atoms);
        let widened = crate::ty::join::compute(&combined, builder);
        let r#type = builder.union_of(&widened);

        merged_parameters.push(Parameter { name: a_parameter.name, r#type, flags: a_parameter.flags });
    }

    let return_type =
        crate::ty::meet::compute(a_signature.return_type, b_signature.return_type, world, options, report, builder);

    let throws = match (a_signature.throws, b_signature.throws) {
        (Some(a_throws), Some(b_throws)) => {
            Some(crate::ty::meet::compute(a_throws, b_throws, world, options, report, builder))
        }
        (Some(throws), None) | (None, Some(throws)) => Some(throws),
        (None, None) => None,
    };

    let pure = a_signature.flags.contains(SignatureFlag::IsPure) && b_signature.flags.contains(SignatureFlag::IsPure);
    let variadic =
        a_signature.flags.contains(SignatureFlag::IsVariadic) && b_signature.flags.contains(SignatureFlag::IsVariadic);
    let mut flags = U8Flags::empty();
    flags.set_value(SignatureFlag::IsPure, pure);
    flags.set_value(SignatureFlag::IsVariadic, variadic);

    let parameters = if merged_parameters.is_empty() { None } else { Some(builder.parameters(&merged_parameters)) };
    let signature = builder.signature(Signature { parameters, return_type, throws, flags });

    Some(Atom::Callable(CallableAtom::Signature(signature)))
}
