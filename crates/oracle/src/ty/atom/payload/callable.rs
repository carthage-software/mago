use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U8Flags;
use mago_span::Span;

use crate::path::Path;
use crate::ty::Type;

/// `callable`, `callable(int): string`, `Closure(int): string`, or a
/// reference to a known function/method/closure.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CallableAtom<'arena> {
    /// Just `callable`, no signature info.
    Any,
    /// `\Closure` with a known signature (e.g. `Closure(int): string`).
    /// Subtype of both `Callable` and the named `\Closure` object; the latter
    /// relationship is enforced at subtype time, not here.
    Closure(&'arena Signature<'arena>),
    /// An anonymous callable signature: `callable(...)`.
    Signature(&'arena Signature<'arena>),
    /// A reference to a known function, method, or closure expression.
    Alias(&'arena CallableAlias<'arena>),
}

/// A callable signature: parameters, return type, optional throws clause.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Signature<'arena> {
    pub parameters: Option<&'arena [Parameter<'arena>]>,
    pub return_type: Type<'arena>,
    pub throws: Option<Type<'arena>>,
    pub flags: U8Flags<SignatureFlag>,
}

/// One parameter inside a [`Signature`]. Carries enough for subtyping
/// (contravariant on type, name match for keyed-arg dispatch) and for
/// diagnostics (default presence, by-reference, variadic).
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Parameter<'arena> {
    pub name: &'arena [u8],
    pub r#type: Type<'arena>,
    pub flags: U8Flags<ParameterFlag>,
}

/// A reference to a known function, method, or closure expression.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CallableAlias<'arena> {
    Function(Path<'arena>),
    Method { class: Path<'arena>, method: &'arena [u8] },
    Closure(Span),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum SignatureFlag {
    IsVariadic = 1 << 0,
    IsPure = 1 << 1,
}

impl From<SignatureFlag> for u8 {
    fn from(flag: SignatureFlag) -> Self {
        flag as u8
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ParameterFlag {
    HasDefault = 1 << 0,
    ByReference = 1 << 1,
    Variadic = 1 << 2,
}

impl From<ParameterFlag> for u8 {
    fn from(flag: ParameterFlag) -> Self {
        flag as u8
    }
}

impl Display for CallableAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CallableAtom::Any => f.write_str("callable"),
            CallableAtom::Signature(signature) => render_signature(signature, false, f),
            CallableAtom::Closure(signature) => render_signature(signature, true, f),
            CallableAtom::Alias(alias) => Display::fmt(alias, f),
        }
    }
}

#[inline]
fn render_signature(signature: &Signature<'_>, is_closure: bool, f: &mut Formatter<'_>) -> FmtResult {
    f.write_str("(")?;
    if signature.flags.contains(SignatureFlag::IsPure) {
        f.write_str("pure-")?;
    }

    f.write_str(if is_closure { "closure(" } else { "callable(" })?;
    if let Some(parameters) = signature.parameters {
        for (index, parameter) in parameters.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }

            if parameter.flags.contains(ParameterFlag::Variadic) {
                f.write_str("...")?;
            }

            Display::fmt(&parameter.r#type, f)?;
            if parameter.flags.contains(ParameterFlag::HasDefault) {
                f.write_str("=")?;
            }
        }
    }

    write!(f, "): {})", signature.return_type)
}

impl Display for CallableAlias<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CallableAlias::Function(name) => write!(f, "Closure<{}>(...)", name),
            CallableAlias::Method { class, method } => {
                write!(f, "Closure<{}::{}>(...)", class, String::from_utf8_lossy(method))
            }
            CallableAlias::Closure(span) => {
                write!(f, "Closure<anonymous@{}::{}>(...)", span.file_id.as_u64(), span.start.offset)
            }
        }
    }
}

impl CopyInto for CallableAtom<'_> {
    type Output<'arena> = CallableAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            CallableAtom::Any => CallableAtom::Any,
            CallableAtom::Closure(signature) => CallableAtom::Closure(copy_ref_into(signature, arena)),
            CallableAtom::Signature(signature) => CallableAtom::Signature(copy_ref_into(signature, arena)),
            CallableAtom::Alias(alias) => CallableAtom::Alias(copy_ref_into(alias, arena)),
        }
    }
}

impl CopyInto for Signature<'_> {
    type Output<'arena> = Signature<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Signature {
            parameters: self.parameters.map(|parameters| copy_slice_into(parameters, arena)),
            return_type: self.return_type.copy_into(arena),
            throws: self.throws.map(|throws| throws.copy_into(arena)),
            flags: self.flags,
        }
    }
}

impl CopyInto for Parameter<'_> {
    type Output<'arena> = Parameter<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Parameter { name: arena.alloc_slice_copy(self.name), r#type: self.r#type.copy_into(arena), flags: self.flags }
    }
}

impl CopyInto for CallableAlias<'_> {
    type Output<'arena> = CallableAlias<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            CallableAlias::Function(name) => CallableAlias::Function(name.copy_into(arena)),
            CallableAlias::Method { class, method } => {
                CallableAlias::Method { class: class.copy_into(arena), method: arena.alloc_slice_copy(method) }
            }
            CallableAlias::Closure(span) => CallableAlias::Closure(span),
        }
    }
}
