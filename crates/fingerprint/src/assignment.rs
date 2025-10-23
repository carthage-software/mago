use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Assignment<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "assignment".hash(hasher);
        self.lhs.fingerprint_with_hasher(hasher, resolved_names, options);
        self.operator.fingerprint_with_hasher(hasher, resolved_names, options);
        self.rhs.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for AssignmentOperator {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        self.as_str().hash(hasher);
    }
}
