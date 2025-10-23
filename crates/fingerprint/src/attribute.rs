use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;
use std::hash::Hash;

impl Fingerprintable for AttributeList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        for attribute in self.attributes.iter() {
            attribute.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Attribute<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "attr".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
