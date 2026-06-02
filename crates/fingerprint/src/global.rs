use mago_names::ResolvedNames;
use mago_syntax::cst::Global;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Global<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "global".hash(hasher);

        for variable in &self.variables {
            variable.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
