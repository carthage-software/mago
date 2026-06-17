#[cfg(feature = "serde")]
use serde::Deserialize;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ReadWriteVisibility {
    pub read: Visibility,
    pub write: Visibility,
}

impl Visibility {
    #[inline]
    #[must_use]
    pub const fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    #[inline]
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Visibility::Protected)
    }

    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Visibility::Private)
    }

    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Protected => "protected",
            Visibility::Private => "private",
        }
    }
}

impl ReadWriteVisibility {
    #[inline]
    #[must_use]
    pub const fn new(read: Visibility, write: Visibility) -> Self {
        Self { read, write }
    }

    #[inline]
    #[must_use]
    pub const fn is_read_public(&self) -> bool {
        self.read.is_public()
    }

    #[inline]
    #[must_use]
    pub const fn is_read_protected(&self) -> bool {
        self.read.is_protected()
    }

    #[inline]
    #[must_use]
    pub const fn is_read_private(&self) -> bool {
        self.read.is_private()
    }

    #[inline]
    #[must_use]
    pub const fn is_write_public(&self) -> bool {
        self.write.is_public()
    }

    #[inline]
    #[must_use]
    pub const fn is_write_protected(&self) -> bool {
        self.write.is_protected()
    }

    #[inline]
    #[must_use]
    pub const fn is_write_private(&self) -> bool {
        self.write.is_private()
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
