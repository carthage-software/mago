use mago_names::ResolvedNames;
use mago_syntax::ast::Assignment;
use mago_syntax::ast::AssignmentOperator;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Assignment<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "assignment".hash(hasher);
        self.lhs.fingerprint_with_hasher(hasher, resolved_names, options);
        self.operator.fingerprint_with_hasher(hasher, resolved_names, options);
        self.rhs.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for AssignmentOperator {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        self.as_str().hash(hasher);
    }
}
