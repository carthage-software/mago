use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

use crate::symbol::class_like::part::visibility::Visibility;
use crate::ty::Type;

/// A type derived from another type: `key-of<T>`, `value-of<T>`, `T[K]`,
/// `int-mask<...>`, `template-type<...>`, `new<T>`, `properties-of<T>`.
/// Resolved during expansion.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DerivedAtom<'arena> {
    /// `key-of<T>`: the key type of an array-like or iterable `T`.
    KeyOf(Type<'arena>),
    /// `value-of<T>`: the value type. For `BackedEnum` subclasses, the
    /// backing values.
    ValueOf(Type<'arena>),
    /// `properties-of<T>`, `public-properties-of<T>`, etc.: for each property
    /// of class `T` (filtered by `visibility`), produce
    /// `array{prop_name: prop_type}`.
    PropertiesOf { target: Type<'arena>, visibility: Option<Visibility> },
    /// `T[K]`: element access. For `array{a: int}`, `T['a']` resolves to
    /// `int`.
    IndexAccess { target: Type<'arena>, index: Type<'arena> },
    /// `int-mask<A::FLAG_FOO, A::FLAG_BAR>`: the set of integers formable by
    /// bitwise-OR-ing some subset of the listed literal-int values.
    IntMask(&'arena [Type<'arena>]),
    /// `int-mask-of<A::FLAG_*>`: `IntMask` over all members of a constant-
    /// wildcard family.
    IntMaskOf(Type<'arena>),
    /// `template-type<$object, ClassName, T>`: given a value `$object` of
    /// some specialized class, look up the bound type for template `T` of
    /// `ClassName`.
    TemplateType { object: Type<'arena>, class_name: Type<'arena>, template_name: Type<'arena> },
    /// `new<T>`: if `T` is `class-string<Foo>` or a literal class-string,
    /// produce `Foo` (the instance type).
    New(Type<'arena>),
}

impl Display for DerivedAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DerivedAtom::KeyOf(target) => write!(f, "key-of<{target}>"),
            DerivedAtom::ValueOf(target) => write!(f, "value-of<{target}>"),
            DerivedAtom::PropertiesOf { target, visibility } => match visibility {
                Some(visibility) => write!(f, "{}-properties-of<{target}>", visibility.as_str()),
                None => write!(f, "properties-of<{target}>"),
            },
            DerivedAtom::IndexAccess { target, index } => write!(f, "{target}[{index}]"),
            DerivedAtom::IntMask(members) => {
                f.write_str("int-mask<")?;
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }

                    Display::fmt(member, f)?;
                }

                f.write_str(">")
            }
            DerivedAtom::IntMaskOf(target) => write!(f, "int-mask-of<{target}>"),
            DerivedAtom::TemplateType { object, class_name, template_name } => {
                write!(f, "template-type<{object}, {class_name}, {template_name}>")
            }
            DerivedAtom::New(target) => write!(f, "new<{target}>"),
        }
    }
}

impl CopyInto for DerivedAtom<'_> {
    type Output<'arena> = DerivedAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            DerivedAtom::KeyOf(target) => DerivedAtom::KeyOf(target.copy_into(arena)),
            DerivedAtom::ValueOf(target) => DerivedAtom::ValueOf(target.copy_into(arena)),
            DerivedAtom::PropertiesOf { target, visibility } => {
                DerivedAtom::PropertiesOf { target: target.copy_into(arena), visibility }
            }
            DerivedAtom::IndexAccess { target, index } => {
                DerivedAtom::IndexAccess { target: target.copy_into(arena), index: index.copy_into(arena) }
            }
            DerivedAtom::IntMask(members) => DerivedAtom::IntMask(copy_slice_into(members, arena)),
            DerivedAtom::IntMaskOf(target) => DerivedAtom::IntMaskOf(target.copy_into(arena)),
            DerivedAtom::TemplateType { object, class_name, template_name } => DerivedAtom::TemplateType {
                object: object.copy_into(arena),
                class_name: class_name.copy_into(arena),
                template_name: template_name.copy_into(arena),
            },
            DerivedAtom::New(target) => DerivedAtom::New(target.copy_into(arena)),
        }
    }
}
