use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Conditional<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "ternary".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);

        match &self.then {
            Some(then) => then.fingerprint_with_hasher(hasher, resolved_names, options),
            None => "short_ternary".hash(hasher),
        }

        self.r#else.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
