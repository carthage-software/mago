//! Array family: keyed arrays (`array<K, V>`, `array{a: int, ...}`,
//! `array{}`) and lists (`list<T>`, `non-empty-list<T>`,
//! `list{0: int, ...}`).
//!
//! The two PHP-level kinds (`Array` and `List`) share most rules. Lists are
//! int-keyed keyed arrays whose values share an element type; the family
//! treats them uniformly where the rules coincide and dispatches to
//! shape-specific helpers where they don't.
//!
//! Implemented rules:
//!
//! - **Reflexivity**: handled by the dispatcher.
//! - **`array{}` (empty)** refines every list and every keyed array (an
//!   empty array fits both views vacuously, except `non-empty` containers).
//! - **List vs list**: element-type covariance; non-empty refines empty-or-
//!   not (`non-empty-list<E> <: list<E>`); sealed-list (`list{...}`)
//!   refines an unsealed list when every known element refines the
//!   container's element type.
//! - **Keyed vs keyed**: key-type and value-type covariance; sealed shapes
//!   refine unsealed keyed-arrays when every known item's key+value refine
//!   the container's parameters; sealed-vs-sealed checks that every
//!   container required key has a matching (refining) input key, and that
//!   the input doesn't have extra required keys.
//! - **Optional-vs-required**: required `<:` optional (the input always
//!   carries the key, the container also accepts that), but optional `not
//!   <:` required (the input might miss the key).
//! - **List vs keyed**: a list refines an unsealed keyed-array if the
//!   container's key parameter accepts `int` and value parameter accepts
//!   the list's element type. Sealed-keyed containers require fixed
//!   entries the list cannot guarantee, so they reject lists.
//! - **Sealed list vs unsealed list**: every known element refines the
//!   element type.
//! - **Sealed list vs sealed list**: pointwise element refinement, with the
//!   same optional-vs-required and extra-required-element rules as keyed
//!   shapes.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::refines as type_refines;
use crate::ty::Type;
use crate::ty::well_known;
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
    match container {
        Atom::List(container_payload) => refines_list(input, container_payload, world, options, report, builder),
        Atom::Array(container_payload) => refines_keyed(input, container_payload, world, options, report, builder),
        _ => false,
    }
}

#[inline]
fn refines_list<'arena, S, A, W>(
    input: Atom<'arena>,
    container: &ListAtom<'arena>,
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
    if input == well_known::EMPTY_ARRAY {
        return !container.flags.contains(ListFlag::NonEmpty);
    }

    match input {
        Atom::List(input_payload) => list_refines_list(input_payload, container, world, options, report, builder),
        _ => false,
    }
}

#[inline]
fn refines_keyed<'arena, S, A, W>(
    input: Atom<'arena>,
    container: &ArrayAtom<'arena>,
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
    if input == well_known::EMPTY_ARRAY {
        return !container.flags.contains(ArrayFlag::NonEmpty);
    }

    match input {
        Atom::Array(input_payload) => keyed_refines_keyed(input_payload, container, world, options, report, builder),
        Atom::List(input_payload) => {
            let (Some(key_param), Some(value_param)) = (container.key_param, container.value_param) else {
                return false;
            };

            if container.flags.contains(ArrayFlag::NonEmpty) && !input_payload.flags.contains(ListFlag::NonEmpty) {
                return false;
            }

            type_refines(well_known::TYPE_INT, key_param, world, options, report, builder)
                && type_refines(input_payload.element_type, value_param, world, options, report, builder)
        }
        _ => false,
    }
}

#[inline]
fn list_refines_list<'arena, S, A, W>(
    input: &ListAtom<'arena>,
    container: &ListAtom<'arena>,
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
    if container.flags.contains(ListFlag::NonEmpty)
        && !input.flags.contains(ListFlag::NonEmpty)
        && !has_required_known_element(input)
    {
        return false;
    }

    let Some(container_elements) = container.known_elements else {
        if let Some(input_elements) = input.known_elements {
            for input_element in input_elements {
                if !type_refines(input_element.value, container.element_type, world, options, report, builder) {
                    return false;
                }
            }
        }

        return type_refines(input.element_type, container.element_type, world, options, report, builder);
    };

    let Some(input_elements) = input.known_elements else {
        return false;
    };

    for input_element in input_elements {
        match container_elements.iter().find(|element| element.index == input_element.index) {
            Some(container_element) => {
                if !type_refines(input_element.value, container_element.value, world, options, report, builder) {
                    return false;
                }
            }
            None => {
                if !type_refines(input_element.value, container.element_type, world, options, report, builder) {
                    return false;
                }
            }
        }
    }

    for container_element in container_elements {
        match input_elements.iter().find(|element| element.index == container_element.index) {
            Some(input_element) => {
                if !container_element.optional && input_element.optional {
                    return false;
                }
            }
            None => {
                if !container_element.optional {
                    return false;
                }
            }
        }
    }

    type_refines(input.element_type, container.element_type, world, options, report, builder)
}

#[inline]
fn has_required_known_element(payload: &ListAtom<'_>) -> bool {
    payload.known_elements.is_some_and(|elements| elements.iter().any(|element| !element.optional))
}

#[inline]
fn keyed_refines_keyed<'arena, S, A, W>(
    input: &ArrayAtom<'arena>,
    container: &ArrayAtom<'arena>,
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
    if container.flags.contains(ArrayFlag::NonEmpty)
        && !input.flags.contains(ArrayFlag::NonEmpty)
        && !has_required_known_item(input)
    {
        return false;
    }

    if container.is_sealed() {
        return sealed_refines_sealed(input, container, world, options, report, builder);
    }

    let (Some(container_key), Some(container_value)) = (container.key_param, container.value_param) else {
        return false;
    };

    if let Some(items) = input.known_items {
        for item in items {
            let key_type = key_to_type(item.key, builder);
            if !type_refines(key_type, container_key, world, options, report, builder) {
                return false;
            }

            if !type_refines(item.value, container_value, world, options, report, builder) {
                return false;
            }
        }
    }

    if let (Some(input_key), Some(input_value)) = (input.key_param, input.value_param) {
        if !type_refines(input_key, container_key, world, options, report, builder) {
            return false;
        }

        if !type_refines(input_value, container_value, world, options, report, builder) {
            return false;
        }
    }

    true
}

#[inline]
fn sealed_refines_sealed<'arena, S, A, W>(
    input: &ArrayAtom<'arena>,
    container: &ArrayAtom<'arena>,
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
    if input.key_param.is_some() || input.value_param.is_some() {
        return false;
    }

    let input_items = input.known_items.unwrap_or_default();
    let container_items = container.known_items.unwrap_or_default();

    for input_item in input_items {
        let in_container = container_items.iter().any(|item| item.key == input_item.key);
        if !in_container && !input_item.optional {
            return false;
        }
    }

    for container_item in container_items {
        match input_items.iter().find(|item| item.key == container_item.key) {
            Some(input_item) => {
                if !container_item.optional && input_item.optional {
                    return false;
                }

                if !type_refines(input_item.value, container_item.value, world, options, report, builder) {
                    return false;
                }
            }
            None => {
                if !container_item.optional {
                    return false;
                }
            }
        }
    }

    true
}

#[inline]
fn has_required_known_item(payload: &ArrayAtom<'_>) -> bool {
    payload.known_items.is_some_and(|items| items.iter().any(|item| !item.optional))
}

#[inline]
fn key_to_type<'arena, S, A>(key: ArrayKey<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    match key {
        ArrayKey::Int(value) => builder.union_of(&[Atom::int_literal(value)]),
        ArrayKey::String(name) => {
            let literal = builder.string_literal(name.as_bytes());

            builder.union_of(&[literal])
        }
        ArrayKey::Const { .. } => well_known::TYPE_ARRAY_KEY,
    }
}
