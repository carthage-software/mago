use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Declare<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "declare".hash(hasher);

        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }

        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for DeclareItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "declare_item".hash(hasher);
        mago_atom::ascii_lowercase_atom(self.name.value).hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for DeclareBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            DeclareBody::Statement(statement) => {
                "declare_statement".hash(hasher);
                statement.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            DeclareBody::ColonDelimited(body) => {
                body.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for DeclareColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "declare_colon_delimited".hash(hasher);

        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
