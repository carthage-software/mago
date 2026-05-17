use mago_names::ResolvedNames;
use mago_syntax::ast::Conditional;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Conditional<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "ternary".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);

        match &self.then {
            Some(then) => then.fingerprint_with_hasher(hasher, resolved_names, options),
            None => "short_ternary".hash(hasher),
        }

        self.r#else.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
