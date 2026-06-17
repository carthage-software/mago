//! Type expansion: resolve non-structural type forms (`Alias`,
//! `Reference`, `Derived`, `Conditional`, contextual keywords) into
//! their structural definitions.
//!
//! [`expand`] is the no-context entry point for callers that just want
//! `Alias` / `Reference` / `Derived` resolution. [`expand_with`] takes
//! an explicit [`ExpansionContext`] and additionally substitutes
//! contextual keywords (`self`, `static`, `parent`) and evaluates
//! `Conditional` atoms when `evaluate_conditional` is on.
//!
//! # Stages
//!
//! - **Stage 1:** `Alias` resolution.
//! - **Stage 2:** `Reference` resolution (`SymbolReference`,
//!   `MemberReference`, `GlobalReference`).
//! - **Stage 3:** `Derived` evaluation (`KeyOf`, `ValueOf`,
//!   `IndexAccess`, `IntMask`, `IntMaskOf`, `TemplateType`,
//!   `PropertiesOf`, `New`).
//! - **Stage 4:** Contextual keyword substitution (`self`, `static`,
//!   `parent`, `$this`) and `Conditional` evaluation.
//!
//! # Structural descent
//!
//! Expansion descends into every nested type: `Object`
//! type arguments, list / keyed-array / iterable element types, sealed
//! known items, class-like-string constraints, generic-parameter
//! constraints, conditional / derived / callable operands. The walk
//! is delegated to [`crate::transform::flat_map`]; this module owns
//! only the per-atom resolution logic.

use std::collections::BTreeSet;

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::alias::AliasAtom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::named::ObjectFlag;
use crate::ty::atom::payload::reference::GlobalReferenceAtom;
use crate::ty::atom::payload::reference::MemberReferenceAtom;
use crate::ty::atom::payload::reference::NameSelector;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::overlaps;
use crate::ty::lattice::refines;
use crate::ty::transform;
use crate::ty::well_known;
use crate::world::World;

mod context;

pub use self::context::ExpansionContext;

/// Resolve every expandable atom inside `ty` against `world`, with the
/// default expansion context (no contextual class names, conditionals
/// preserved).
#[inline]
pub fn expand<'arena, S, A, W>(ty: Type<'arena>, world: &W, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    expand_with(ty, world, &ExpansionContext::default(), builder)
}

/// Like [`expand`] but with a caller-supplied [`ExpansionContext`].
/// Returns the same [`Type`] when nothing changed.
#[inline]
pub fn expand_with<'arena, S, A, W>(
    ty: Type<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    transform::flat_map_with_builder(ty, |atom, builder| resolve_atom(atom, world, context, builder), builder)
}

/// Per-atom resolution. By the time this fires, [`crate::transform`]
/// has already recursively walked every nested [`Type`] carried in the
/// atom's payload; the closure receives an atom whose children are
/// fully expanded.
#[inline]
fn resolve_atom<'arena, S, A, W>(
    atom: Atom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    match atom {
        Atom::Alias(payload) => resolve_alias(atom, payload, world, context, builder),
        Atom::Reference(payload) => resolve_reference(payload, world, context, builder),
        Atom::MemberReference(payload) => resolve_member_reference(atom, payload, world, context, builder),
        Atom::GlobalReference(payload) => resolve_global_reference(atom, payload, world, context, builder),
        Atom::Derived(payload) => resolve_derived(atom, payload, world, context, builder),
        Atom::Conditional(payload) => resolve_conditional(atom, payload, world, context, builder),
        Atom::Object(payload) => resolve_object(atom, payload, world, context, builder),
        Atom::GenericParameter(payload) => resolve_generic_parameter(atom, payload, context),
        _ => vec![atom],
    }
}

#[inline]
fn resolve_alias<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &AliasAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if !context.evaluate_aliases {
        return vec![atom];
    }

    let Some(body) = world.alias_body(payload.class_name.id, payload.alias_name) else {
        return vec![atom];
    };

    expand_with(body, world, context, builder).atoms.to_vec()
}

/// `SymbolReference("Foo", type_arguments)` is, semantically, the same
/// value-set as `Object("Foo", ...)`. Convert it; the type arguments
/// have already been walked by the surrounding [`crate::transform`]
/// call, so we just reuse them. Contextual keyword substitution applies
/// (a `self` / `static` / `parent` reference picks up the corresponding
/// context entry).
#[inline]
fn resolve_reference<'arena, S, A, W>(
    payload: &SymbolReferenceAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let resolved_name = resolve_keyword_name(payload.name, U8Flags::empty(), context).unwrap_or(payload.name);
    let mut object =
        ObjectAtom { name: resolved_name, type_arguments: payload.type_arguments, flags: U8Flags::empty() };

    if context.fill_template_defaults && object.type_arguments.is_none() {
        let arity = world.template_parameter_arity(object.name.id);
        if arity > 0 {
            let filled: Vec<Type<'arena>> = (0..arity)
                .map(|position| {
                    world
                        .template_parameter_at(object.name.id, position)
                        .and_then(|parameter| parameter.upper_bound)
                        .unwrap_or(well_known::TYPE_MIXED)
                })
                .collect();
            object.type_arguments = Some(builder.types(&filled));
        }
    }

    vec![builder.object(object)]
}

/// Replace a free `GenericParameter` atom with its constraint. Gated
/// on [`ExpansionContext::substitute_template_constraints`]; when off,
/// the atom passes through (the common case - comparing two template
/// parameters for identity must see them as opaque).
#[inline]
fn resolve_generic_parameter<'arena>(
    atom: Atom<'arena>,
    payload: &GenericParameterAtom<'arena>,
    context: &ExpansionContext<'arena>,
) -> Vec<Atom<'arena>> {
    if !context.substitute_template_constraints {
        return vec![atom];
    }

    payload.constraint.atoms.to_vec()
}

/// Resolve a class-like constant reference.
///
/// `Foo::CONST` (an `Identifier` selector) resolves to the constant's declared
/// type via [`World::class_constant_type`], recursively expanded.
///
/// A wildcard / prefix / suffix selector (`Foo::*`, `Foo::STATUS_*`,
/// `Foo::*_FLAG`, `Foo::*PART*`) resolves to the union of every constant whose
/// name matches the selector - and, when `Foo` is an enum, every matching case
/// as well. Each matched constant body is itself expanded; each matched enum
/// case contributes its singleton case type. When nothing matches (or the
/// world knows no constants for the class-like), the reference is preserved
/// unchanged.
#[inline]
fn resolve_member_reference<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &MemberReferenceAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if !context.evaluate_class_constants {
        return vec![atom];
    }

    let class = payload.class_like_name;

    if let NameSelector::Identifier(constant) = payload.selector {
        let Some(body) = world.class_constant_type(class.id, constant) else {
            return vec![atom];
        };

        return expand_with(body, world, context, builder).atoms.to_vec();
    }

    let selector = payload.selector;

    let mut constant_bodies: Vec<Type<'arena>> = Vec::new();
    for constant in world.class_constants(class.id) {
        if selector_matches(selector, constant.name.as_bytes())
            && let Some(body) = constant.ty.effective()
        {
            constant_bodies.push(body);
        }
    }

    let mut case_names: Vec<&'arena [u8]> = Vec::new();
    if world.class_like_kind(class.id) == Some(ClassLikeKind::Enum) {
        for case in world.enum_cases(class.id) {
            let name = case.name.as_bytes();
            if selector_matches(selector, name) {
                case_names.push(name);
            }
        }
    }

    if constant_bodies.is_empty() && case_names.is_empty() {
        return vec![atom];
    }

    let mut resolved: Vec<Atom<'arena>> = Vec::new();
    for body in constant_bodies {
        let expanded = expand_with(body, world, context, builder);
        resolved.extend_from_slice(expanded.atoms);
    }

    for case in case_names {
        resolved.push(builder.enum_case(class.as_bytes(), case));
    }

    resolved
}

/// Whether `name` is picked by `selector`. `Identifier` is an exact match;
/// `StartsWith` / `EndsWith` / `Contains` are byte-substring tests; `Wildcard`
/// matches everything.
#[inline]
fn selector_matches(selector: NameSelector<'_>, name: &[u8]) -> bool {
    let bytes = name;
    match selector {
        NameSelector::Identifier(target) => bytes == target,
        NameSelector::StartsWith(prefix) => bytes.starts_with(prefix),
        NameSelector::EndsWith(suffix) => bytes.ends_with(suffix),
        NameSelector::Contains(needle) => contains_subslice(bytes, needle),
        NameSelector::Wildcard => true,
    }
}

/// `true` iff `needle` occurs as a contiguous subslice of `haystack`. An empty
/// needle matches anything.
#[inline]
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    if needle.len() > haystack.len() {
        return false;
    }

    haystack.windows(needle.len()).any(|window| window == needle)
}

/// A global constant reference resolves through
/// [`World::global_constant_type`]. Wildcard selectors pass through.
#[inline]
fn resolve_global_reference<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &GlobalReferenceAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if !context.evaluate_global_constants {
        return vec![atom];
    }

    let NameSelector::Identifier(name) = payload.selector else {
        return vec![atom];
    };

    let Some(body) = world.global_constant_type(SymbolId::constant(name)) else {
        return vec![atom];
    };

    expand_with(body, world, context, builder).atoms.to_vec()
}

#[inline]
fn resolve_derived<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &DerivedAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let evaluated = match *payload {
        DerivedAtom::KeyOf(target) => Some(evaluate_key_of(target, builder)),
        DerivedAtom::ValueOf(target) => Some(evaluate_value_of(target, builder)),
        DerivedAtom::IndexAccess { target, index } => Some(evaluate_index_access(target, index, builder)),
        DerivedAtom::IntMask(members) => Some(evaluate_int_mask(members, builder)),
        DerivedAtom::IntMaskOf(target) => Some(evaluate_int_mask_of(target, builder)),
        DerivedAtom::TemplateType { object, class_name, template_name } => {
            evaluate_template_type(object, class_name, template_name, world, context, builder)
        }
        DerivedAtom::PropertiesOf { target, visibility } => evaluate_properties_of(target, visibility, world, builder),
        DerivedAtom::New(target) => evaluate_new(target, world, context, builder),
    };

    match evaluated {
        Some(ty) => ty.atoms.to_vec(),
        None => vec![atom],
    }
}

/// Conditional `T is U ? A : B` (or its negated form).
///
/// When [`ExpansionContext::evaluate_conditional`] is on, the test
/// `subject <: target` is decided via the lattice. A subtype hit picks
/// the then-branch (or the otherwise-branch when negated); a disjoint
/// pair picks the other side; an undecidable test widens to the union
/// of both branches.
///
/// When [`ExpansionContext::evaluate_conditional`] is off, the atom is
/// preserved unchanged - its operands have already been walked by the
/// enclosing transform call.
#[inline]
fn resolve_conditional<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &ConditionalAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if !context.evaluate_conditional {
        return vec![atom];
    }

    let mut report = LatticeReport::new();
    let options = LatticeOptions::default();
    let test_passes = refines(payload.subject, payload.target, world, options, &mut report, builder);
    let test_disjoint = !overlaps(payload.subject, payload.target, world, options, &mut report, builder);

    let (chosen_then, chosen_otherwise) =
        if payload.negated { (payload.otherwise, payload.then) } else { (payload.then, payload.otherwise) };

    let result = if test_passes {
        chosen_then
    } else if test_disjoint {
        chosen_otherwise
    } else {
        let mut atoms: Vec<Atom<'arena>> = Vec::new();
        atoms.extend_from_slice(chosen_then.atoms);
        atoms.extend_from_slice(chosen_otherwise.atoms);
        builder.union_of(&atoms)
    };

    result.atoms.to_vec()
}

/// Resolve a named-object atom. Combines three independent stages:
///
/// - Contextual keyword substitution (`self` / `static` / `parent` /
///   `$this`) when the corresponding [`ExpansionContext`] entry is set.
/// - Final-function collapse: with [`ExpansionContext::function_is_final`],
///   drop the `IsStatic` / `IsThis` modality flags on a named class
///   that already has a concrete name (no `static_class` binding
///   needed).
/// - Default-fill of unfilled generic positions when
///   [`ExpansionContext::fill_template_defaults`] is on.
#[inline]
fn resolve_object<'arena, S, A, W>(
    atom: Atom<'arena>,
    payload: &ObjectAtom<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let mut object = *payload;
    let mut changed = false;

    if let Some(class) = resolve_keyword_name(object.name, object.flags, context) {
        object.name = class;
        object.flags.unset(ObjectFlag::IsStatic);
        object.flags.unset(ObjectFlag::IsThis);
        changed = true;
    } else if context.function_is_final
        && (object.flags.contains(ObjectFlag::IsStatic) || object.flags.contains(ObjectFlag::IsThis))
    {
        object.flags.unset(ObjectFlag::IsStatic);
        object.flags.unset(ObjectFlag::IsThis);
        changed = true;
    }

    if context.fill_template_defaults && object.type_arguments.is_none() {
        let arity = world.template_parameter_arity(object.name.id);
        if arity > 0 {
            let filled: Vec<Type<'arena>> = (0..arity)
                .map(|position| {
                    world
                        .template_parameter_at(object.name.id, position)
                        .and_then(|parameter| parameter.upper_bound)
                        .unwrap_or(well_known::TYPE_MIXED)
                })
                .collect();
            object.type_arguments = Some(builder.types(&filled));
            changed = true;
        }
    }

    if changed { vec![builder.object(object)] } else { vec![atom] }
}

/// Map `self` / `static` / `parent` / the `IsStatic` / `IsThis`
/// modality flags to a concrete class name pulled from `context`.
/// Returns `None` when no keyword applies (the atom is a plain
/// `Named(C)`) or when the context lacks the required entry.
#[inline]
fn resolve_keyword_name<'arena>(
    name: Path<'arena>,
    flags: U8Flags<ObjectFlag>,
    context: &ExpansionContext<'arena>,
) -> Option<Path<'arena>> {
    let bytes = name.as_bytes();
    if flags.contains(ObjectFlag::IsThis) || flags.contains(ObjectFlag::IsStatic) || bytes == b"static" {
        context.static_class
    } else if bytes == b"self" {
        context.self_class
    } else if bytes == b"parent" {
        context.parent_class
    } else {
        None
    }
}

/// `key-of<τ>`: keys admissible by `τ`. Operand has
/// already been expanded by the surrounding walk.
#[inline]
fn evaluate_key_of<'arena, S, A>(target: Type<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let [only] = target.atoms else {
        return well_known::TYPE_MIXED;
    };

    match *only {
        Atom::List(payload) => {
            let mut keys: Vec<Atom<'arena>> = Vec::new();
            if let Some(entries) = payload.known_elements {
                for entry in entries {
                    keys.push(Atom::int_literal(i64::from(entry.index)));
                }
            }

            match payload.known_count {
                Some(known_count) => {
                    let count = std::num::NonZeroI64::from(known_count).get();
                    keys.push(builder.int_range(Some(0), Some(count - 1)));
                }
                None => {
                    keys.push(well_known::NON_NEGATIVE_INT);
                }
            }

            builder.union_of(&keys)
        }
        Atom::Array(payload) => {
            let mut keys: Vec<Atom<'arena>> = Vec::new();
            if let Some(entries) = payload.known_items {
                for entry in entries {
                    if let Some(key) = array_key_to_atom(entry.key, builder) {
                        keys.push(key);
                    }
                }
            }

            if let Some(key_param) = payload.key_param {
                keys.extend_from_slice(key_param.atoms);
            }

            if keys.is_empty() { well_known::TYPE_MIXED } else { builder.union_of(&keys) }
        }
        Atom::Iterable(payload) => payload.key_type,
        Atom::ObjectShape(payload) => {
            let Some(entries) = payload.known_properties else {
                return well_known::TYPE_NEVER;
            };

            let keys: Vec<Atom<'arena>> = entries.iter().map(|entry| builder.string_literal(entry.name)).collect();

            if keys.is_empty() { well_known::TYPE_NEVER } else { builder.union_of(&keys) }
        }
        _ => well_known::TYPE_MIXED,
    }
}

/// `value-of<τ>`: values admissible by `τ`.
#[inline]
fn evaluate_value_of<'arena, S, A>(target: Type<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let [only] = target.atoms else {
        return well_known::TYPE_MIXED;
    };

    match *only {
        Atom::List(payload) => {
            let mut values: Vec<Atom<'arena>> = Vec::new();
            if let Some(entries) = payload.known_elements {
                for entry in entries {
                    values.extend_from_slice(entry.value.atoms);
                }
            }

            values.extend_from_slice(payload.element_type.atoms);

            builder.union_of(&values)
        }
        Atom::Array(payload) => {
            let mut values: Vec<Atom<'arena>> = Vec::new();
            if let Some(entries) = payload.known_items {
                for entry in entries {
                    values.extend_from_slice(entry.value.atoms);
                }
            }

            if let Some(value_param) = payload.value_param {
                values.extend_from_slice(value_param.atoms);
            }

            if values.is_empty() { well_known::TYPE_MIXED } else { builder.union_of(&values) }
        }
        Atom::Iterable(payload) => payload.value_type,
        Atom::ObjectShape(payload) => {
            let Some(entries) = payload.known_properties else {
                return well_known::TYPE_NEVER;
            };

            let mut values: Vec<Atom<'arena>> = Vec::new();
            for entry in entries {
                values.extend_from_slice(entry.value.atoms);
            }

            if values.is_empty() { well_known::TYPE_NEVER } else { builder.union_of(&values) }
        }
        _ => well_known::TYPE_MIXED,
    }
}

/// `τ[κ]`.
#[inline]
fn evaluate_index_access<'arena, S, A>(
    target: Type<'arena>,
    index: Type<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let ([only], [key_atom]) = (target.atoms, index.atoms) else {
        return well_known::TYPE_MIXED;
    };

    match *only {
        Atom::Array(payload) => {
            if let Some(entries) = payload.known_items
                && let Some(literal_key) = atom_to_array_key(*key_atom)
            {
                for entry in entries {
                    if entry.key == literal_key {
                        return entry.value;
                    }
                }
            }

            payload.value_param.unwrap_or(well_known::TYPE_NEVER)
        }
        Atom::List(payload) => {
            if let Some(index_value) = literal_int(*key_atom)
                && index_value >= 0
                && let Some(entries) = payload.known_elements
            {
                for entry in entries {
                    if i64::from(entry.index) == index_value {
                        return entry.value;
                    }
                }
            }

            payload.element_type
        }
        Atom::Iterable(payload) => payload.value_type,
        Atom::ObjectShape(payload) => {
            let Some(entries) = payload.known_properties else {
                return well_known::TYPE_NEVER;
            };

            if let Some(literal) = string_literal_value(*key_atom) {
                for entry in entries {
                    if entry.name == literal {
                        return entry.value;
                    }
                }

                return well_known::TYPE_NEVER;
            }

            let mut values: Vec<Atom<'arena>> = Vec::new();
            for entry in entries {
                values.extend_from_slice(entry.value.atoms);
            }

            if values.is_empty() { well_known::TYPE_NEVER } else { builder.union_of(&values) }
        }
        _ => well_known::TYPE_MIXED,
    }
}

#[inline]
fn string_literal_value(atom: Atom<'_>) -> Option<&'_ [u8]> {
    let Atom::String(payload) = atom else {
        return None;
    };

    match payload.literal {
        StringLiteral::Value(value) => Some(value),
        _ => None,
    }
}

#[inline]
fn evaluate_int_mask<'arena, S, A>(
    members: &[Type<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut literals: Vec<i64> = Vec::with_capacity(members.len());
    for member in members {
        let [only] = member.atoms else {
            return well_known::TYPE_MIXED;
        };

        match literal_int(*only) {
            Some(value) => literals.push(value),
            None => return well_known::TYPE_MIXED,
        }
    }

    int_mask_union(&literals, builder)
}

#[inline]
fn evaluate_int_mask_of<'arena, S, A>(target: Type<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut literals: Vec<i64> = Vec::new();
    for atom in target.atoms {
        match literal_int(*atom) {
            Some(value) => literals.push(value),
            None => return well_known::TYPE_MIXED,
        }
    }

    int_mask_union(&literals, builder)
}

#[inline]
fn int_mask_union<'arena, S, A>(literals: &[i64], builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let count = literals.len();
    if count == 0 {
        return builder.union_of(&[Atom::int_literal(0)]);
    }

    if count > 16 {
        return well_known::TYPE_INT;
    }

    let total = 1u32 << count;
    let mut values: BTreeSet<i64> = BTreeSet::new();
    for mask in 0..total {
        let mut combined: i64 = 0;
        for (bit, &literal) in literals.iter().enumerate() {
            if (mask >> bit) & 1 == 1 {
                combined |= literal;
            }
        }

        values.insert(combined);
    }

    let atoms: Vec<Atom<'arena>> = values.into_iter().map(Atom::int_literal).collect();

    builder.union_of(&atoms)
}

/// `template-type<$object, ClassName, T>`: the type bound to template `T` of
/// `ClassName` for the value `$object`.
///
/// When `$object`'s type carries a concrete binding for `T` - either directly
/// (`$object: ClassName<int>`) or through inheritance (`$object: Sub<int>`
/// where `Sub extends ClassName<int>`) - that binding is the result. Otherwise
/// the declared upper bound of `T` (or `mixed` when unbounded) stands in as the
/// widest sound type. Returns `None` (the atom passes through) only when the
/// world knows nothing about `ClassName` or `T`.
#[inline]
fn evaluate_template_type<'arena, S, A, W>(
    object: Type<'arena>,
    class_name: Type<'arena>,
    template_name: Type<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let class = single_object_or_reference_name(class_name)?;
    let template = single_string_literal_value(template_name)?;
    let position = world.template_parameter_index(class.id, template)?;

    if let Some(binding) = template_binding_from_object(object, class, position, world) {
        return Some(expand_with(binding, world, context, builder));
    }

    let parameter = world.template_parameter_at(class.id, position)?;

    Some(expand_with(parameter.upper_bound.unwrap_or(well_known::TYPE_MIXED), world, context, builder))
}

/// Read the type `object` binds to `class`'s template at `position`, if any.
///
/// `object` is the already-expanded type of the value operand. A single object
/// atom that *is* `class` exposes the binding through its own type arguments; a
/// single object atom that *descends from* `class` exposes it through the
/// inheritance edge ([`World::inherited_template_argument`]). Anything else -
/// a union, a non-object, a raw `class` with no arguments - yields `None`.
#[inline]
fn template_binding_from_object<'arena, W>(
    object: Type<'arena>,
    class: Path<'_>,
    position: usize,
    world: &W,
) -> Option<Type<'arena>>
where
    W: World<'arena>,
{
    let [only] = object.atoms else {
        return None;
    };

    let (object_name, type_arguments) = match only {
        Atom::Object(payload) => (payload.name, payload.type_arguments),
        Atom::Reference(payload) => (payload.name, payload.type_arguments),
        _ => return None,
    };

    if object_name.as_bytes() == class.as_bytes() {
        return type_arguments.and_then(|arguments| arguments.get(position).copied());
    }

    if world.descends_from(object_name.id, class.id) {
        return world.inherited_template_argument(object_name.id, class.id, position);
    }

    None
}

/// `properties-of<C>`: enumerate `C`'s declared
/// properties and produce a sealed `array{name: type, ...}` shape.
/// `visibility` filters the enumeration; `None` keeps every visible
/// property.
#[inline]
fn evaluate_properties_of<'arena, S, A, W>(
    target: Type<'arena>,
    visibility: Option<Visibility>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let class = single_object_or_reference_name(target)?;

    let count = world.class_property_count(class.id);
    let mut entries: Vec<KnownItem<'arena>> = Vec::with_capacity(count);
    for position in 0..count {
        let Some(property) = world.class_property_at(class.id, position) else {
            continue;
        };

        if let Some(required) = visibility
            && property.visibility != required
        {
            continue;
        }

        entries.push(KnownItem { key: ArrayKey::String(property.name), value: property.r#type, optional: false });
    }

    entries.sort_by_key(|entry| entry.key);

    let known_items = Some(builder.known_items(&entries));
    let shape = builder.array(ArrayAtom { key_param: None, value_param: None, known_items, flags: U8Flags::empty() });

    Some(builder.union_of(&[shape]))
}

/// `new<C>`: the type produced by `new C(...)`.
///
/// The operand has already been expanded by the surrounding walk. The result
/// depends on what `C` resolves to:
///
/// - `Object(C)` / `Reference(C)` / a literal `class-string<Foo>` instantiate
///   the named class, filling any templates `C` declares with their upper
///   bound (or `mixed` when unbounded) - the nominal class with its widest
///   sound type arguments.
/// - A bounded or generic class-string (`class-string<T>`, `class-string<T of
///   Foo>`) instantiates the *constraint*: `new` on a value of type
///   `class-string<T>` produces a `T`, so the constraint type is the result.
/// - An unconstrained `class-string` produces `object` - some instance, but of
///   no statically known class.
///
/// `new<C>` carries no constructor arguments, so this is the most precise
/// result obtainable from the type alone; argument-driven inference belongs to
/// the call-site analyzer, not type expansion.
#[inline]
fn evaluate_new<'arena, S, A, W>(
    target: Type<'arena>,
    world: &W,
    context: &ExpansionContext<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let [only] = target.atoms else {
        return None;
    };

    match only {
        Atom::Object(payload) => Some(instantiate_named_class(payload.name, world, builder)),
        Atom::Reference(payload) => Some(instantiate_named_class(payload.name, world, builder)),
        Atom::ClassLikeString(payload) => match payload.specifier {
            ClassLikeStringSpecifier::Literal { value } => Some(instantiate_named_class(value, world, builder)),
            ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
                Some(expand_with(constraint, world, context, builder))
            }
            ClassLikeStringSpecifier::Any => Some(well_known::TYPE_OBJECT),
        },
        _ => None,
    }
}

/// Build the instance type of `class`, filling each template position `class`
/// declares with its upper bound (or `mixed` when unbounded).
#[inline]
fn instantiate_named_class<'arena, S, A, W>(
    class: Path<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let arity = world.template_parameter_arity(class.id);
    let object = if arity == 0 {
        ObjectAtom { name: class, type_arguments: None, flags: U8Flags::empty() }
    } else {
        let arguments: Vec<Type<'arena>> = (0..arity)
            .map(|position| {
                world
                    .template_parameter_at(class.id, position)
                    .and_then(|parameter| parameter.upper_bound)
                    .unwrap_or(well_known::TYPE_MIXED)
            })
            .collect();

        ObjectAtom { name: class, type_arguments: Some(builder.types(&arguments)), flags: U8Flags::empty() }
    };
    let instance = builder.object(object);

    builder.union_of(&[instance])
}

#[inline]
fn single_object_or_reference_name(ty: Type<'_>) -> Option<Path<'_>> {
    let [only] = ty.atoms else {
        return None;
    };

    match only {
        Atom::Object(payload) => Some(payload.name),
        Atom::Reference(payload) => Some(payload.name),
        _ => None,
    }
}

#[inline]
fn single_string_literal_value(ty: Type<'_>) -> Option<&'_ [u8]> {
    let [only] = ty.atoms else {
        return None;
    };

    string_literal_value(*only)
}

#[inline]
fn array_key_to_atom<'arena, S, A>(
    key: ArrayKey<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    match key {
        ArrayKey::Int(value) => Some(Atom::int_literal(value)),
        ArrayKey::String(name) => Some(builder.string_literal(name)),
        ArrayKey::Const { .. } => None,
    }
}

#[inline]
fn atom_to_array_key(atom: Atom<'_>) -> Option<ArrayKey<'_>> {
    match atom {
        Atom::Int(IntAtom::Literal(value)) => Some(ArrayKey::Int(value)),
        Atom::String(payload) => match payload.literal {
            StringLiteral::Value(value) => Some(ArrayKey::String(value)),
            _ => None,
        },
        _ => None,
    }
}

#[inline]
fn literal_int(atom: Atom<'_>) -> Option<i64> {
    match atom {
        Atom::Int(IntAtom::Literal(value)) => Some(value),
        _ => None,
    }
}
