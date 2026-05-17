use mago_names::ResolvedNames;
use mago_syntax::ast::Clone;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Clone<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "clone".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
