use mago_names::ResolvedNames;
use mago_syntax::cst::Unset;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Unset<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "unset".hash(hasher);

        for value in &self.values {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
