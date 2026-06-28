//! Object family: `object` (the dominator), named objects (`Foo`),
//! enums and enum cases, object shapes, has-method / has-property
//! narrowings.
//!
//! Implements the nominal subtype check plus type-argument
//! specialisation: for same-class containers, walk type arguments
//! by position with the container's variance; for descendant
//! containers, resolve the inherited arguments via
//! [`SymbolTable::inherited_template_argument`], substitute `child`'s
//! actual arguments through them, and then compare positionally
//! with the container's variance.
//!
//! Intersection types (`Foo&Bar`) follow the Int-L / Int-R rules:
//! container intersections require the input to refine every
//! conjunct; input intersections require some conjunct to refine
//! the container.
//!
//! `static` / `$this` modality is enforced asymmetrically: a
//! container marked `static` (or `$this`) accepts only inputs that
//! are at least as constrained on that flag.
//!
//! Structural narrowings:
//!
//! - `HasMethod(m)`: input is accepted iff it is itself `HasMethod(m)`,
//!   or a `Named(C)` (or descendant) where the symbol table confirms `C`
//!   declares / inherits `m`.
//! - `HasProperty(p)`: symmetric to `HasMethod`.
//! - `ObjectShape{props_out}`: shape-vs-shape uses the same rules as
//!   keyed arrays. Every required-out key must be present (and
//!   required) in the input shape with a refining value, and a sealed
//!   container demands a sealed input. `Named(C)` refines an object
//!   shape iff every required property of the shape is declared on `C`
//!   with a refining declared type, queried via [`SymbolTable::class_property_type`].

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeKind;
use crate::symbol::class_like::r#enum::EnumBackingType;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::callable::CallableAlias;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::object::enumeration::EnumAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::named::ObjectFlag;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::object::shape::ObjectShapeFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::CoercionCause;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::refines as type_refines;
use crate::ty::template::substitute;
use crate::ty::well_known;

/// Container is `object` (`ObjectAny`): accept anything in the object
/// family.
#[inline]
#[must_use]
pub const fn refines_object_any(input: Atom<'_>, _container: Atom<'_>) -> bool {
    is_object_family_kind(input.kind())
}

/// Refinement for `Object | Enum | ObjectShape | HasMethod | HasProperty`
/// containers.
#[inline]
pub fn refines<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    match (input, container) {
        (Atom::Object(input_payload), Atom::Object(container_payload)) => {
            refines_named_named(*input_payload, *container_payload, symbols, options, report, builder)
        }
        (Atom::Enum(input_payload), Atom::Enum(container_payload)) => {
            input_payload.name == container_payload.name && container_payload.case.is_none()
        }
        (Atom::Object(_), Atom::Enum(_)) | (Atom::Enum(_), Atom::Object(_)) => false,
        (_, Atom::HasMethod(container_payload)) => refines_has_method(input, container_payload.method_name, symbols),
        (_, Atom::HasProperty(container_payload)) => {
            refines_has_property(input, container_payload.property_name, symbols)
        }
        (_, Atom::ObjectShape(container_payload)) => {
            refines_object_shape(input, *container_payload, symbols, options, report, builder)
        }
        (Atom::Callable(input_callable), Atom::Object(container_payload)) => {
            input_is_closure_instance(input_callable) && is_closure_class(*container_payload)
        }
        _ => false,
    }
}

/// `true` iff the callable atom is a `\Closure` instance: a closure-flagged
/// signature or a reference to a concrete closure expression. A bare
/// `callable` or an anonymous `callable(...)` signature is not - those can
/// equally be a callable string or `[$object, 'method']` array.
#[inline]
const fn input_is_closure_instance(callable: CallableAtom<'_>) -> bool {
    matches!(callable, CallableAtom::Closure(_) | CallableAtom::Alias(CallableAlias::Closure(_)))
}

/// `true` iff the named object is PHP's `\Closure` class. Matched
/// case-insensitively against the resolved class name, since PHP class
/// names are case-insensitive.
#[inline]
fn is_closure_class(container: ObjectAtom<'_>) -> bool {
    container.name.as_bytes().eq_ignore_ascii_case(b"Closure")
}

#[inline]
fn refines_has_method<'arena, A>(input: Atom<'arena>, method: &'arena [u8], symbols: &SymbolTable<'arena, A>) -> bool
where
    A: Arena,
{
    match input {
        Atom::HasMethod(input_payload) => input_payload.method_name == method,
        Atom::Object(input_payload) => symbols.class_has_method(input_payload.name.id, method),
        Atom::Enum(input_payload) => symbols.class_has_method(input_payload.name.id, method),
        Atom::ObjectShape(_) => false,
        _ => false,
    }
}

#[inline]
fn refines_has_property<'arena, A>(
    input: Atom<'arena>,
    property: &'arena [u8],
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    match input {
        Atom::HasProperty(input_payload) => input_payload.property_name == property,
        Atom::Object(input_payload) => symbols.class_has_property(input_payload.name.id, property),
        Atom::Enum(input_payload) => enum_property_present(input_payload.name, property, symbols),
        Atom::ObjectShape(input_payload) => input_payload
            .known_properties
            .is_some_and(|entries| entries.iter().any(|entry| entry.name == property && !entry.optional)),
        _ => false,
    }
}

/// Built-in enum properties: `name` is always present (any enum case has
/// one); `value` is present only on backed enums.
#[inline]
fn enum_property_present<'arena, A>(
    enum_name: Path<'arena>,
    property: &'arena [u8],
    symbols: &SymbolTable<'arena, A>,
) -> bool
where
    A: Arena,
{
    if property == b"name" {
        return true;
    }

    if property == b"value" {
        return symbols.enum_backing(enum_name.id).is_some();
    }

    false
}

#[inline]
fn refines_object_shape<'arena, S, A>(
    input: Atom<'arena>,
    container: ObjectShapeAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    match input {
        Atom::ObjectShape(input_payload) => {
            shape_refines_shape(*input_payload, container, symbols, options, report, builder)
        }
        Atom::Object(input_payload) => {
            named_refines_shape(input_payload.name, container, symbols, options, report, builder)
        }
        Atom::Enum(input_payload) => match build_enum_shape(*input_payload, symbols, builder) {
            Some(shape) => shape_refines_shape(shape, container, symbols, options, report, builder),
            None => false,
        },
        _ => false,
    }
}

/// Synthesize the structural shape of an enum case: `name` is always a
/// `non-empty-string` (or the literal case name when narrowed to a
/// specific case), and `value` is the backing type for backed enums.
/// The shape is sealed because enum cases expose no other properties.
///
/// Returns `None` when the symbol table doesn't know the enum's backing; the
/// caller treats that as "can't prove refinement" and rejects.
#[inline]
fn build_enum_shape<'arena, S, A>(
    payload: EnumAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<ObjectShapeAtom<'arena>>
where
    S: Arena,
    A: Arena,
{
    if symbols.class_like_kind(payload.name.id) != Some(ClassLikeKind::Enum) {
        return None;
    }

    let name_type = match payload.case {
        Some(case_name) => {
            let literal = builder.string_literal(case_name);

            builder.union_of(&[literal])
        }
        None => builder.union_of(&[well_known::NON_EMPTY_STRING]),
    };

    let mut properties = builder.scratch_vec_with(2);
    properties.push(KnownProperty { name: builder.intern(b"name"), value: name_type, optional: false });
    if let Some(backing) = symbols.enum_backing(payload.name.id) {
        let value_type = builder.union_of(&[match backing {
            EnumBackingType::Int => well_known::INT,
            EnumBackingType::String => well_known::STRING,
        }]);

        properties.push(KnownProperty { name: builder.intern(b"value"), value: value_type, optional: false });
    }

    Some(ObjectShapeAtom {
        known_properties: Some(builder.known_properties(&properties)),
        flags: U8Flags::empty().with(ObjectShapeFlag::Sealed),
    })
}

/// Shape-vs-shape rule , mirroring the keyed-
/// array rule: every required key in the container must be present
/// (required) in the input with a refining value, a sealed container
/// demands a sealed input, and the input may not introduce keys
/// the container does not list when sealed.
#[inline]
fn shape_refines_shape<'arena, S, A>(
    input: ObjectShapeAtom<'arena>,
    container: ObjectShapeAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let input_properties = input.known_properties.unwrap_or_default();
    let container_properties = container.known_properties.unwrap_or_default();

    if container.flags.contains(ObjectShapeFlag::Sealed) && !input.flags.contains(ObjectShapeFlag::Sealed) {
        return false;
    }

    for container_entry in container_properties {
        match input_properties.iter().find(|entry| entry.name == container_entry.name) {
            Some(input_entry) => {
                if !container_entry.optional && input_entry.optional {
                    return false;
                }

                if !type_refines(input_entry.value, container_entry.value, symbols, options, report, builder) {
                    return false;
                }
            }
            None => {
                if !container_entry.optional {
                    return false;
                }
            }
        }
    }

    if container.flags.contains(ObjectShapeFlag::Sealed) {
        for input_entry in input_properties {
            if !container_properties.iter().any(|entry| entry.name == input_entry.name) {
                return false;
            }
        }
    }

    true
}

/// `Named(C) <: object{p1: T1, p2: T2, ...}` iff the symbol table records every
/// required property `pi` on `C` (or an ancestor) with a declared type
/// that refines `Ti`. Optional container properties impose no
/// requirement when missing on `C`.
#[inline]
fn named_refines_shape<'arena, S, A>(
    class: Path<'arena>,
    container: ObjectShapeAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let container_properties = container.known_properties.unwrap_or_default();

    for container_entry in container_properties {
        match symbols.class_property_type(class.id, container_entry.name) {
            Some(declared) => {
                if !type_refines(declared, container_entry.value, symbols, options, report, builder) {
                    return false;
                }
            }
            None => {
                if !container_entry.optional {
                    return false;
                }
            }
        }
    }

    true
}

/// Nominal check plus type-argument specialisation.
///
/// Negated conjuncts in the container's intersections are checked by the
/// standard intersection-conjunct loop before this family is consulted
/// (each conjunct must accept the input), so descendant-exclusion logic
/// falls out for free; input-side negations likewise compose through the
/// input-intersection rule.
///
/// Explicit arguments are normalized to the declared template arity: a
/// class the symbol table declares with no template parameters cannot
/// meaningfully constrain anything via explicit arguments (arity-0
/// reduction), and over-supplied arguments past the declared positions
/// are meaningless and get truncated. This keeps `Foo<int>` and `Foo`,
/// and `Box<int>` and `Box<int, string>` (arity 1), agreeing on refines /
/// overlaps / meet outcomes regardless of how the atoms were constructed.
/// Unsupplied positions are filled with the parameter's upper bound (or
/// `mixed`) and tracked as default-filled so the variance check can skip
/// them.
#[inline]
fn refines_named_named<'arena, S, A>(
    input: ObjectAtom<'arena>,
    container: ObjectAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if !symbols.descends_from(input.name.id, container.name.id) {
        return false;
    }

    if !modality_satisfied(input.flags, container.flags) {
        return false;
    }

    let arity = symbols.template_parameter_arity(container.name.id);
    if arity == 0 {
        // A *known* non-generic class ignores spurious type arguments, so
        // `Foo<int>` is value-equivalent to bare `Foo`. An *unknown* class has
        // no declared variance to consult, so distinct explicit arguments must
        // not be conflated (`Foo<int>` and `Foo<string>` stay separate).
        if symbols.class_like_kind(container.name.id).is_some() {
            return true;
        }

        return input.type_arguments == container.type_arguments;
    }

    let supplied_container_arguments = container.type_arguments.unwrap_or_default();
    let input_actual_arguments = input.type_arguments.unwrap_or_default();
    let same_class = input.name == container.name;

    for position in 0..arity {
        let (container_argument, container_is_default) = match supplied_container_arguments.get(position) {
            Some(argument) => (*argument, false),
            None => (default_template_argument(container.name, position, symbols), true),
        };

        let Some((input_argument, input_is_default)) = input_argument_for_container_position(
            input.name,
            input_actual_arguments,
            container.name,
            position,
            same_class,
            symbols,
            builder,
        ) else {
            return false;
        };

        let variance = symbols
            .template_parameter_at(container.name.id, position)
            .map(|parameter| parameter.variance)
            .unwrap_or_default();

        if !compare_with_variance(
            input_argument,
            input_is_default,
            container_argument,
            container_is_default,
            variance,
            symbols,
            options,
            report,
            builder,
        ) {
            return false;
        }
    }

    true
}

/// Resolve "what does the input pass for the container's template at
/// `position`", free of any remaining references to the input's own
/// templates. The second tuple field is `true` when the position was
/// filled from the parameter's declared default rather than an explicit
/// argument.
///
/// Same-class case: the input's positional argument, or its constraint /
/// `mixed` when no argument was supplied at the use site (partial
/// application).
///
/// Strict-descendant case: query [`SymbolTable::inherited_template_argument`]
/// for the chain-resolved type (in the input's template namespace), then
/// substitute the input's actual arguments into any `GenericParameter`
/// references that name the input's own templates.
#[inline]
fn input_argument_for_container_position<'arena, S, A>(
    input_name: Path<'arena>,
    input_actual_arguments: &[Type<'arena>],
    container_name: Path<'arena>,
    position: usize,
    same_class: bool,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<(Type<'arena>, bool)>
where
    S: Arena,
    A: Arena,
{
    if same_class {
        if let Some(argument) = input_actual_arguments.get(position) {
            return Some((*argument, false));
        }

        return Some((default_template_argument(input_name, position, symbols), true));
    }

    let inherited = resolve_inherited_argument(input_name, container_name, position, symbols, builder, 16)?;
    let input_entity = DefiningEntity::ClassLike(input_name);

    let substituted = substitute(
        inherited,
        &|payload: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
            if payload.defining_entity != input_entity {
                return None;
            }

            let parameter_position = symbols.template_parameter_index(input_name.id, payload.name)?;

            input_actual_arguments.get(parameter_position).copied()
        },
        builder,
    );

    Some((substituted, false))
}

/// Reconstruct the type `input` passes to `container`'s `position`-th parameter
/// when that position is filled purely by *template forwarding* and never
/// materialized as an inherited argument - the transitive case, e.g.
/// `class C<U> extends B<U>` where `class B<T> extends A<T>`, so `C` forwards
/// `U` into `A::T` without `C` writing an explicit `A` argument. The (closed)
/// forwarding relation carries `C::U -> A::T`; we rebuild that
/// `GenericParameter` reference so the caller can substitute `input`'s actual
/// arguments into it.
#[inline]
fn forwarded_template_argument<'arena, S, A>(
    input_name: Path<'arena>,
    container_name: Path<'arena>,
    position: usize,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let container_parameter = symbols.template_parameter_at(container_name.id, position)?;
    let input_symbol = symbols.get_class_like(input_name.id)?;
    let forwarding = input_symbol.forwardings().iter().find(|forwarding| {
        forwarding.target.defining_entity == container_name.id && forwarding.target.name == container_parameter.name
    })?;

    let constraint = input_symbol
        .generics()
        .iter()
        .find(|parameter| parameter.name == forwarding.parameter)
        .map_or(well_known::TYPE_MIXED, |parameter| parameter.constraint);

    let atom = builder.generic_parameter(GenericParameterAtom {
        name: forwarding.parameter,
        defining_entity: DefiningEntity::ClassLike(input_name),
        constraint,
    });

    Some(builder.union_of(&[atom]))
}

/// Resolve the type `input` passes to `container`'s `position`-th parameter,
/// expressed in `input`'s template namespace - direct, forwarded, or resolved
/// transitively through an intermediate ancestor.
///
/// For the transitive case (`class A extends B<...>` where `B extends C<...>`,
/// asking what `A` passes to `C`), it walks to a direct parent that descends
/// from `container`, resolves the parent's `container` argument recursively,
/// then substitutes the parent's own template parameters with the arguments
/// `input` passes to that parent. `depth` bounds the walk so cyclic hierarchies
/// terminate.
pub(crate) fn resolve_inherited_argument<'arena, S, A>(
    input_name: Path<'arena>,
    container_name: Path<'arena>,
    position: usize,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    depth: u8,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    if let Some(argument) = symbols.inherited_template_argument(input_name.id, container_name.id, position) {
        return Some(argument);
    }

    if let Some(argument) = forwarded_template_argument(input_name, container_name, position, symbols, builder) {
        return Some(argument);
    }

    let remaining = depth.checked_sub(1)?;

    let symbol = symbols.get_class_like(input_name.id)?;
    let (superclass, lists) = symbol.inheritance_edges();
    let parents =
        superclass.into_iter().chain(lists[0].edges().iter().copied()).chain(lists[1].edges().iter().copied());

    for edge in parents {
        let parent = edge.target;
        if parent.id == input_name.id || parent.id == container_name.id {
            continue;
        }

        if !symbols.descends_from(parent.id, container_name.id) {
            continue;
        }

        let Some(parent_argument) =
            resolve_inherited_argument(parent, container_name, position, symbols, builder, remaining)
        else {
            continue;
        };

        let resolved = substitute(
            parent_argument,
            &|payload: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
                if payload.defining_entity != DefiningEntity::ClassLike(parent) {
                    return None;
                }

                edge.arguments.iter().find(|argument| argument.parameter == payload.name).map(|argument| argument.ty)
            },
            builder,
        );

        return Some(resolved);
    }

    None
}

/// Compare a single type-argument pair under the container parameter's
/// declared variance. When [`LatticeOptions::template_default_coercion`] is
/// set, a default-filled position on either side bypasses the check and
/// records [`CoercionCause::TemplateDefault`] so the consumer can warn about
/// the unpinned position. With the flag off (the sound default) the
/// default-filled value is compared like any other: a covariant or
/// contravariant default still passes through `mixed` at the top/bottom, but
/// an invariant default must match exactly, so the lattice stays sound.
#[inline]
fn compare_with_variance<'arena, S, A>(
    input: Type<'arena>,
    input_is_default: bool,
    container: Type<'arena>,
    container_is_default: bool,
    variance: Variance,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    if options.template_default_coercion {
        if container_is_default && !matches!(variance, Variance::Contravariant) {
            report.add_cause(CoercionCause::TemplateDefault);
            return true;
        }

        if input_is_default && matches!(variance, Variance::Contravariant) {
            report.add_cause(CoercionCause::TemplateDefault);
            return true;
        }
    }

    match variance {
        Variance::Covariant => type_refines(input, container, symbols, options, report, builder),
        Variance::Contravariant => type_refines(container, input, symbols, options, report, builder),
        Variance::Invariant => {
            type_refines(input, container, symbols, options, report, builder)
                && type_refines(container, input, symbols, options, report, builder)
        }
    }
}

/// The default-fill type-argument for `class`'s template parameter at
/// `position`: its upper bound, or `mixed` when unbounded. Callers track
/// default-filled provenance out of band, since [`Type`] carries no flow
/// flags.
#[inline]
fn default_template_argument<'arena, A>(
    class: Path<'_>,
    position: usize,
    symbols: &SymbolTable<'arena, A>,
) -> Type<'arena>
where
    A: Arena,
{
    symbols
        .template_parameter_at(class.id, position)
        .map(|parameter| parameter.constraint)
        .unwrap_or(well_known::TYPE_MIXED)
}

/// `static<C>` accepts only `static` or `$this`; `$this<C>` accepts only
/// `$this`. A plain `Named(C)` refines neither, because the late-static
/// modality is a stronger guarantee than nominal identity. Inputs more
/// specific than the container's modality are accepted (`$this <: static`).
#[inline]
fn modality_satisfied(input: U8Flags<ObjectFlag>, container: U8Flags<ObjectFlag>) -> bool {
    if container.contains(ObjectFlag::IsThis) && !input.contains(ObjectFlag::IsThis) {
        return false;
    }

    if container.contains(ObjectFlag::IsStatic)
        && !(input.contains(ObjectFlag::IsStatic) || input.contains(ObjectFlag::IsThis))
    {
        return false;
    }

    true
}

#[inline]
pub(crate) const fn is_object_family_kind(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::Object
            | AtomKind::Enum
            | AtomKind::ObjectShape
            | AtomKind::HasMethod
            | AtomKind::HasProperty
            | AtomKind::ObjectAny
    )
}
