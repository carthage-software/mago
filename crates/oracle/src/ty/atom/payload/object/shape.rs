use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U8Flags;

use crate::ty::Type;

/// `object{name: string, ...}`: a structural object shape.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectShapeAtom<'arena> {
    pub known_properties: Option<&'arena [KnownProperty<'arena>]>,
    pub flags: U8Flags<ObjectShapeFlag>,
}

/// One entry in an object shape's known-properties list.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KnownProperty<'arena> {
    pub name: &'arena [u8],
    pub value: Type<'arena>,
    pub optional: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ObjectShapeFlag {
    Sealed = 1 << 0,
}

impl From<ObjectShapeFlag> for u8 {
    fn from(flag: ObjectShapeFlag) -> Self {
        flag as u8
    }
}

impl Display for ObjectShapeAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("object{")?;
        let mut first = true;
        if let Some(known_properties) = self.known_properties {
            for entry in known_properties {
                if !first {
                    f.write_str(", ")?;
                }

                first = false;
                f.write_str(&String::from_utf8_lossy(entry.name))?;
                if entry.optional {
                    f.write_str("?")?;
                }

                f.write_str(": ")?;
                Display::fmt(&entry.value, f)?;
            }
        }

        if !self.flags.contains(ObjectShapeFlag::Sealed) {
            if !first {
                f.write_str(", ")?;
            }

            f.write_str("...")?;
        }

        f.write_str("}")
    }
}

impl CopyInto for ObjectShapeAtom<'_> {
    type Output<'arena> = ObjectShapeAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ObjectShapeAtom {
            known_properties: self.known_properties.map(|known_properties| copy_slice_into(known_properties, arena)),
            flags: self.flags,
        }
    }
}

impl CopyInto for KnownProperty<'_> {
    type Output<'arena> = KnownProperty<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        KnownProperty {
            name: arena.alloc_slice_copy(self.name),
            value: self.value.copy_into(arena),
            optional: self.optional,
        }
    }
}
