use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

use crate::path::Path;
use crate::ty::Type;

/// `Foo`, `Foo<int>`: an unresolved symbol reference. Expansion resolves it
/// against the [`SymbolTable`](crate::symbol) into a structural type.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolReferenceAtom<'arena> {
    pub name: Path<'arena>,
    pub type_arguments: Option<&'arena [Type<'arena>]>,
}

/// `Foo::CONST`, `Foo::*`, `Foo::PREFIX_*`: a class-like constant reference.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberReferenceAtom<'arena> {
    pub class_like_name: Path<'arena>,
    pub selector: NameSelector<'arena>,
}

/// A reference to a global constant, optionally via wildcard selector.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalReferenceAtom<'arena> {
    pub selector: NameSelector<'arena>,
}

/// How a member or global reference picks one or more matching names.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NameSelector<'arena> {
    Identifier(&'arena [u8]),
    StartsWith(&'arena [u8]),
    EndsWith(&'arena [u8]),
    Contains(&'arena [u8]),
    Wildcard,
}

impl Display for NameSelector<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            NameSelector::Identifier(name) => f.write_str(&String::from_utf8_lossy(name)),
            NameSelector::StartsWith(name) => write!(f, "{}*", String::from_utf8_lossy(name)),
            NameSelector::EndsWith(name) => write!(f, "*{}", String::from_utf8_lossy(name)),
            NameSelector::Contains(name) => write!(f, "*{}*", String::from_utf8_lossy(name)),
            NameSelector::Wildcard => f.write_str("*"),
        }
    }
}

impl Display for SymbolReferenceAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.name.fmt(f)?;
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

        Ok(())
    }
}

impl Display for MemberReferenceAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}::{}", self.class_like_name, self.selector)
    }
}

impl Display for GlobalReferenceAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.selector, f)
    }
}

impl CopyInto for NameSelector<'_> {
    type Output<'arena> = NameSelector<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            NameSelector::Identifier(name) => NameSelector::Identifier(arena.alloc_slice_copy(name)),
            NameSelector::StartsWith(name) => NameSelector::StartsWith(arena.alloc_slice_copy(name)),
            NameSelector::EndsWith(name) => NameSelector::EndsWith(arena.alloc_slice_copy(name)),
            NameSelector::Contains(name) => NameSelector::Contains(arena.alloc_slice_copy(name)),
            NameSelector::Wildcard => NameSelector::Wildcard,
        }
    }
}

impl CopyInto for SymbolReferenceAtom<'_> {
    type Output<'arena> = SymbolReferenceAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        SymbolReferenceAtom {
            name: self.name.copy_into(arena),
            type_arguments: self.type_arguments.map(|type_arguments| copy_slice_into(type_arguments, arena)),
        }
    }
}

impl CopyInto for MemberReferenceAtom<'_> {
    type Output<'arena> = MemberReferenceAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MemberReferenceAtom {
            class_like_name: self.class_like_name.copy_into(arena),
            selector: self.selector.copy_into(arena),
        }
    }
}

impl CopyInto for GlobalReferenceAtom<'_> {
    type Output<'arena> = GlobalReferenceAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        GlobalReferenceAtom { selector: self.selector.copy_into(arena) }
    }
}
