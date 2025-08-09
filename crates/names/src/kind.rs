use serde::Deserialize;
use serde::Serialize;
use strum::Display;

/// Categorizes the type of PHP name being referenced, primarily for alias
/// resolution and determining naming rules (like case sensitivity).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum NameKind {
    /// Represents class, interface, trait, enum, or namespace names.
    /// Typically imported using `use Some\Name;`.
    Default,
    /// Represents function names.
    /// Typically imported using `use function Some\funcName;`.
    Function,
    /// Represents constant names.
    /// Typically imported using `use const Some\CONST_NAME;`.
    Constant,
}

impl NameKind {
    /// Checks if the kind is `NameKind::Default`.
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, NameKind::Default)
    }

    /// Checks if the kind is `NameKind::Function`.
    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, NameKind::Function)
    }

    /// Checks if the kind is `NameKind::Constant`.
    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(self, NameKind::Constant)
    }

    /// Checks if names of this kind are treated case-sensitively in PHP.
    ///
    /// Currently, only constants (`NameKind::Constant`) are case-sensitive.
    /// Class, interface, trait, namespace, and function names are generally
    /// resolved case-insensitively.
    #[inline]
    pub const fn is_case_sensitive(&self) -> bool {
        // Renamed from is_case_sensitive
        // Only constants are case-sensitive in PHP name resolution
        self.is_constant()
    }
}
