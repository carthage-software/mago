use mago_syntax::ast::Modifier;

/// Represents the visibility level of class members (properties, methods, constants) in PHP.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Visibility {
    /// Represents `public` visibility. Accessible from anywhere.
    /// This is the default visibility in PHP if none is specified.
    #[default]
    Public,
    /// Represents `protected` visibility. Accessible only within the declaring class,
    /// its parent classes, and inheriting classes.
    Protected,
    /// Represents `private` visibility. Accessible only within the declaring class.
    Private,
}

impl Visibility {
    /// Checks if the visibility level is `Public`.
    #[inline]
    #[must_use]
    pub const fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    /// Checks if the visibility level is `Protected`.
    #[inline]
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Visibility::Protected)
    }

    /// Checks if the visibility level is `Private`.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Visibility::Private)
    }

    /// Returns the visibility level as static bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &'static [u8] {
        match self {
            Visibility::Public => b"public",
            Visibility::Protected => b"protected",
            Visibility::Private => b"private",
        }
    }
}

/// Formats the visibility level as the corresponding lowercase PHP keyword.
impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", mago_bytes::BytesDisplay(self.as_bytes()))
    }
}

/// Attempts to convert an AST `Modifier` node into a `Visibility` level.
impl TryFrom<&Modifier<'_>> for Visibility {
    type Error = ();

    fn try_from(value: &Modifier<'_>) -> Result<Self, Self::Error> {
        match value {
            Modifier::Public(_) | Modifier::PublicSet(_) => Ok(Visibility::Public),
            Modifier::Protected(_) | Modifier::ProtectedSet(_) => Ok(Visibility::Protected),
            Modifier::Private(_) | Modifier::PrivateSet(_) => Ok(Visibility::Private),
            _ => Err(()),
        }
    }
}
