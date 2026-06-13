use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U8Flags;

use crate::name::Name;
use crate::ty::Type;

/// A named object type: `Foo`, `Foo<int, string>`, `static`, `$this`.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectAtom<'arena> {
    pub name: Name<'arena>,
    pub type_arguments: Option<&'arena [Type<'arena>]>,
    pub flags: U8Flags<ObjectFlag>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ObjectFlag {
    IsStatic = 1 << 0,
    IsThis = 1 << 1,
    RemappedParameters = 1 << 2,
}

impl From<ObjectFlag> for u8 {
    fn from(flag: ObjectFlag) -> Self {
        flag as u8
    }
}

impl Display for ObjectAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.flags.contains(ObjectFlag::IsThis) {
            f.write_str("$this(")?;
        }

        f.write_str(&self.name.as_str_lossy())?;
        if let Some(type_arguments) = self.type_arguments {
            f.write_str("<")?;
            for (index, argument) in type_arguments.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }

                Display::fmt(argument, f)?;
            }

            f.write_str(">")?;
        }

        if self.flags.contains(ObjectFlag::IsThis) {
            f.write_str(")")?;
        } else if self.flags.contains(ObjectFlag::IsStatic) {
            f.write_str("&static")?;
        }

        Ok(())
    }
}

impl CopyInto for ObjectAtom<'_> {
    type Output<'arena> = ObjectAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ObjectAtom {
            name: self.name.copy_into(arena),
            type_arguments: self.type_arguments.map(|type_arguments| copy_slice_into(type_arguments, arena)),
            flags: self.flags,
        }
    }
}
