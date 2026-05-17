use mago_names::ResolvedNames;
use mago_syntax::ast::Throw;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Throw<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "throw".hash(hasher);
        self.exception.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
