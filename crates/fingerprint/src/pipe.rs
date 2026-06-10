use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::cst::Pipe;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Pipe<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "|>".hash(hasher);
        self.input.fingerprint_with_hasher(hasher, resolved_names, options);
        self.callable.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
