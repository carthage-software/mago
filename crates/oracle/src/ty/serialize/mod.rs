//! Structural serialization for [`Type`].
//!
//! A `Type<'arena>` borrows its atom payloads from the arena that built it;
//! its addresses are meaningless outside that arena (and across processes).
//! This module instead produces a [`SerializableType`]; a self-contained,
//! owned structural representation that can round-trip through any byte
//! format.
//!
//! # Use it manually
//!
//! ```ignore
//! let serial = my_type.to_serializable();
//! // ... persist `serial` somewhere ...
//! let restored: Type<'_> = serial.build(&mut builder);
//! ```
//!
//! # Serde
//!
//! With the `serde` Cargo feature enabled, [`SerializableType`] gains
//! `Serialize`/`Deserialize` derives. [`Type`] itself only derives
//! `Serialize`: deserialization needs an arena and a builder, so it always
//! goes through [`SerializableType::build`].
//!
//! # Identity contract
//!
//! Round-tripping preserves **structural content**, not addresses. After
//! `let restored = ty.to_serializable().build(&mut builder);`:
//!
//! - `ty == restored` always (structural equality).
//! - `ty.ptr_eq(&restored)` iff `builder` is the builder that produced
//!   `ty`; hash-consing then hands back the original allocations.
//!
//! Consumers caching across runs should store [`SerializableType`], not
//! [`Type`], and call [`SerializableType::build`] after deserialising.
//!
//! A [`Typed`] (a type plus its flow flags and consumer meta byte) round-
//! trips through [`SerializableTyped`], which carries the raw flag bits and
//! the meta byte alongside the structural form.

use std::num::NonZeroU32;

#[cfg(feature = "serde")]
use serde::Deserialize;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_flags::U16Flags;
use mago_span::Span;

use crate::symbol::class_like::ClassLikeKind;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::ty::Type;
use crate::ty::Typed;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::alias::AliasAtom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::callable::CallableAlias;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::ParameterFlag;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::callable::SignatureFlag;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::enumeration::EnumAtom;
use crate::ty::atom::payload::object::has_method::HasMethodAtom;
use crate::ty::atom::payload::object::has_property::HasPropertyAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::named::ObjectFlag;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::object::shape::ObjectShapeFlag;
use crate::ty::atom::payload::reference::GlobalReferenceAtom;
use crate::ty::atom::payload::reference::MemberReferenceAtom;
use crate::ty::atom::payload::reference::NameSelector;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::resource::ResourceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::float::LiteralFloat;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::atom::payload::variable::VariableAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known;
use crate::var::Var;

/// Self-contained structural form of a [`Type`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableType {
    pub atoms: Vec<SerializableAtom>,
}

/// Self-contained structural form of a [`Typed`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableTyped {
    pub ty: SerializableType,
    /// Raw [`FlowFlag`](crate::flags::FlowFlag) bits.
    pub flags: u16,
    /// Consumer meta byte from [`Typed::meta`].
    pub meta: u8,
}

/// Self-contained structural form of one atom within a
/// [`SerializableType`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableAtom {
    Null,
    Never,
    Void,
    Placeholder,
    Mixed {
        non_null: bool,
        is_empty: bool,
        truthiness: SerializableTruthiness,
    },
    Bool,
    True,
    False,
    Int(SerializableInt),
    Float(SerializableFloat),
    String(SerializableString),
    ClassLikeString {
        kind: SerializableClassLikeKind,
        specifier: SerializableClassLikeSpecifier,
    },
    Scalar,
    Numeric,
    ArrayKey,
    Object {
        name: Vec<u8>,
        type_arguments: Option<Vec<SerializableType>>,
        is_static: bool,
        is_this: bool,
        remapped_parameters: bool,
    },
    Enum {
        name: Vec<u8>,
        case: Option<Vec<u8>>,
    },
    ObjectShape {
        known_properties: Vec<SerializableKnownProperty>,
        sealed: bool,
    },
    HasMethod {
        method_name: Vec<u8>,
    },
    HasProperty {
        property_name: Vec<u8>,
    },
    Array {
        key_param: Option<Box<SerializableType>>,
        value_param: Option<Box<SerializableType>>,
        known_items: Vec<SerializableKnownItem>,
        non_empty: bool,
    },
    List {
        element_type: Box<SerializableType>,
        known_elements: Vec<SerializableKnownElement>,
        known_count: Option<u32>,
        non_empty: bool,
    },
    Iterable {
        key_type: Box<SerializableType>,
        value_type: Box<SerializableType>,
    },
    Callable(SerializableCallable),
    Resource(SerializableResource),
    GenericParameter {
        name: Vec<u8>,
        defining_entity: SerializableDefiningEntity,
        constraint: Box<SerializableType>,
    },
    Variable {
        name: Vec<u8>,
    },
    Reference {
        name: Vec<u8>,
        type_arguments: Option<Vec<SerializableType>>,
    },
    MemberReference {
        class_like_name: Vec<u8>,
        selector: SerializableNameSelector,
    },
    GlobalReference {
        selector: SerializableNameSelector,
    },
    Alias {
        class_name: Vec<u8>,
        alias_name: Vec<u8>,
    },
    Conditional {
        subject: Box<SerializableType>,
        target: Box<SerializableType>,
        then: Box<SerializableType>,
        otherwise: Box<SerializableType>,
        negated: bool,
    },
    Derived(SerializableDerived),
    ObjectAny,
    Negated {
        inner: SerializableType,
    },
    Intersected {
        head: Box<SerializableAtom>,
        conjuncts: Vec<SerializableAtom>,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableTruthiness {
    Undetermined,
    Truthy,
    Falsy,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableInt {
    Unspecified,
    UnspecifiedLiteral,
    Literal(i64),
    Range { lower: Option<i64>, upper: Option<i64> },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableFloat {
    Unspecified,
    UnspecifiedLiteral,
    Literal(f64),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableString {
    pub literal: SerializableStringLiteral,
    pub casing: SerializableStringCasing,
    pub is_numeric: bool,
    pub is_truthy: bool,
    pub is_non_empty: bool,
    pub is_callable: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableStringLiteral {
    None,
    Unspecified,
    Value(Vec<u8>),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableStringCasing {
    Unspecified,
    Lowercase,
    Uppercase,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableClassLikeKind {
    Class,
    Interface,
    Enum,
    Trait,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableClassLikeSpecifier {
    Any,
    Literal { value: Vec<u8> },
    OfType { constraint: Box<SerializableType> },
    Generic { constraint: Box<SerializableType> },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SerializableResource {
    Any,
    Open,
    Closed,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableKnownProperty {
    pub name: Vec<u8>,
    pub value: SerializableType,
    pub optional: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableKnownItem {
    pub key: SerializableArrayKey,
    pub value: SerializableType,
    pub optional: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableArrayKey {
    Int(i64),
    String(Vec<u8>),
    Const { class: Vec<u8>, name: Vec<u8> },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableKnownElement {
    pub index: u32,
    pub value: SerializableType,
    pub optional: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableCallable {
    Any,
    Signature(SerializableSignature),
    Closure(SerializableSignature),
    Alias(SerializableCallableAlias),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableSignature {
    pub return_type: SerializableType,
    pub throws: Option<SerializableType>,
    pub parameters: Vec<SerializableParameter>,
    pub is_variadic: bool,
    pub is_pure: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SerializableParameter {
    pub name: Vec<u8>,
    pub r#type: SerializableType,
    pub has_default: bool,
    pub by_reference: bool,
    pub variadic: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableCallableAlias {
    Function(Vec<u8>),
    Method { class: Vec<u8>, method: Vec<u8> },
    Closure(Span),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableDefiningEntity {
    ClassLike(Vec<u8>),
    Method { class: Vec<u8>, method: Vec<u8> },
    Function(Vec<u8>),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableNameSelector {
    Identifier(Vec<u8>),
    StartsWith(Vec<u8>),
    EndsWith(Vec<u8>),
    Contains(Vec<u8>),
    Wildcard,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SerializableDerived {
    KeyOf(Box<SerializableType>),
    ValueOf(Box<SerializableType>),
    PropertiesOf {
        target: Box<SerializableType>,
        visibility: Option<Visibility>,
    },
    IndexAccess {
        target: Box<SerializableType>,
        index: Box<SerializableType>,
    },
    IntMask(Vec<SerializableType>),
    IntMaskOf(Box<SerializableType>),
    TemplateType {
        object: Box<SerializableType>,
        class_name: Box<SerializableType>,
        template_name: Box<SerializableType>,
    },
    New(Box<SerializableType>),
}

impl Type<'_> {
    /// Build a self-contained structural mirror of `self` suitable for
    /// persistence. See the [`crate::ty::serialize`] module docs for the
    /// identity contract.
    #[inline]
    #[must_use]
    pub fn to_serializable(self) -> SerializableType {
        SerializableType { atoms: self.atoms.iter().map(|&atom| encode_atom(atom)).collect() }
    }
}

impl SerializableType {
    /// Re-cons this structural form through `builder` and return a fresh
    /// [`Type`]. The result is structurally equal to the type that
    /// produced this `SerializableType`.
    #[inline]
    pub fn build<'arena, S, A>(&self, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
    where
        S: Arena,
        A: Arena,
    {
        let atoms: Vec<Atom<'arena>> = self.atoms.iter().map(|atom| decode_atom(atom, builder)).collect();

        builder.union_of(&atoms)
    }
}

impl Typed<'_> {
    /// Build a self-contained structural mirror of `self`, carrying the raw
    /// flow-flag bits and the consumer meta byte alongside the structural
    /// form of the type.
    #[inline]
    #[must_use]
    pub fn to_serializable(self) -> SerializableTyped {
        SerializableTyped { ty: self.ty.to_serializable(), flags: self.flags.bits(), meta: self.meta }
    }
}

impl SerializableTyped {
    /// Re-cons this structural form through `builder` and return a fresh
    /// [`Typed`] with the recorded flow flags and meta byte restored.
    #[inline]
    pub fn build<'arena, S, A>(&self, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Typed<'arena>
    where
        S: Arena,
        A: Arena,
    {
        Typed { ty: self.ty.build(builder), flags: U16Flags::from_bits(self.flags), meta: self.meta }
    }
}

impl Atom<'_> {
    /// Build a self-contained structural mirror of `self` suitable for
    /// persistence. Round-trips through [`SerializableAtom::build`]
    /// preserving structural content (arena addresses are not preserved;
    /// see [`crate::ty::serialize`] module docs).
    #[inline]
    #[must_use]
    pub fn to_serializable(self) -> SerializableAtom {
        encode_atom(self)
    }
}

impl SerializableAtom {
    /// Re-cons this structural form through `builder` and return a fresh
    /// [`Atom`]. Equivalent (structurally) to the original atom that
    /// produced this `SerializableAtom`.
    #[inline]
    #[must_use]
    pub fn build<'arena, S, A>(&self, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
    where
        S: Arena,
        A: Arena,
    {
        decode_atom(self, builder)
    }
}

#[inline]
fn encode_type(ty: Type<'_>) -> SerializableType {
    ty.to_serializable()
}

#[inline]
fn encode_name(name: &[u8]) -> Vec<u8> {
    name.to_vec()
}

#[inline]
fn encode_atom(atom: Atom<'_>) -> SerializableAtom {
    match atom {
        Atom::Null => SerializableAtom::Null,
        Atom::Never => SerializableAtom::Never,
        Atom::Void => SerializableAtom::Void,
        Atom::Placeholder => SerializableAtom::Placeholder,
        Atom::Bool => SerializableAtom::Bool,
        Atom::True => SerializableAtom::True,
        Atom::False => SerializableAtom::False,
        Atom::Scalar => SerializableAtom::Scalar,
        Atom::Numeric => SerializableAtom::Numeric,
        Atom::ArrayKey => SerializableAtom::ArrayKey,
        Atom::ObjectAny => SerializableAtom::ObjectAny,
        Atom::Mixed(payload) => SerializableAtom::Mixed {
            non_null: payload.is_non_null(),
            is_empty: payload.is_empty(),
            truthiness: encode_truthiness(payload.truthiness()),
        },
        Atom::Int(payload) => SerializableAtom::Int(encode_int(payload)),
        Atom::Float(payload) => SerializableAtom::Float(encode_float(payload)),
        Atom::String(payload) => SerializableAtom::String(encode_string(payload)),
        Atom::ClassLikeString(payload) => SerializableAtom::ClassLikeString {
            kind: encode_class_like_kind(payload.kind),
            specifier: encode_class_like_specifier(payload.specifier),
        },
        Atom::Object(payload) => SerializableAtom::Object {
            name: encode_name(payload.name.as_bytes()),
            type_arguments: payload
                .type_arguments
                .map(|type_arguments| type_arguments.iter().map(|&ty| encode_type(ty)).collect()),
            is_static: payload.flags.contains(ObjectFlag::IsStatic),
            is_this: payload.flags.contains(ObjectFlag::IsThis),
            remapped_parameters: payload.flags.contains(ObjectFlag::RemappedParameters),
        },
        Atom::Enum(payload) => {
            SerializableAtom::Enum { name: encode_name(payload.name.as_bytes()), case: payload.case.map(encode_name) }
        }
        Atom::ObjectShape(payload) => {
            let known_properties: Vec<SerializableKnownProperty> = payload
                .known_properties
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| SerializableKnownProperty {
                            name: encode_name(entry.name),
                            value: encode_type(entry.value),
                            optional: entry.optional,
                        })
                        .collect()
                })
                .unwrap_or_default();

            SerializableAtom::ObjectShape { known_properties, sealed: payload.flags.contains(ObjectShapeFlag::Sealed) }
        }
        Atom::HasMethod(payload) => SerializableAtom::HasMethod { method_name: encode_name(payload.method_name) },
        Atom::HasProperty(payload) => {
            SerializableAtom::HasProperty { property_name: encode_name(payload.property_name) }
        }
        Atom::Array(payload) => {
            let known_items: Vec<SerializableKnownItem> = payload
                .known_items
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| SerializableKnownItem {
                            key: encode_array_key(entry.key),
                            value: encode_type(entry.value),
                            optional: entry.optional,
                        })
                        .collect()
                })
                .unwrap_or_default();

            SerializableAtom::Array {
                key_param: payload.key_param.map(|ty| Box::new(encode_type(ty))),
                value_param: payload.value_param.map(|ty| Box::new(encode_type(ty))),
                known_items,
                non_empty: payload.flags.contains(ArrayFlag::NonEmpty),
            }
        }
        Atom::List(payload) => {
            let known_elements: Vec<SerializableKnownElement> = payload
                .known_elements
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| SerializableKnownElement {
                            index: entry.index,
                            value: encode_type(entry.value),
                            optional: entry.optional,
                        })
                        .collect()
                })
                .unwrap_or_default();

            SerializableAtom::List {
                element_type: Box::new(encode_type(payload.element_type)),
                known_elements,
                known_count: payload.known_count.map(NonZeroU32::get),
                non_empty: payload.flags.contains(ListFlag::NonEmpty),
            }
        }
        Atom::Iterable(payload) => SerializableAtom::Iterable {
            key_type: Box::new(encode_type(payload.key_type)),
            value_type: Box::new(encode_type(payload.value_type)),
        },
        Atom::Callable(payload) => SerializableAtom::Callable(encode_callable(payload)),
        Atom::Resource(payload) => SerializableAtom::Resource(encode_resource(payload)),
        Atom::GenericParameter(payload) => SerializableAtom::GenericParameter {
            name: encode_name(payload.name),
            defining_entity: encode_defining_entity(payload.defining_entity),
            constraint: Box::new(encode_type(payload.constraint)),
        },
        Atom::Variable(payload) => SerializableAtom::Variable { name: encode_name(payload.name.as_bytes()) },
        Atom::Reference(payload) => SerializableAtom::Reference {
            name: encode_name(payload.name.as_bytes()),
            type_arguments: payload
                .type_arguments
                .map(|type_arguments| type_arguments.iter().map(|&ty| encode_type(ty)).collect()),
        },
        Atom::MemberReference(payload) => SerializableAtom::MemberReference {
            class_like_name: encode_name(payload.class_like_name.as_bytes()),
            selector: encode_name_selector(payload.selector),
        },
        Atom::GlobalReference(payload) => {
            SerializableAtom::GlobalReference { selector: encode_name_selector(payload.selector) }
        }
        Atom::Alias(payload) => SerializableAtom::Alias {
            class_name: encode_name(payload.class_name.as_bytes()),
            alias_name: encode_name(payload.alias_name),
        },
        Atom::Conditional(payload) => SerializableAtom::Conditional {
            subject: Box::new(encode_type(payload.subject)),
            target: Box::new(encode_type(payload.target)),
            then: Box::new(encode_type(payload.then)),
            otherwise: Box::new(encode_type(payload.otherwise)),
            negated: payload.negated,
        },
        Atom::Derived(payload) => SerializableAtom::Derived(encode_derived(*payload)),
        Atom::Negated(inner) => SerializableAtom::Negated { inner: encode_type(*inner) },
        Atom::Intersected(payload) => SerializableAtom::Intersected {
            head: Box::new(encode_atom(*payload.head)),
            conjuncts: payload.conjuncts.iter().map(|&conjunct| encode_atom(conjunct)).collect(),
        },
    }
}

#[inline]
const fn encode_truthiness(truthiness: Truthiness) -> SerializableTruthiness {
    match truthiness {
        Truthiness::Undetermined => SerializableTruthiness::Undetermined,
        Truthiness::Truthy => SerializableTruthiness::Truthy,
        Truthiness::Falsy => SerializableTruthiness::Falsy,
    }
}

#[inline]
fn encode_int(payload: IntAtom<'_>) -> SerializableInt {
    match payload {
        IntAtom::Unspecified => SerializableInt::Unspecified,
        IntAtom::UnspecifiedLiteral => SerializableInt::UnspecifiedLiteral,
        IntAtom::Literal(value) => SerializableInt::Literal(value),
        IntAtom::Range(range) => SerializableInt::Range { lower: range.lower(), upper: range.upper() },
    }
}

#[inline]
const fn encode_float(payload: FloatAtom) -> SerializableFloat {
    match payload {
        FloatAtom::Unspecified => SerializableFloat::Unspecified,
        FloatAtom::UnspecifiedLiteral => SerializableFloat::UnspecifiedLiteral,
        FloatAtom::Literal(literal) => SerializableFloat::Literal(literal.value()),
    }
}

#[inline]
fn encode_string(payload: &StringAtom<'_>) -> SerializableString {
    SerializableString {
        literal: match payload.literal {
            StringLiteral::None => SerializableStringLiteral::None,
            StringLiteral::Unspecified => SerializableStringLiteral::Unspecified,
            StringLiteral::Value(value) => SerializableStringLiteral::Value(encode_name(value)),
        },
        casing: match payload.casing {
            StringCasing::Unspecified => SerializableStringCasing::Unspecified,
            StringCasing::Lowercase => SerializableStringCasing::Lowercase,
            StringCasing::Uppercase => SerializableStringCasing::Uppercase,
        },
        is_numeric: payload.flags.contains(StringRefinementFlag::Numeric),
        is_truthy: payload.flags.contains(StringRefinementFlag::Truthy),
        is_non_empty: payload.flags.contains(StringRefinementFlag::NonEmpty),
        is_callable: payload.flags.contains(StringRefinementFlag::Callable),
    }
}

#[inline]
const fn encode_class_like_kind(kind: ClassLikeKind) -> SerializableClassLikeKind {
    match kind {
        ClassLikeKind::Class => SerializableClassLikeKind::Class,
        ClassLikeKind::Interface => SerializableClassLikeKind::Interface,
        ClassLikeKind::Enum => SerializableClassLikeKind::Enum,
        ClassLikeKind::Trait => SerializableClassLikeKind::Trait,
    }
}

#[inline]
fn encode_class_like_specifier(specifier: ClassLikeStringSpecifier<'_>) -> SerializableClassLikeSpecifier {
    match specifier {
        ClassLikeStringSpecifier::Any => SerializableClassLikeSpecifier::Any,
        ClassLikeStringSpecifier::Literal { value } => {
            SerializableClassLikeSpecifier::Literal { value: encode_name(value.as_bytes()) }
        }
        ClassLikeStringSpecifier::OfType { constraint } => {
            SerializableClassLikeSpecifier::OfType { constraint: Box::new(encode_type(constraint)) }
        }
        ClassLikeStringSpecifier::Generic { constraint } => {
            SerializableClassLikeSpecifier::Generic { constraint: Box::new(encode_type(constraint)) }
        }
    }
}

#[inline]
fn encode_array_key(key: ArrayKey<'_>) -> SerializableArrayKey {
    match key {
        ArrayKey::Int(value) => SerializableArrayKey::Int(value),
        ArrayKey::String(name) => SerializableArrayKey::String(encode_name(name)),
        ArrayKey::Const { class, name } => {
            SerializableArrayKey::Const { class: encode_name(class.as_bytes()), name: encode_name(name) }
        }
    }
}

#[inline]
fn encode_callable(payload: CallableAtom<'_>) -> SerializableCallable {
    match payload {
        CallableAtom::Any => SerializableCallable::Any,
        CallableAtom::Signature(signature) => SerializableCallable::Signature(encode_signature(signature)),
        CallableAtom::Closure(signature) => SerializableCallable::Closure(encode_signature(signature)),
        CallableAtom::Alias(alias) => SerializableCallable::Alias(encode_callable_alias(alias)),
    }
}

#[inline]
fn encode_signature(signature: &Signature<'_>) -> SerializableSignature {
    let parameters: Vec<SerializableParameter> = signature
        .parameters
        .map(|entries| {
            entries
                .iter()
                .map(|parameter| SerializableParameter {
                    name: encode_name(parameter.name),
                    r#type: encode_type(parameter.r#type),
                    has_default: parameter.flags.contains(ParameterFlag::HasDefault),
                    by_reference: parameter.flags.contains(ParameterFlag::ByReference),
                    variadic: parameter.flags.contains(ParameterFlag::Variadic),
                })
                .collect()
        })
        .unwrap_or_default();

    SerializableSignature {
        return_type: encode_type(signature.return_type),
        throws: signature.throws.map(encode_type),
        parameters,
        is_variadic: signature.flags.contains(SignatureFlag::IsVariadic),
        is_pure: signature.flags.contains(SignatureFlag::IsPure),
    }
}

#[inline]
fn encode_callable_alias(alias: &CallableAlias<'_>) -> SerializableCallableAlias {
    match *alias {
        CallableAlias::Function(name) => SerializableCallableAlias::Function(encode_name(name.as_bytes())),
        CallableAlias::Method { class, method } => {
            SerializableCallableAlias::Method { class: encode_name(class.as_bytes()), method: encode_name(method) }
        }
        CallableAlias::Closure(span) => SerializableCallableAlias::Closure(span),
    }
}

#[inline]
const fn encode_resource(payload: ResourceAtom) -> SerializableResource {
    match payload {
        ResourceAtom::Any => SerializableResource::Any,
        ResourceAtom::Open => SerializableResource::Open,
        ResourceAtom::Closed => SerializableResource::Closed,
    }
}

#[inline]
fn encode_defining_entity(entity: DefiningEntity<'_>) -> SerializableDefiningEntity {
    match entity {
        DefiningEntity::ClassLike(name) => SerializableDefiningEntity::ClassLike(encode_name(name.as_bytes())),
        DefiningEntity::Method { class, method } => {
            SerializableDefiningEntity::Method { class: encode_name(class.as_bytes()), method: encode_name(method) }
        }
        DefiningEntity::Function(name) => SerializableDefiningEntity::Function(encode_name(name.as_bytes())),
    }
}

#[inline]
fn encode_name_selector(selector: NameSelector<'_>) -> SerializableNameSelector {
    match selector {
        NameSelector::Identifier(name) => SerializableNameSelector::Identifier(encode_name(name)),
        NameSelector::StartsWith(name) => SerializableNameSelector::StartsWith(encode_name(name)),
        NameSelector::EndsWith(name) => SerializableNameSelector::EndsWith(encode_name(name)),
        NameSelector::Contains(name) => SerializableNameSelector::Contains(encode_name(name)),
        NameSelector::Wildcard => SerializableNameSelector::Wildcard,
    }
}

#[inline]
fn encode_derived(payload: DerivedAtom<'_>) -> SerializableDerived {
    match payload {
        DerivedAtom::KeyOf(target) => SerializableDerived::KeyOf(Box::new(encode_type(target))),
        DerivedAtom::ValueOf(target) => SerializableDerived::ValueOf(Box::new(encode_type(target))),
        DerivedAtom::PropertiesOf { target, visibility } => {
            SerializableDerived::PropertiesOf { target: Box::new(encode_type(target)), visibility }
        }
        DerivedAtom::IndexAccess { target, index } => SerializableDerived::IndexAccess {
            target: Box::new(encode_type(target)),
            index: Box::new(encode_type(index)),
        },
        DerivedAtom::IntMask(members) => {
            SerializableDerived::IntMask(members.iter().map(|&ty| encode_type(ty)).collect())
        }
        DerivedAtom::IntMaskOf(target) => SerializableDerived::IntMaskOf(Box::new(encode_type(target))),
        DerivedAtom::TemplateType { object, class_name, template_name } => SerializableDerived::TemplateType {
            object: Box::new(encode_type(object)),
            class_name: Box::new(encode_type(class_name)),
            template_name: Box::new(encode_type(template_name)),
        },
        DerivedAtom::New(target) => SerializableDerived::New(Box::new(encode_type(target))),
    }
}

#[inline]
fn decode_atom<'arena, S, A>(atom: &SerializableAtom, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        SerializableAtom::Null => well_known::NULL,
        SerializableAtom::Never => well_known::NEVER,
        SerializableAtom::Void => well_known::VOID,
        SerializableAtom::Placeholder => well_known::PLACEHOLDER,
        SerializableAtom::Bool => well_known::BOOL,
        SerializableAtom::True => well_known::TRUE,
        SerializableAtom::False => well_known::FALSE,
        SerializableAtom::Scalar => well_known::SCALAR,
        SerializableAtom::Numeric => well_known::NUMERIC,
        SerializableAtom::ArrayKey => well_known::ARRAY_KEY,
        SerializableAtom::ObjectAny => well_known::OBJECT,
        SerializableAtom::Mixed { non_null, is_empty, truthiness } => Atom::Mixed(
            MixedAtom::EMPTY
                .with_is_non_null(*non_null)
                .with_is_empty(*is_empty)
                .with_truthiness(decode_truthiness(*truthiness)),
        ),
        SerializableAtom::Int(payload) => decode_int(*payload, builder),
        SerializableAtom::Float(payload) => Atom::Float(decode_float(*payload)),
        SerializableAtom::String(payload) => decode_string(payload, builder),
        SerializableAtom::ClassLikeString { kind, specifier } => {
            let kind = decode_class_like_kind(*kind);
            let specifier = decode_class_like_specifier(specifier, builder);

            builder.class_like_string(ClassLikeStringAtom { kind, specifier })
        }
        SerializableAtom::Object { name, type_arguments, is_static, is_this, remapped_parameters } => {
            let name = builder.intern_class_like_path(name);
            let type_arguments = type_arguments.as_ref().map(|arguments| {
                let types: Vec<Type<'arena>> = arguments.iter().map(|argument| argument.build(builder)).collect();

                builder.types(&types)
            });
            let mut flags = U8Flags::empty();
            flags.set_value(ObjectFlag::IsStatic, *is_static);
            flags.set_value(ObjectFlag::IsThis, *is_this);
            flags.set_value(ObjectFlag::RemappedParameters, *remapped_parameters);

            builder.object(ObjectAtom { name, type_arguments, flags })
        }
        SerializableAtom::Enum { name, case } => {
            let name = builder.intern_class_like_path(name);
            let case = case.as_ref().map(|case| builder.intern(case));

            builder.enumeration(EnumAtom { name, case })
        }
        SerializableAtom::ObjectShape { known_properties, sealed } => {
            let known_properties = if known_properties.is_empty() {
                None
            } else {
                let entries: Vec<KnownProperty<'arena>> = known_properties
                    .iter()
                    .map(|property| KnownProperty {
                        name: builder.intern(&property.name),
                        value: property.value.build(builder),
                        optional: property.optional,
                    })
                    .collect();

                Some(builder.known_properties(&entries))
            };
            let mut flags = U8Flags::empty();
            flags.set_value(ObjectShapeFlag::Sealed, *sealed);

            builder.object_shape(ObjectShapeAtom { known_properties, flags })
        }
        SerializableAtom::HasMethod { method_name } => {
            Atom::HasMethod(HasMethodAtom { method_name: builder.intern(method_name) })
        }
        SerializableAtom::HasProperty { property_name } => {
            Atom::HasProperty(HasPropertyAtom { property_name: builder.intern(property_name) })
        }
        SerializableAtom::Array { key_param, value_param, known_items, non_empty } => {
            let key_param = key_param.as_ref().map(|ty| ty.build(builder));
            let value_param = value_param.as_ref().map(|ty| ty.build(builder));
            let known_items = if known_items.is_empty() {
                None
            } else {
                let entries: Vec<KnownItem<'arena>> = known_items
                    .iter()
                    .map(|item| KnownItem {
                        key: decode_array_key(&item.key, builder),
                        value: item.value.build(builder),
                        optional: item.optional,
                    })
                    .collect();

                Some(builder.known_items(&entries))
            };
            let mut flags = U8Flags::empty();
            flags.set_value(ArrayFlag::NonEmpty, *non_empty);

            builder.array(ArrayAtom { key_param, value_param, known_items, flags })
        }
        SerializableAtom::List { element_type, known_elements, known_count, non_empty } => {
            let element_type = element_type.build(builder);
            let known_elements = if known_elements.is_empty() {
                None
            } else {
                let entries: Vec<KnownElement<'arena>> = known_elements
                    .iter()
                    .map(|element| KnownElement {
                        index: element.index,
                        value: element.value.build(builder),
                        optional: element.optional,
                    })
                    .collect();

                Some(builder.known_elements(&entries))
            };
            let mut flags = U8Flags::empty();
            flags.set_value(ListFlag::NonEmpty, *non_empty);

            builder.list(ListAtom {
                element_type,
                known_elements,
                known_count: known_count.and_then(NonZeroU32::new),
                flags,
            })
        }
        SerializableAtom::Iterable { key_type, value_type } => {
            let key_type = key_type.build(builder);
            let value_type = value_type.build(builder);

            builder.iterable(IterableAtom { key_type, value_type })
        }
        SerializableAtom::Callable(callable) => Atom::Callable(decode_callable(callable, builder)),
        SerializableAtom::Resource(resource) => Atom::Resource(decode_resource(*resource)),
        SerializableAtom::GenericParameter { name, defining_entity, constraint } => {
            let name = builder.intern(name);
            let defining_entity = decode_defining_entity(defining_entity, builder);
            let constraint = constraint.build(builder);

            builder.generic_parameter(GenericParameterAtom { name, defining_entity, constraint })
        }
        SerializableAtom::Variable { name } => Atom::Variable(VariableAtom { name: Var::new(builder.intern(name)) }),
        SerializableAtom::Reference { name, type_arguments } => {
            let name = builder.intern_class_like_path(name);
            let type_arguments = type_arguments.as_ref().map(|arguments| {
                let types: Vec<Type<'arena>> = arguments.iter().map(|argument| argument.build(builder)).collect();

                builder.types(&types)
            });

            builder.reference(SymbolReferenceAtom { name, type_arguments })
        }
        SerializableAtom::MemberReference { class_like_name, selector } => {
            let class_like_name = builder.intern_class_like_path(class_like_name);
            let selector = decode_name_selector(selector, builder);

            builder.member_reference(MemberReferenceAtom { class_like_name, selector })
        }
        SerializableAtom::GlobalReference { selector } => {
            let selector = decode_name_selector(selector, builder);

            builder.global_reference(GlobalReferenceAtom { selector })
        }
        SerializableAtom::Alias { class_name, alias_name } => {
            let class_name = builder.intern_class_like_path(class_name);
            let alias_name = builder.intern(alias_name);

            builder.alias(AliasAtom { class_name, alias_name })
        }
        SerializableAtom::Conditional { subject, target, then, otherwise, negated } => {
            let subject = subject.build(builder);
            let target = target.build(builder);
            let then = then.build(builder);
            let otherwise = otherwise.build(builder);

            builder.conditional(ConditionalAtom { subject, target, then, otherwise, negated: *negated })
        }
        SerializableAtom::Derived(derived) => {
            let payload = decode_derived(derived, builder);

            builder.derived(payload)
        }
        SerializableAtom::Negated { inner } => {
            let inner = inner.build(builder);

            builder.negated(inner)
        }
        SerializableAtom::Intersected { head, conjuncts } => {
            let head = decode_atom(head, builder);
            let conjuncts: Vec<Atom<'arena>> =
                conjuncts.iter().map(|conjunct| decode_atom(conjunct, builder)).collect();

            builder.intersected(head, &conjuncts)
        }
    }
}

#[inline]
const fn decode_truthiness(truthiness: SerializableTruthiness) -> Truthiness {
    match truthiness {
        SerializableTruthiness::Undetermined => Truthiness::Undetermined,
        SerializableTruthiness::Truthy => Truthiness::Truthy,
        SerializableTruthiness::Falsy => Truthiness::Falsy,
    }
}

#[inline]
fn decode_int<'arena, S, A>(payload: SerializableInt, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match payload {
        SerializableInt::Unspecified => Atom::Int(IntAtom::Unspecified),
        SerializableInt::UnspecifiedLiteral => Atom::Int(IntAtom::UnspecifiedLiteral),
        SerializableInt::Literal(value) => Atom::int_literal(value),
        SerializableInt::Range { lower, upper } => builder.int_range_atom(lower, upper),
    }
}

#[inline]
const fn decode_float(payload: SerializableFloat) -> FloatAtom {
    match payload {
        SerializableFloat::Unspecified => FloatAtom::Unspecified,
        SerializableFloat::UnspecifiedLiteral => FloatAtom::UnspecifiedLiteral,
        SerializableFloat::Literal(value) => FloatAtom::Literal(LiteralFloat::new(value)),
    }
}

#[inline]
fn decode_string<'arena, S, A>(
    payload: &SerializableString,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let literal = match &payload.literal {
        SerializableStringLiteral::None => StringLiteral::None,
        SerializableStringLiteral::Unspecified => StringLiteral::Unspecified,
        SerializableStringLiteral::Value(value) => StringLiteral::Value(builder.intern(value)),
    };
    let casing = match payload.casing {
        SerializableStringCasing::Unspecified => StringCasing::Unspecified,
        SerializableStringCasing::Lowercase => StringCasing::Lowercase,
        SerializableStringCasing::Uppercase => StringCasing::Uppercase,
    };
    let mut flags = U8Flags::empty();
    flags.set_value(StringRefinementFlag::Numeric, payload.is_numeric);
    flags.set_value(StringRefinementFlag::Truthy, payload.is_truthy);
    flags.set_value(StringRefinementFlag::NonEmpty, payload.is_non_empty);
    flags.set_value(StringRefinementFlag::Callable, payload.is_callable);

    builder.string(StringAtom { literal, casing, flags })
}

#[inline]
const fn decode_class_like_kind(kind: SerializableClassLikeKind) -> ClassLikeKind {
    match kind {
        SerializableClassLikeKind::Class => ClassLikeKind::Class,
        SerializableClassLikeKind::Interface => ClassLikeKind::Interface,
        SerializableClassLikeKind::Enum => ClassLikeKind::Enum,
        SerializableClassLikeKind::Trait => ClassLikeKind::Trait,
    }
}

#[inline]
fn decode_class_like_specifier<'arena, S, A>(
    specifier: &SerializableClassLikeSpecifier,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> ClassLikeStringSpecifier<'arena>
where
    S: Arena,
    A: Arena,
{
    match specifier {
        SerializableClassLikeSpecifier::Any => ClassLikeStringSpecifier::Any,
        SerializableClassLikeSpecifier::Literal { value } => {
            ClassLikeStringSpecifier::Literal { value: builder.intern_class_like_path(value) }
        }
        SerializableClassLikeSpecifier::OfType { constraint } => {
            ClassLikeStringSpecifier::OfType { constraint: constraint.build(builder) }
        }
        SerializableClassLikeSpecifier::Generic { constraint } => {
            ClassLikeStringSpecifier::Generic { constraint: constraint.build(builder) }
        }
    }
}

#[inline]
fn decode_array_key<'arena, S, A>(
    key: &SerializableArrayKey,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> ArrayKey<'arena>
where
    S: Arena,
    A: Arena,
{
    match key {
        SerializableArrayKey::Int(value) => ArrayKey::Int(*value),
        SerializableArrayKey::String(name) => ArrayKey::String(builder.intern(name)),
        SerializableArrayKey::Const { class, name } => {
            let class = builder.intern_class_like_path(class);
            let name = builder.intern(name);
            ArrayKey::Const { class, name }
        }
    }
}

#[inline]
fn decode_callable<'arena, S, A>(
    callable: &SerializableCallable,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> CallableAtom<'arena>
where
    S: Arena,
    A: Arena,
{
    match callable {
        SerializableCallable::Any => CallableAtom::Any,
        SerializableCallable::Signature(signature) => {
            let signature = decode_signature(signature, builder);

            CallableAtom::Signature(builder.signature(signature))
        }
        SerializableCallable::Closure(signature) => {
            let signature = decode_signature(signature, builder);

            CallableAtom::Closure(builder.signature(signature))
        }
        SerializableCallable::Alias(alias) => {
            let alias = decode_callable_alias(alias, builder);

            CallableAtom::Alias(builder.callable_alias(alias))
        }
    }
}

#[inline]
fn decode_signature<'arena, S, A>(
    signature: &SerializableSignature,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Signature<'arena>
where
    S: Arena,
    A: Arena,
{
    let parameters = if signature.parameters.is_empty() {
        None
    } else {
        let entries: Vec<Parameter<'arena>> = signature
            .parameters
            .iter()
            .map(|parameter| {
                let mut flags = U8Flags::empty();
                flags.set_value(ParameterFlag::HasDefault, parameter.has_default);
                flags.set_value(ParameterFlag::ByReference, parameter.by_reference);
                flags.set_value(ParameterFlag::Variadic, parameter.variadic);

                Parameter { name: builder.intern(&parameter.name), r#type: parameter.r#type.build(builder), flags }
            })
            .collect();

        Some(builder.parameters(&entries))
    };
    let return_type = signature.return_type.build(builder);
    let throws = signature.throws.as_ref().map(|throws| throws.build(builder));
    let mut flags = U8Flags::empty();
    flags.set_value(SignatureFlag::IsVariadic, signature.is_variadic);
    flags.set_value(SignatureFlag::IsPure, signature.is_pure);

    Signature { parameters, return_type, throws, flags }
}

#[inline]
fn decode_callable_alias<'arena, S, A>(
    alias: &SerializableCallableAlias,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> CallableAlias<'arena>
where
    S: Arena,
    A: Arena,
{
    match alias {
        SerializableCallableAlias::Function(name) => CallableAlias::Function(builder.intern_function_like_path(name)),
        SerializableCallableAlias::Method { class, method } => {
            let class = builder.intern_class_like_path(class);
            let method = builder.intern(method);
            CallableAlias::Method { class, method }
        }
        SerializableCallableAlias::Closure(span) => CallableAlias::Closure(*span),
    }
}

#[inline]
const fn decode_resource(resource: SerializableResource) -> ResourceAtom {
    match resource {
        SerializableResource::Any => ResourceAtom::Any,
        SerializableResource::Open => ResourceAtom::Open,
        SerializableResource::Closed => ResourceAtom::Closed,
    }
}

#[inline]
fn decode_defining_entity<'arena, S, A>(
    entity: &SerializableDefiningEntity,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> DefiningEntity<'arena>
where
    S: Arena,
    A: Arena,
{
    match entity {
        SerializableDefiningEntity::ClassLike(name) => DefiningEntity::ClassLike(builder.intern_class_like_path(name)),
        SerializableDefiningEntity::Method { class, method } => {
            let class = builder.intern_class_like_path(class);
            let method = builder.intern(method);
            DefiningEntity::Method { class, method }
        }
        SerializableDefiningEntity::Function(name) => DefiningEntity::Function(builder.intern_function_like_path(name)),
    }
}

#[inline]
fn decode_name_selector<'arena, S, A>(
    selector: &SerializableNameSelector,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> NameSelector<'arena>
where
    S: Arena,
    A: Arena,
{
    match selector {
        SerializableNameSelector::Identifier(name) => NameSelector::Identifier(builder.intern(name)),
        SerializableNameSelector::StartsWith(name) => NameSelector::StartsWith(builder.intern(name)),
        SerializableNameSelector::EndsWith(name) => NameSelector::EndsWith(builder.intern(name)),
        SerializableNameSelector::Contains(name) => NameSelector::Contains(builder.intern(name)),
        SerializableNameSelector::Wildcard => NameSelector::Wildcard,
    }
}

#[inline]
fn decode_derived<'arena, S, A>(
    derived: &SerializableDerived,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> DerivedAtom<'arena>
where
    S: Arena,
    A: Arena,
{
    match derived {
        SerializableDerived::KeyOf(target) => DerivedAtom::KeyOf(target.build(builder)),
        SerializableDerived::ValueOf(target) => DerivedAtom::ValueOf(target.build(builder)),
        SerializableDerived::PropertiesOf { target, visibility } => {
            DerivedAtom::PropertiesOf { target: target.build(builder), visibility: *visibility }
        }
        SerializableDerived::IndexAccess { target, index } => {
            DerivedAtom::IndexAccess { target: target.build(builder), index: index.build(builder) }
        }
        SerializableDerived::IntMask(types) => {
            let members: Vec<Type<'arena>> = types.iter().map(|ty| ty.build(builder)).collect();

            DerivedAtom::IntMask(builder.types(&members))
        }
        SerializableDerived::IntMaskOf(target) => DerivedAtom::IntMaskOf(target.build(builder)),
        SerializableDerived::TemplateType { object, class_name, template_name } => DerivedAtom::TemplateType {
            object: object.build(builder),
            class_name: class_name.build(builder),
            template_name: template_name.build(builder),
        },
        SerializableDerived::New(target) => DerivedAtom::New(target.build(builder)),
    }
}
