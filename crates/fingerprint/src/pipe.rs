use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Pipe<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "|>".hash(hasher);
        self.input.fingerprint_with_hasher(hasher, resolved_names, options);
        self.callable.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
