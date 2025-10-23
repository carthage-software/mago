use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for OpeningTag<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            OpeningTag::Full(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
            OpeningTag::Short(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for FullOpeningTag<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        // Opening tags do not contribute to the fingerprint
    }
}

impl Fingerprintable for ShortOpeningTag {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        // Opening tags do not contribute to the fingerprint
    }
}

impl Fingerprintable for ClosingTag {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        _hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        // Closing tags do not contribute to the fingerprint
    }
}
