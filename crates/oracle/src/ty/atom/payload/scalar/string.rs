use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_flags::U8Flags;

use crate::name::Name;

/// `string` and its refinement axes: `non-empty-string`, `truthy-string`,
/// `lowercase-string`, `numeric-string`, `callable-string`, literal values,
/// and combinations thereof.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringAtom<'arena> {
    pub literal: StringLiteral<'arena>,
    pub casing: StringCasing,
    pub flags: U8Flags<StringRefinementFlag>,
}

/// Three states for the literal-value field: no literal info, came-from-a-
/// literal-source-but-value-unknown, or exact value.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StringLiteral<'arena> {
    None,
    Unspecified,
    Value(Name<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum StringCasing {
    #[default]
    Unspecified,
    Lowercase,
    Uppercase,
}

/// Boolean refinements that stack: a string can be both `non-empty` and
/// `truthy` and `numeric`, etc.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum StringRefinementFlag {
    Numeric = 1 << 0,
    Truthy = 1 << 1,
    NonEmpty = 1 << 2,
    Callable = 1 << 3,
}

impl From<StringRefinementFlag> for u8 {
    fn from(flag: StringRefinementFlag) -> Self {
        flag as u8
    }
}

impl Display for StringAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let label = match self.literal {
            StringLiteral::Value(value) => return write!(f, "string('{}')", value.as_str_lossy()),
            StringLiteral::Unspecified => label_literal_string(self),
            StringLiteral::None => label_general_string(self),
        };

        f.write_str(label)
    }
}

#[inline]
fn label_literal_string(atom: &StringAtom<'_>) -> &'static str {
    if atom.flags.contains(StringRefinementFlag::Truthy) {
        if atom.flags.contains(StringRefinementFlag::Numeric) {
            "truthy-numeric-literal-string"
        } else {
            match atom.casing {
                StringCasing::Lowercase => "truthy-lowercase-literal-string",
                StringCasing::Uppercase => "truthy-uppercase-literal-string",
                StringCasing::Unspecified => "truthy-literal-string",
            }
        }
    } else if atom.flags.contains(StringRefinementFlag::Numeric) {
        "numeric-literal-string"
    } else if atom.flags.contains(StringRefinementFlag::NonEmpty) {
        match atom.casing {
            StringCasing::Lowercase => "lowercase-non-empty-literal-string",
            StringCasing::Uppercase => "uppercase-non-empty-literal-string",
            StringCasing::Unspecified => "non-empty-literal-string",
        }
    } else {
        match atom.casing {
            StringCasing::Lowercase => "lowercase-literal-string",
            StringCasing::Uppercase => "uppercase-literal-string",
            StringCasing::Unspecified => "literal-string",
        }
    }
}

#[inline]
fn label_general_string(atom: &StringAtom<'_>) -> &'static str {
    if atom.flags.contains(StringRefinementFlag::Callable) {
        return match atom.casing {
            StringCasing::Lowercase => "lowercase-callable-string",
            StringCasing::Uppercase => "uppercase-callable-string",
            StringCasing::Unspecified => "callable-string",
        };
    }

    if atom.flags.contains(StringRefinementFlag::Truthy) {
        if atom.flags.contains(StringRefinementFlag::Numeric) {
            return "truthy-numeric-string";
        }

        return match atom.casing {
            StringCasing::Lowercase => "truthy-lowercase-string",
            StringCasing::Uppercase => "truthy-uppercase-string",
            StringCasing::Unspecified => "truthy-string",
        };
    }

    if atom.flags.contains(StringRefinementFlag::Numeric) {
        return "numeric-string";
    }

    if atom.flags.contains(StringRefinementFlag::NonEmpty) {
        return match atom.casing {
            StringCasing::Lowercase => "lowercase-non-empty-string",
            StringCasing::Uppercase => "uppercase-non-empty-string",
            StringCasing::Unspecified => "non-empty-string",
        };
    }

    match atom.casing {
        StringCasing::Lowercase => "lowercase-string",
        StringCasing::Uppercase => "uppercase-string",
        StringCasing::Unspecified => "string",
    }
}

impl CopyInto for StringAtom<'_> {
    type Output<'arena> = StringAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        StringAtom { literal: self.literal.copy_into(arena), casing: self.casing, flags: self.flags }
    }
}

impl CopyInto for StringLiteral<'_> {
    type Output<'arena> = StringLiteral<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            StringLiteral::None => StringLiteral::None,
            StringLiteral::Unspecified => StringLiteral::Unspecified,
            StringLiteral::Value(value) => StringLiteral::Value(value.copy_into(arena)),
        }
    }
}
