use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Echo<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "echo".hash(hasher);

        for value in self.values.iter() {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for EchoTag<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "echo_tag".hash(hasher);

        for value in self.values.iter() {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
