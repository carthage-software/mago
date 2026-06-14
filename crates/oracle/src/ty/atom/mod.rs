use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

use crate::ty::Type;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::alias::AliasAtom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::intersected::IntersectedAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::enumeration::EnumAtom;
use crate::ty::atom::payload::object::has_method::HasMethodAtom;
use crate::ty::atom::payload::object::has_property::HasPropertyAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::reference::GlobalReferenceAtom;
use crate::ty::atom::payload::reference::MemberReferenceAtom;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::resource::ResourceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::float::LiteralFloat;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::variable::VariableAtom;

pub mod kind;
pub mod payload;
pub mod set;

/// A single member of a union [`Type`].
///
/// Variant order is pinned to match [`AtomKind`]'s discriminants: [`Ord`] is
/// kind-first structural order, which is the canonical order of atoms within
/// a union.
///
/// Payloads of sixteen bytes or fewer live inline; larger payloads live
/// behind an `&'arena` reference and are deduplicated by the
/// [`TypeBuilder`](crate::ty::builder::TypeBuilder). Equality and ordering are
/// structural, with a pointer fast path on the reference payloads: within
/// one builder, consing makes pointer identity coincide with structural
/// equality, so the deep comparison only runs for cross-arena values and
/// genuine mismatches.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
pub enum Atom<'arena> {
    Null,
    Never,
    Void,
    Placeholder,
    Mixed(MixedAtom),
    Bool,
    True,
    False,
    Int(IntAtom<'arena>),
    Float(FloatAtom),
    String(&'arena StringAtom<'arena>),
    ClassLikeString(&'arena ClassLikeStringAtom<'arena>),
    Scalar,
    Numeric,
    ArrayKey,
    Object(&'arena ObjectAtom<'arena>),
    Enum(&'arena EnumAtom<'arena>),
    ObjectShape(&'arena ObjectShapeAtom<'arena>),
    HasMethod(HasMethodAtom<'arena>),
    HasProperty(HasPropertyAtom<'arena>),
    Array(&'arena ArrayAtom<'arena>),
    List(&'arena ListAtom<'arena>),
    Iterable(&'arena IterableAtom<'arena>),
    Callable(CallableAtom<'arena>),
    Resource(ResourceAtom),
    GenericParameter(&'arena GenericParameterAtom<'arena>),
    Variable(VariableAtom<'arena>),
    Reference(&'arena SymbolReferenceAtom<'arena>),
    MemberReference(&'arena MemberReferenceAtom<'arena>),
    GlobalReference(&'arena GlobalReferenceAtom<'arena>),
    Alias(&'arena AliasAtom<'arena>),
    Conditional(&'arena ConditionalAtom<'arena>),
    Derived(&'arena DerivedAtom<'arena>),
    ObjectAny,
    Negated(&'arena Type<'arena>),
    Intersected(&'arena IntersectedAtom<'arena>),
}

fn pointer_or_structural_eq<T>(left: &T, right: &T) -> bool
where
    T: PartialEq,
{
    core::ptr::eq(left, right) || left == right
}

impl PartialEq for Atom<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if core::mem::discriminant(self) != core::mem::discriminant(other) {
            return false;
        }

        match (self, other) {
            (Atom::Mixed(left), Atom::Mixed(right)) => left == right,
            (Atom::Int(left), Atom::Int(right)) => left == right,
            (Atom::Float(left), Atom::Float(right)) => left == right,
            (Atom::String(left), Atom::String(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::ClassLikeString(left), Atom::ClassLikeString(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Object(left), Atom::Object(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Enum(left), Atom::Enum(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::ObjectShape(left), Atom::ObjectShape(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::HasMethod(left), Atom::HasMethod(right)) => left == right,
            (Atom::HasProperty(left), Atom::HasProperty(right)) => left == right,
            (Atom::Array(left), Atom::Array(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::List(left), Atom::List(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Iterable(left), Atom::Iterable(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Callable(left), Atom::Callable(right)) => left == right,
            (Atom::Resource(left), Atom::Resource(right)) => left == right,
            (Atom::GenericParameter(left), Atom::GenericParameter(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Variable(left), Atom::Variable(right)) => left == right,
            (Atom::Reference(left), Atom::Reference(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::MemberReference(left), Atom::MemberReference(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::GlobalReference(left), Atom::GlobalReference(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Alias(left), Atom::Alias(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Conditional(left), Atom::Conditional(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Derived(left), Atom::Derived(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Negated(left), Atom::Negated(right)) => pointer_or_structural_eq(*left, *right),
            (Atom::Intersected(left), Atom::Intersected(right)) => pointer_or_structural_eq(*left, *right),
            _ => true,
        }
    }
}

impl<'arena> Atom<'arena> {
    /// `int(value)`: an integer literal atom. Pure value, no interning
    /// involved.
    #[inline]
    #[must_use]
    pub const fn int_literal(value: i64) -> Self {
        Atom::Int(IntAtom::Literal(value))
    }

    /// `float(value)`: a float literal atom. Pure value, no interning
    /// involved.
    #[inline]
    #[must_use]
    pub const fn float_literal(value: f64) -> Self {
        Atom::Float(FloatAtom::Literal(LiteralFloat::new(value)))
    }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> AtomKind {
        match self {
            Atom::Null => AtomKind::Null,
            Atom::Never => AtomKind::Never,
            Atom::Void => AtomKind::Void,
            Atom::Placeholder => AtomKind::Placeholder,
            Atom::Mixed(_) => AtomKind::Mixed,
            Atom::Bool => AtomKind::Bool,
            Atom::True => AtomKind::True,
            Atom::False => AtomKind::False,
            Atom::Int(_) => AtomKind::Int,
            Atom::Float(_) => AtomKind::Float,
            Atom::String(_) => AtomKind::String,
            Atom::ClassLikeString(_) => AtomKind::ClassLikeString,
            Atom::Scalar => AtomKind::Scalar,
            Atom::Numeric => AtomKind::Numeric,
            Atom::ArrayKey => AtomKind::ArrayKey,
            Atom::Object(_) => AtomKind::Object,
            Atom::Enum(_) => AtomKind::Enum,
            Atom::ObjectShape(_) => AtomKind::ObjectShape,
            Atom::HasMethod(_) => AtomKind::HasMethod,
            Atom::HasProperty(_) => AtomKind::HasProperty,
            Atom::Array(_) => AtomKind::Array,
            Atom::List(_) => AtomKind::List,
            Atom::Iterable(_) => AtomKind::Iterable,
            Atom::Callable(_) => AtomKind::Callable,
            Atom::Resource(_) => AtomKind::Resource,
            Atom::GenericParameter(_) => AtomKind::GenericParameter,
            Atom::Variable(_) => AtomKind::Variable,
            Atom::Reference(_) => AtomKind::Reference,
            Atom::MemberReference(_) => AtomKind::MemberReference,
            Atom::GlobalReference(_) => AtomKind::GlobalReference,
            Atom::Alias(_) => AtomKind::Alias,
            Atom::Conditional(_) => AtomKind::Conditional,
            Atom::Derived(_) => AtomKind::Derived,
            Atom::ObjectAny => AtomKind::ObjectAny,
            Atom::Negated(_) => AtomKind::Negated,
            Atom::Intersected(_) => AtomKind::Intersected,
        }
    }

    /// `&` conjuncts this atom intersects with. Empty slice when none.
    #[inline]
    #[must_use]
    pub const fn intersection_types(&self) -> &'arena [Atom<'arena>] {
        match self {
            Atom::Intersected(intersected) => intersected.conjuncts,
            _ => &[],
        }
    }

    /// `true` iff at least one intersection conjunct is present.
    #[inline]
    #[must_use]
    pub const fn has_intersection_types(&self) -> bool {
        !self.intersection_types().is_empty()
    }

    /// `true` iff this kind of atom supports intersections at all.
    #[inline]
    #[must_use]
    pub const fn can_be_intersected(&self) -> bool {
        !matches!(self.kind(), AtomKind::Intersected)
    }

    /// `true` iff the rendered form is large enough to benefit from multiline
    /// pretty formatting when used as a generic parameter.
    #[inline]
    #[must_use]
    pub const fn is_complex(&self) -> bool {
        match self {
            Atom::ObjectShape(_)
            | Atom::Array(_)
            | Atom::List(_)
            | Atom::Callable(_)
            | Atom::Intersected(_)
            | Atom::Iterable(_) => true,
            Atom::Object(object) => object.type_arguments.is_some(),
            Atom::Reference(reference) => reference.type_arguments.is_some(),
            _ => false,
        }
    }
}

impl Display for Atom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Atom::Null => f.write_str("null"),
            Atom::Never => f.write_str("never"),
            Atom::Void => f.write_str("void"),
            Atom::Placeholder => f.write_str("_"),
            Atom::Bool => f.write_str("bool"),
            Atom::True => f.write_str("true"),
            Atom::False => f.write_str("false"),
            Atom::Scalar => f.write_str("scalar"),
            Atom::Numeric => f.write_str("numeric"),
            Atom::ArrayKey => f.write_str("array-key"),
            Atom::ObjectAny => f.write_str("object"),
            Atom::Mixed(payload) => Display::fmt(payload, f),
            Atom::Int(payload) => Display::fmt(payload, f),
            Atom::Float(payload) => Display::fmt(payload, f),
            Atom::String(payload) => Display::fmt(payload, f),
            Atom::ClassLikeString(payload) => Display::fmt(payload, f),
            Atom::Object(payload) => Display::fmt(payload, f),
            Atom::Enum(payload) => Display::fmt(payload, f),
            Atom::ObjectShape(payload) => Display::fmt(payload, f),
            Atom::HasMethod(payload) => Display::fmt(payload, f),
            Atom::HasProperty(payload) => Display::fmt(payload, f),
            Atom::Array(payload) => Display::fmt(payload, f),
            Atom::List(payload) => Display::fmt(payload, f),
            Atom::Iterable(payload) => Display::fmt(payload, f),
            Atom::Callable(payload) => Display::fmt(payload, f),
            Atom::Resource(payload) => Display::fmt(payload, f),
            Atom::GenericParameter(payload) => Display::fmt(payload, f),
            Atom::Variable(payload) => Display::fmt(payload, f),
            Atom::Reference(payload) => Display::fmt(payload, f),
            Atom::MemberReference(payload) => Display::fmt(payload, f),
            Atom::GlobalReference(payload) => Display::fmt(payload, f),
            Atom::Alias(payload) => Display::fmt(payload, f),
            Atom::Conditional(payload) => Display::fmt(payload, f),
            Atom::Derived(payload) => Display::fmt(payload, f),
            Atom::Negated(inner) => {
                let atoms = inner.atoms;
                if atoms.len() == 1 && !atoms[0].has_intersection_types() {
                    write!(f, "!{}", atoms[0])
                } else {
                    write!(f, "!({inner})")
                }
            }
            Atom::Intersected(payload) => Display::fmt(payload, f),
        }
    }
}

impl CopyInto for Atom<'_> {
    type Output<'arena> = Atom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            Atom::Null => Atom::Null,
            Atom::Never => Atom::Never,
            Atom::Void => Atom::Void,
            Atom::Placeholder => Atom::Placeholder,
            Atom::Mixed(payload) => Atom::Mixed(payload),
            Atom::Bool => Atom::Bool,
            Atom::True => Atom::True,
            Atom::False => Atom::False,
            Atom::Int(payload) => Atom::Int(payload.copy_into(arena)),
            Atom::Float(payload) => Atom::Float(payload),
            Atom::String(payload) => Atom::String(copy_ref_into(payload, arena)),
            Atom::ClassLikeString(payload) => Atom::ClassLikeString(copy_ref_into(payload, arena)),
            Atom::Scalar => Atom::Scalar,
            Atom::Numeric => Atom::Numeric,
            Atom::ArrayKey => Atom::ArrayKey,
            Atom::Object(payload) => Atom::Object(copy_ref_into(payload, arena)),
            Atom::Enum(payload) => Atom::Enum(copy_ref_into(payload, arena)),
            Atom::ObjectShape(payload) => Atom::ObjectShape(copy_ref_into(payload, arena)),
            Atom::HasMethod(payload) => Atom::HasMethod(payload.copy_into(arena)),
            Atom::HasProperty(payload) => Atom::HasProperty(payload.copy_into(arena)),
            Atom::Array(payload) => Atom::Array(copy_ref_into(payload, arena)),
            Atom::List(payload) => Atom::List(copy_ref_into(payload, arena)),
            Atom::Iterable(payload) => Atom::Iterable(copy_ref_into(payload, arena)),
            Atom::Callable(payload) => Atom::Callable(payload.copy_into(arena)),
            Atom::Resource(payload) => Atom::Resource(payload),
            Atom::GenericParameter(payload) => Atom::GenericParameter(copy_ref_into(payload, arena)),
            Atom::Variable(payload) => Atom::Variable(payload.copy_into(arena)),
            Atom::Reference(payload) => Atom::Reference(copy_ref_into(payload, arena)),
            Atom::MemberReference(payload) => Atom::MemberReference(copy_ref_into(payload, arena)),
            Atom::GlobalReference(payload) => Atom::GlobalReference(copy_ref_into(payload, arena)),
            Atom::Alias(payload) => Atom::Alias(copy_ref_into(payload, arena)),
            Atom::Conditional(payload) => Atom::Conditional(copy_ref_into(payload, arena)),
            Atom::Derived(payload) => Atom::Derived(copy_ref_into(payload, arena)),
            Atom::ObjectAny => Atom::ObjectAny,
            Atom::Negated(inner) => Atom::Negated(copy_ref_into(inner, arena)),
            Atom::Intersected(payload) => Atom::Intersected(copy_ref_into(payload, arena)),
        }
    }
}
