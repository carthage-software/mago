use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::num::NonZeroU32;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U8Flags;

use crate::path::Path;
use crate::ty::Type;

/// A literal key in a keyed-array shape.
///
/// `Const` carries an unresolved `Class::CONSTANT` reference; resolution
/// happens during expansion.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArrayKey<'arena> {
    Int(i64),
    String(&'arena [u8]),
    Const { class: Path<'arena>, name: &'arena [u8] },
}

/// One entry in a keyed-array shape's known items list.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KnownItem<'arena> {
    pub key: ArrayKey<'arena>,
    pub value: Type<'arena>,
    pub optional: bool,
}

/// One entry in a list shape's known elements list.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KnownElement<'arena> {
    pub index: u32,
    pub value: Type<'arena>,
    pub optional: bool,
}

/// `array<K, V>`, `array{a: int, ...}`, `array{}`.
///
/// `key_param` / `value_param` describe the rest type when present.
/// `known_items` is a sorted list of fixed entries.
///
/// "Sealed" is the absence of a rest type: `key_param` and `value_param` both
/// `None` means the shape admits no extra entries beyond `known_items`. There
/// is intentionally no separate `sealed` flag.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArrayAtom<'arena> {
    pub key_param: Option<Type<'arena>>,
    pub value_param: Option<Type<'arena>>,
    pub known_items: Option<&'arena [KnownItem<'arena>]>,
    pub flags: U8Flags<ArrayFlag>,
}

impl ArrayAtom<'_> {
    /// `true` iff this shape admits no entries beyond its known items.
    #[inline]
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        self.key_param.is_none() && self.value_param.is_none()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ArrayFlag {
    NonEmpty = 1 << 0,
}

impl From<ArrayFlag> for u8 {
    fn from(flag: ArrayFlag) -> Self {
        flag as u8
    }
}

/// `list<T>`, `non-empty-list<T>`, `list{0: int, 1: string, ...}`.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ListAtom<'arena> {
    pub element_type: Type<'arena>,
    pub known_elements: Option<&'arena [KnownElement<'arena>]>,
    pub known_count: Option<NonZeroU32>,
    pub flags: U8Flags<ListFlag>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ListFlag {
    NonEmpty = 1 << 0,
}

impl From<ListFlag> for u8 {
    fn from(flag: ListFlag) -> Self {
        flag as u8
    }
}

impl Display for ArrayKey<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ArrayKey::Int(value) => write!(f, "{value}"),
            ArrayKey::String(name) => write!(f, "'{}'", String::from_utf8_lossy(name)),
            ArrayKey::Const { class, name } => {
                write!(f, "{}::{}", class, String::from_utf8_lossy(name))
            }
        }
    }
}

impl Display for ArrayAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(known_items) = self.known_items {
            f.write_str("array{")?;
            let mut first = true;
            for entry in known_items {
                if !first {
                    f.write_str(", ")?;
                }

                first = false;
                Display::fmt(&entry.key, f)?;
                if entry.optional {
                    f.write_str("?")?;
                }

                f.write_str(": ")?;
                Display::fmt(&entry.value, f)?;
            }

            if let (Some(key), Some(value)) = (self.key_param, self.value_param) {
                if !first {
                    f.write_str(", ")?;
                }

                write!(f, "...<{key}, {value}>")?;
            }

            f.write_str("}")?;
        } else if let (Some(key), Some(value)) = (self.key_param, self.value_param) {
            let head = if self.flags.contains(ArrayFlag::NonEmpty) { "non-empty-array" } else { "array" };
            write!(f, "{head}<{key}, {value}>")?;
        } else {
            f.write_str("array{}")?;
        }

        Ok(())
    }
}

impl Display for ListAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(known_elements) = self.known_elements {
            f.write_str("list{")?;
            let mut first = true;
            for entry in known_elements {
                if !first {
                    f.write_str(", ")?;
                }

                first = false;
                write!(f, "{}", entry.index)?;
                if entry.optional {
                    f.write_str("?")?;
                }

                write!(f, ": {}", entry.value)?;
            }

            f.write_str("}")?;
        } else {
            let head = if self.flags.contains(ListFlag::NonEmpty) { "non-empty-list" } else { "list" };
            write!(f, "{head}<{}>", self.element_type)?;
        }

        Ok(())
    }
}

impl CopyInto for ArrayKey<'_> {
    type Output<'arena> = ArrayKey<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            ArrayKey::Int(value) => ArrayKey::Int(value),
            ArrayKey::String(name) => ArrayKey::String(arena.alloc_slice_copy(name)),
            ArrayKey::Const { class, name } => {
                ArrayKey::Const { class: class.copy_into(arena), name: arena.alloc_slice_copy(name) }
            }
        }
    }
}

impl CopyInto for KnownItem<'_> {
    type Output<'arena> = KnownItem<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        KnownItem { key: self.key.copy_into(arena), value: self.value.copy_into(arena), optional: self.optional }
    }
}

impl CopyInto for KnownElement<'_> {
    type Output<'arena> = KnownElement<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        KnownElement { index: self.index, value: self.value.copy_into(arena), optional: self.optional }
    }
}

impl CopyInto for ArrayAtom<'_> {
    type Output<'arena> = ArrayAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ArrayAtom {
            key_param: self.key_param.map(|key_param| key_param.copy_into(arena)),
            value_param: self.value_param.map(|value_param| value_param.copy_into(arena)),
            known_items: self.known_items.map(|known_items| copy_slice_into(known_items, arena)),
            flags: self.flags,
        }
    }
}

impl CopyInto for ListAtom<'_> {
    type Output<'arena> = ListAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ListAtom {
            element_type: self.element_type.copy_into(arena),
            known_elements: self.known_elements.map(|known_elements| copy_slice_into(known_elements, arena)),
            known_count: self.known_count,
            flags: self.flags,
        }
    }
}
