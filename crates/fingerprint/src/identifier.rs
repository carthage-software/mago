use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Identifier<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Identifier::Local(ident) => ident.fingerprint_with_hasher(hasher, resolved_names, options),
            Identifier::Qualified(ident) => ident.fingerprint_with_hasher(hasher, resolved_names, options),
            Identifier::FullyQualified(ident) => ident.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for LocalIdentifier<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        if let Some(name) = resolved_names.resolve(self) {
            mago_atom::ascii_lowercase_atom(name).hash(hasher);
        } else {
            self.value.hash(hasher);
        }
    }
}

impl Fingerprintable for QualifiedIdentifier<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        if let Some(name) = resolved_names.resolve(self) {
            mago_atom::ascii_lowercase_atom(name).hash(hasher);
        } else {
            self.value.hash(hasher);
        }
    }
}

impl Fingerprintable for FullyQualifiedIdentifier<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        if let Some(name) = resolved_names.resolve(self) {
            mago_atom::ascii_lowercase_atom(name).hash(hasher);
        } else {
            self.value.hash(hasher);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::fingerprint_code;

    #[test]
    fn test_identifier_case_insensitive_without_resolution() {
        let fp1 = fingerprint_code("<?php class Foo {}");
        let fp2 = fingerprint_code("<?php class FOO {}");
        let fp3 = fingerprint_code("<?php class foo {}");

        assert_eq!(fp1, fp2);
        assert_eq!(fp1, fp3);
    }

    #[test]
    fn test_different_identifiers() {
        let foo_fp = fingerprint_code("<?php class Foo {}");
        let bar_fp = fingerprint_code("<?php class Bar {}");

        assert_ne!(foo_fp, bar_fp);
    }

    #[test]
    fn test_function_names_case_insensitive() {
        let fp1 = fingerprint_code("<?php function myFunc() {}");
        let fp2 = fingerprint_code("<?php function MyFunc() {}");
        let fp3 = fingerprint_code("<?php function MYFUNC() {}");

        assert_eq!(fp1, fp2);
        assert_eq!(fp1, fp3);
    }
}
