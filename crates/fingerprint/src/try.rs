use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Try<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "try".hash(hasher);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
        for catch_clause in self.catch_clauses.iter() {
            catch_clause.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.finally_clause.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for TryCatchClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "catch".hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for TryFinallyClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "finally".hash(hasher);
        self.block.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
