use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Namespace<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "namespace".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NamespaceBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            NamespaceBody::Implicit(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
            NamespaceBody::BraceDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for NamespaceImplicitBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "namespace_implicit".hash(hasher);

        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
