//! Well-known [`Atom`]s and [`Type`]s, fixed at compile time.
//!
//! Everything here has lifetime `'static`; covariance lets it embed into any
//! arena-allocated type without copying. The
//! [`TypeBuilder`](crate::ty::builder::TypeBuilder) seeds its deduplication
//! tables with these values, so building a well-known shape always yields the
//! seeded instance.
//!
//! Atoms are `const` items: their equality is structural (payloads compare by
//! value through the references), so per-use-site promotion is harmless.
//! Types are `static` items on purpose - [`Type::ptr_eq`] fast paths compare
//! the atom-slice address, and only a `static` guarantees one address across
//! every mention.

use mago_flags::U8Flags;

use crate::symbol::class_like::ClassLikeKind;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::resource::ResourceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::int::IntRange;
use crate::ty::atom::payload::scalar::mixed::MixedAtom;
use crate::ty::atom::payload::scalar::mixed::Truthiness;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;

const NO_STRING_FLAGS: U8Flags<StringRefinementFlag> = U8Flags::empty();
const NON_EMPTY_FLAGS: U8Flags<StringRefinementFlag> = U8Flags::from_bits(StringRefinementFlag::NonEmpty as u8);
const TRUTHY_FLAGS: U8Flags<StringRefinementFlag> =
    U8Flags::from_bits(StringRefinementFlag::Truthy as u8 | StringRefinementFlag::NonEmpty as u8);
const NUMERIC_FLAGS: U8Flags<StringRefinementFlag> = U8Flags::from_bits(StringRefinementFlag::Numeric as u8);
const TRUTHY_NUMERIC_FLAGS: U8Flags<StringRefinementFlag> =
    U8Flags::from_bits(NUMERIC_FLAGS.bits() | TRUTHY_FLAGS.bits());
const CALLABLE_FLAGS: U8Flags<StringRefinementFlag> =
    U8Flags::from_bits(StringRefinementFlag::Callable as u8 | TRUTHY_FLAGS.bits());

pub const NULL: Atom<'static> = Atom::Null;
pub const NEVER: Atom<'static> = Atom::Never;
pub const VOID: Atom<'static> = Atom::Void;
pub const PLACEHOLDER: Atom<'static> = Atom::Placeholder;
pub const TRUE: Atom<'static> = Atom::True;
pub const FALSE: Atom<'static> = Atom::False;
pub const SCALAR: Atom<'static> = Atom::Scalar;
pub const NUMERIC: Atom<'static> = Atom::Numeric;
pub const ARRAY_KEY: Atom<'static> = Atom::ArrayKey;
pub const MIXED: Atom<'static> = Atom::Mixed(MixedAtom::EMPTY);
pub const NON_NULL_MIXED: Atom<'static> = Atom::Mixed(MixedAtom::EMPTY.with_is_non_null(true));
pub const TRUTHY_MIXED: Atom<'static> = Atom::Mixed(MixedAtom::EMPTY.with_truthiness(Truthiness::Truthy));
pub const FALSY_MIXED: Atom<'static> = Atom::Mixed(MixedAtom::EMPTY.with_truthiness(Truthiness::Falsy));
pub const ISSET_FROM_LOOP: Atom<'static> = Atom::Mixed(MixedAtom::EMPTY.with_is_isset_from_loop(true));
pub const BOOL: Atom<'static> = Atom::Bool;
pub const OBJECT: Atom<'static> = Atom::ObjectAny;
pub const INT: Atom<'static> = Atom::Int(IntAtom::Unspecified);
pub const POSITIVE_INT: Atom<'static> = Atom::Int(IntAtom::Range(&IntRange::new(Some(1), None)));
pub const NEGATIVE_INT: Atom<'static> = Atom::Int(IntAtom::Range(&IntRange::new(None, Some(-1))));
pub const NON_POSITIVE_INT: Atom<'static> = Atom::Int(IntAtom::Range(&IntRange::new(None, Some(0))));
pub const NON_NEGATIVE_INT: Atom<'static> = Atom::Int(IntAtom::Range(&IntRange::new(Some(0), None)));
pub const LITERAL_INT: Atom<'static> = Atom::Int(IntAtom::UnspecifiedLiteral);
pub const INT_ZERO: Atom<'static> = Atom::Int(IntAtom::Literal(0));
pub const INT_ONE: Atom<'static> = Atom::Int(IntAtom::Literal(1));
pub const INT_MINUS_ONE: Atom<'static> = Atom::Int(IntAtom::Literal(-1));
pub const FLOAT: Atom<'static> = Atom::Float(FloatAtom::Unspecified);
pub const LITERAL_FLOAT: Atom<'static> = Atom::Float(FloatAtom::UnspecifiedLiteral);
pub const STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::None,
    casing: StringCasing::Unspecified,
    flags: NO_STRING_FLAGS,
});
pub const NON_EMPTY_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::None,
    casing: StringCasing::Unspecified,
    flags: NON_EMPTY_FLAGS,
});
pub const TRUTHY_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Unspecified, flags: TRUTHY_FLAGS });
pub const LOWERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Lowercase, flags: NO_STRING_FLAGS });
pub const UPPERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Uppercase, flags: NO_STRING_FLAGS });
pub const NON_EMPTY_LOWERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Lowercase, flags: NON_EMPTY_FLAGS });
pub const NON_EMPTY_UPPERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Uppercase, flags: NON_EMPTY_FLAGS });
pub const TRUTHY_LOWERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Lowercase, flags: TRUTHY_FLAGS });
pub const TRUTHY_UPPERCASE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Uppercase, flags: TRUTHY_FLAGS });
pub const NUMERIC_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Unspecified, flags: NUMERIC_FLAGS });
pub const TRUTHY_NUMERIC_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::None,
    casing: StringCasing::Unspecified,
    flags: TRUTHY_NUMERIC_FLAGS,
});
pub const CALLABLE_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::None,
    casing: StringCasing::Unspecified,
    flags: CALLABLE_FLAGS,
});
pub const LOWERCASE_CALLABLE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Lowercase, flags: CALLABLE_FLAGS });
pub const UPPERCASE_CALLABLE_STRING: Atom<'static> =
    Atom::String(&StringAtom { literal: StringLiteral::None, casing: StringCasing::Uppercase, flags: CALLABLE_FLAGS });
pub const LITERAL_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::Unspecified,
    casing: StringCasing::Unspecified,
    flags: NO_STRING_FLAGS,
});
pub const NON_EMPTY_LITERAL_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::Unspecified,
    casing: StringCasing::Unspecified,
    flags: NON_EMPTY_FLAGS,
});
pub const EMPTY_STRING: Atom<'static> = Atom::String(&StringAtom {
    literal: StringLiteral::Value(b""),
    casing: StringCasing::Unspecified,
    flags: NO_STRING_FLAGS,
});
pub const CLASS_STRING: Atom<'static> = Atom::ClassLikeString(&ClassLikeStringAtom {
    kind: ClassLikeKind::Class,
    specifier: ClassLikeStringSpecifier::Any,
});
pub const INTERFACE_STRING: Atom<'static> = Atom::ClassLikeString(&ClassLikeStringAtom {
    kind: ClassLikeKind::Interface,
    specifier: ClassLikeStringSpecifier::Any,
});
pub const ENUM_STRING: Atom<'static> =
    Atom::ClassLikeString(&ClassLikeStringAtom { kind: ClassLikeKind::Enum, specifier: ClassLikeStringSpecifier::Any });
pub const TRAIT_STRING: Atom<'static> = Atom::ClassLikeString(&ClassLikeStringAtom {
    kind: ClassLikeKind::Trait,
    specifier: ClassLikeStringSpecifier::Any,
});
pub const RESOURCE: Atom<'static> = Atom::Resource(ResourceAtom::Any);
pub const OPEN_RESOURCE: Atom<'static> = Atom::Resource(ResourceAtom::Open);
pub const CLOSED_RESOURCE: Atom<'static> = Atom::Resource(ResourceAtom::Closed);

const TYPE_MIXED_VALUE: Type<'static> = Type::from_canonical_atoms(&[MIXED]);
const TYPE_ARRAY_KEY_VALUE: Type<'static> = Type::from_canonical_atoms(&[ARRAY_KEY]);
pub const ITERABLE_MIXED_MIXED: Atom<'static> =
    Atom::Iterable(&IterableAtom { key_type: TYPE_MIXED_VALUE, value_type: TYPE_MIXED_VALUE });
pub const EMPTY_ARRAY: Atom<'static> =
    Atom::Array(&ArrayAtom { key_param: None, value_param: None, known_items: None, flags: U8Flags::empty() });
pub const ARRAY_KEY_MIXED: Atom<'static> = Atom::Array(&ArrayAtom {
    key_param: Some(TYPE_ARRAY_KEY_VALUE),
    value_param: Some(TYPE_MIXED_VALUE),
    known_items: None,
    flags: U8Flags::empty(),
});
pub const CALLABLE: Atom<'static> = Atom::Callable(CallableAtom::Any);
pub static TYPE_NULL: Type<'static> = Type::from_canonical_atoms(&[NULL]);
pub static TYPE_NEVER: Type<'static> = Type::from_canonical_atoms(&[NEVER]);
pub static TYPE_VOID: Type<'static> = Type::from_canonical_atoms(&[VOID]);
pub static TYPE_MIXED: Type<'static> = TYPE_MIXED_VALUE;
pub static TYPE_BOOL: Type<'static> = Type::from_canonical_atoms(&[BOOL]);
pub static TYPE_TRUE: Type<'static> = Type::from_canonical_atoms(&[TRUE]);
pub static TYPE_FALSE: Type<'static> = Type::from_canonical_atoms(&[FALSE]);
pub static TYPE_INT: Type<'static> = Type::from_canonical_atoms(&[INT]);
pub static TYPE_FLOAT: Type<'static> = Type::from_canonical_atoms(&[FLOAT]);
pub static TYPE_STRING: Type<'static> = Type::from_canonical_atoms(&[STRING]);
pub static TYPE_OBJECT: Type<'static> = Type::from_canonical_atoms(&[OBJECT]);
pub static TYPE_SCALAR: Type<'static> = Type::from_canonical_atoms(&[SCALAR]);
pub static TYPE_NUMERIC: Type<'static> = Type::from_canonical_atoms(&[NUMERIC]);
pub static TYPE_ARRAY_KEY: Type<'static> = TYPE_ARRAY_KEY_VALUE;
pub static TYPE_CALLABLE: Type<'static> = Type::from_canonical_atoms(&[CALLABLE]);
pub static TYPE_INT_OR_FLOAT: Type<'static> = Type::from_canonical_atoms(&[INT, FLOAT]);
pub static TYPE_INT_OR_STRING: Type<'static> = Type::from_canonical_atoms(&[INT, STRING]);
pub static TYPE_NULL_OR_SCALAR: Type<'static> = Type::from_canonical_atoms(&[NULL, SCALAR]);
pub static TYPE_NULL_OR_STRING: Type<'static> = Type::from_canonical_atoms(&[NULL, STRING]);
pub static TYPE_NULL_OR_INT: Type<'static> = Type::from_canonical_atoms(&[NULL, INT]);
pub static TYPE_NULL_OR_FLOAT: Type<'static> = Type::from_canonical_atoms(&[NULL, FLOAT]);
pub static TYPE_NULL_OR_OBJECT: Type<'static> = Type::from_canonical_atoms(&[NULL, OBJECT]);
pub static TYPE_MINUS_ONE_ZERO_ONE: Type<'static> = Type::from_canonical_atoms(&[INT_MINUS_ONE, INT_ZERO, INT_ONE]);

/// Every well-known atom, in declaration order. The
/// [`TypeBuilder`](crate::ty::builder::TypeBuilder) seeds its payload tables from
/// this list.
pub const ATOMS: &[Atom<'static>] = &[
    NULL,
    NEVER,
    VOID,
    PLACEHOLDER,
    TRUE,
    FALSE,
    SCALAR,
    NUMERIC,
    ARRAY_KEY,
    MIXED,
    NON_NULL_MIXED,
    TRUTHY_MIXED,
    FALSY_MIXED,
    ISSET_FROM_LOOP,
    BOOL,
    OBJECT,
    INT,
    POSITIVE_INT,
    NEGATIVE_INT,
    NON_POSITIVE_INT,
    NON_NEGATIVE_INT,
    LITERAL_INT,
    INT_ZERO,
    INT_ONE,
    INT_MINUS_ONE,
    FLOAT,
    LITERAL_FLOAT,
    STRING,
    NON_EMPTY_STRING,
    TRUTHY_STRING,
    LOWERCASE_STRING,
    UPPERCASE_STRING,
    NON_EMPTY_LOWERCASE_STRING,
    NON_EMPTY_UPPERCASE_STRING,
    TRUTHY_LOWERCASE_STRING,
    TRUTHY_UPPERCASE_STRING,
    NUMERIC_STRING,
    TRUTHY_NUMERIC_STRING,
    CALLABLE_STRING,
    LOWERCASE_CALLABLE_STRING,
    UPPERCASE_CALLABLE_STRING,
    LITERAL_STRING,
    NON_EMPTY_LITERAL_STRING,
    EMPTY_STRING,
    CLASS_STRING,
    INTERFACE_STRING,
    ENUM_STRING,
    TRAIT_STRING,
    RESOURCE,
    OPEN_RESOURCE,
    CLOSED_RESOURCE,
    ITERABLE_MIXED_MIXED,
    EMPTY_ARRAY,
    ARRAY_KEY_MIXED,
    CALLABLE,
];

/// Every well-known type, in declaration order.
///
/// The [`TypeBuilder`](crate::ty::builder::TypeBuilder) seeds its union table
/// from this list. A function rather than a table because `static`
/// initializers cannot read other `static`s, and these must be reads of the
/// `TYPE_*` statics for [`Type::ptr_eq`] to hold against them.
#[must_use]
pub fn types() -> [Type<'static>; 23] {
    [
        TYPE_NULL,
        TYPE_NEVER,
        TYPE_VOID,
        TYPE_MIXED,
        TYPE_BOOL,
        TYPE_TRUE,
        TYPE_FALSE,
        TYPE_INT,
        TYPE_FLOAT,
        TYPE_STRING,
        TYPE_OBJECT,
        TYPE_SCALAR,
        TYPE_NUMERIC,
        TYPE_ARRAY_KEY,
        TYPE_CALLABLE,
        TYPE_INT_OR_FLOAT,
        TYPE_INT_OR_STRING,
        TYPE_NULL_OR_SCALAR,
        TYPE_NULL_OR_STRING,
        TYPE_NULL_OR_INT,
        TYPE_NULL_OR_FLOAT,
        TYPE_NULL_OR_OBJECT,
        TYPE_MINUS_ONE_ZERO_ONE,
    ]
}
