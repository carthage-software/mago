use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Instantiation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "new".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
