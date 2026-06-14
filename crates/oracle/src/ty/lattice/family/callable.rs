//! Callable family: `callable`, `Closure(...)`, anonymous signatures, and
//! known-callable aliases.
//!
//! Within the family:
//!
//! - `callable` (the `Any` variant) accepts any other callable.
//! - `Closure(σ) <: Signature(σ')` when the signatures match (a `Closure`
//!   is a refinement of an anonymous callable).
//! - `Signature(σ) <: Closure(σ')` does NOT hold (the input might be a
//!   non-`\Closure` callable).
//!
//! Cross-family inputs:
//!
//! - A string with the `Callable` refinement flag refines `callable`
//!   (a `callable-string` is a callable name).
//! - `\Closure` named-object refinement is decided by the object family,
//!   not here.
//!
//! Signature comparison is contravariant on
//! parameters and covariant on return: `Sig(P̄_in, R_in)` refines
//! `Sig(P̄_out, R_out)` iff every container parameter at position `i`
//! refines the corresponding input parameter (`P̄_out[i] <: P̄_in[i]`),
//! `R_in <: R_out`, and the input is at least as pure as the container
//! demands. A container with `parameters: None` (an unspecified
//! signature) accepts any signature.

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::ParameterFlag;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::callable::SignatureFlag;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::refines as type_refines;
use crate::world::World;

#[inline]
pub fn refines<'arena, S, A, W>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if let Atom::String(input_payload) = input
        && input_payload.flags.contains(StringRefinementFlag::Callable)
    {
        return true;
    }

    let Atom::Callable(input_payload) = input else {
        return false;
    };

    let Atom::Callable(container_payload) = container else {
        return false;
    };

    match (input_payload, container_payload) {
        (_, CallableAtom::Any) => true,
        (CallableAtom::Any, _) => false,
        (CallableAtom::Signature(input_signature), CallableAtom::Signature(container_signature))
        | (CallableAtom::Closure(input_signature), CallableAtom::Signature(container_signature))
        | (CallableAtom::Closure(input_signature), CallableAtom::Closure(container_signature)) => {
            signature_refines(input_signature, container_signature, world, options, report, builder)
        }
        (CallableAtom::Signature(_), CallableAtom::Closure(_)) => false,
        (CallableAtom::Alias(input_alias), CallableAtom::Alias(container_alias)) => input_alias == container_alias,
        (CallableAtom::Alias(_), _) | (_, CallableAtom::Alias(_)) => false,
    }
}

#[inline]
fn signature_refines<'arena, S, A, W>(
    input_signature: &Signature<'arena>,
    container_signature: &Signature<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if input_signature == container_signature {
        return true;
    }

    if !type_refines(input_signature.return_type, container_signature.return_type, world, options, report, builder) {
        return false;
    }

    if container_signature.flags.contains(SignatureFlag::IsPure)
        && !input_signature.flags.contains(SignatureFlag::IsPure)
    {
        return false;
    }

    if !throws_refines(input_signature.throws, container_signature.throws, world, options, report, builder) {
        return false;
    }

    parameters_refine(input_signature, container_signature, world, options, report, builder)
}

/// Container's `throws` constrains input's: input's exceptions must fit
/// within the container's set. `None` on the container means no
/// constraint; `None` on the input means "throws anything", which is too
/// loose for any constrained container.
#[inline]
fn throws_refines<'arena, S, A, W>(
    input: Option<Type<'arena>>,
    container: Option<Type<'arena>>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    match (input, container) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(input_throws), Some(container_throws)) => {
            type_refines(input_throws, container_throws, world, options, report, builder)
        }
    }
}

#[inline]
fn parameters_refine<'arena, S, A, W>(
    input_signature: &Signature<'arena>,
    container_signature: &Signature<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Some(container_parameters) = container_signature.parameters else {
        return true;
    };
    let Some(input_parameters) = input_signature.parameters else {
        return false;
    };

    let input_required = required_count(input_parameters);
    let container_required = required_count(container_parameters);
    if input_required > container_required {
        return false;
    }

    let input_is_variadic = input_signature.flags.contains(SignatureFlag::IsVariadic);
    let container_is_variadic = container_signature.flags.contains(SignatureFlag::IsVariadic);
    if container_is_variadic && !input_is_variadic {
        return false;
    }

    if !input_is_variadic && input_parameters.len() < container_parameters.len() {
        return false;
    }

    for (index, container_parameter) in container_parameters.iter().enumerate() {
        let input_type = match input_parameters.get(index) {
            Some(parameter) => parameter.r#type,
            None => match input_parameters.last() {
                Some(last) if input_is_variadic => last.r#type,
                _ => return false,
            },
        };

        if !type_refines(container_parameter.r#type, input_type, world, options, report, builder) {
            return false;
        }
    }

    true
}

#[inline]
fn required_count(parameters: &[Parameter<'_>]) -> usize {
    parameters
        .iter()
        .take_while(|parameter| {
            !parameter.flags.contains(ParameterFlag::HasDefault) && !parameter.flags.contains(ParameterFlag::Variadic)
        })
        .count()
}
