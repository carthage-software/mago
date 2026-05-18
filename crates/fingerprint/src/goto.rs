use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::Goto;
use mago_syntax::ast::Label;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Goto<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "goto".hash(hasher);
        crate::hash_ascii_lowercase(self.label.value, hasher);
    }
}

impl Fingerprintable for Label<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "label".hash(hasher);
        crate::hash_ascii_lowercase(self.name.value, hasher);
    }
}
