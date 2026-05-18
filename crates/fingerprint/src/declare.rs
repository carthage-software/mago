use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::Declare;
use mago_syntax::ast::DeclareBody;
use mago_syntax::ast::DeclareColonDelimitedBody;
use mago_syntax::ast::DeclareItem;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Declare<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "declare".hash(hasher);

        for item in &self.items {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }

        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for DeclareItem<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "declare_item".hash(hasher);
        crate::hash_ascii_lowercase(self.name.value, hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for DeclareBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
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
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "declare_colon_delimited".hash(hasher);

        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
