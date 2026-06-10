use strum::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum TagVendor {
    Phan,
    PhpStan,
    Psalm,
    Mago,
}

impl TagVendor {
    #[inline]
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            TagVendor::Psalm => "psalm",
            TagVendor::PhpStan => "phpstan",
            TagVendor::Phan => "phan",
            TagVendor::Mago => "mago",
        }
    }

    #[must_use]
    pub fn from_name(name: &[u8]) -> Option<TagVendor> {
        let name = name.strip_prefix(b"@").unwrap_or(name);

        if name.starts_with(b"psalm-") {
            Some(TagVendor::Psalm)
        } else if name.starts_with(b"phpstan-") {
            Some(TagVendor::PhpStan)
        } else if name.starts_with(b"phan-") {
            Some(TagVendor::Phan)
        } else if name.starts_with(b"mago-") {
            Some(TagVendor::Mago)
        } else {
            None
        }
    }
}
