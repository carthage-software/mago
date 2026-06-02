use mago_names::ResolvedNames;
use mago_syntax::cst::HaltCompiler;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for HaltCompiler<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "halt_compiler".hash(hasher);
    }
}
