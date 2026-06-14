use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

use crate::name::Name;
use crate::ty::Type;

/// `Foo`, `Foo<int>`: an unresolved symbol reference. Expansion resolves it
/// against the [`World`](crate::world) into a structural type.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolReferenceAtom<'arena> {
    pub name: Name<'arena>,
    pub type_arguments: Option<&'arena [Type<'arena>]>,
}

/// `Foo::CONST`, `Foo::*`, `Foo::PREFIX_*`: a class-like constant reference.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberReferenceAtom<'arena> {
    pub class_like_name: Name<'arena>,
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
    Identifier(Name<'arena>),
    StartsWith(Name<'arena>),
    EndsWith(Name<'arena>),
    Contains(Name<'arena>),
    Wildcard,
}

impl Display for NameSelector<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            NameSelector::Identifier(name) => f.write_str(&name.as_str_lossy()),
            NameSelector::StartsWith(name) => write!(f, "{}*", name.as_str_lossy()),
            NameSelector::EndsWith(name) => write!(f, "*{}", name.as_str_lossy()),
            NameSelector::Contains(name) => write!(f, "*{}*", name.as_str_lossy()),
            NameSelector::Wildcard => f.write_str("*"),
        }
    }
}

impl Display for SymbolReferenceAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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

        Ok(())
    }
}

impl Display for MemberReferenceAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}::{}", self.class_like_name.as_str_lossy(), self.selector)
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
            NameSelector::Identifier(name) => NameSelector::Identifier(name.copy_into(arena)),
            NameSelector::StartsWith(name) => NameSelector::StartsWith(name.copy_into(arena)),
            NameSelector::EndsWith(name) => NameSelector::EndsWith(name.copy_into(arena)),
            NameSelector::Contains(name) => NameSelector::Contains(name.copy_into(arena)),
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
