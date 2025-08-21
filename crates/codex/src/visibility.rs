use serde::Deserialize;
use serde::Serialize;

use mago_syntax::ast::Modifier;

/// Represents the visibility level of class members (properties, methods, constants) in PHP.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
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
    pub const fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    /// Checks if the visibility level is `Protected`.
    #[inline]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Visibility::Protected)
    }

    /// Checks if the visibility level is `Private`.
    #[inline]
    pub const fn is_private(&self) -> bool {
        matches!(self, Visibility::Private)
    }

    /// Returns the visibility level as a static string.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Protected => "protected",
            Visibility::Private => "private",
        }
    }
}

/// Formats the visibility level as the corresponding lowercase PHP keyword.
impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
