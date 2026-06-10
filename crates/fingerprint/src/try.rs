use mago_names::ResolvedNames;
use mago_syntax::cst::Try;
use mago_syntax::cst::TryCatchClause;
use mago_syntax::cst::TryFinallyClause;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Try<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "try".hash(hasher);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
        for catch_clause in &self.catch_clauses {
            catch_clause.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.finally_clause.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for TryCatchClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "catch".hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for TryFinallyClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "finally".hash(hasher);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
