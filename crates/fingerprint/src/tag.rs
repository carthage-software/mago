use mago_names::ResolvedNames;
use mago_syntax::cst::ClosingTag;
use mago_syntax::cst::FullOpeningTag;
use mago_syntax::cst::OpeningTag;
use mago_syntax::cst::ShortOpeningTag;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for OpeningTag<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            OpeningTag::Full(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
            OpeningTag::Short(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for FullOpeningTag<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        // Opening tags do not contribute to the fingerprint
    }
}

impl Fingerprintable for ShortOpeningTag {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        // Opening tags do not contribute to the fingerprint
    }
}

impl Fingerprintable for ClosingTag {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        // Closing tags do not contribute to the fingerprint
    }
}
