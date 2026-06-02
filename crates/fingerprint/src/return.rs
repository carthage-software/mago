use mago_names::ResolvedNames;
use mago_syntax::cst::Return;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Return<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "return".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
