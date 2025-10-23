use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Static<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static".hash(hasher);

        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for StaticItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            StaticItem::Abstract(item) => item.fingerprint_with_hasher(hasher, resolved_names, options),
            StaticItem::Concrete(item) => item.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for StaticAbstractItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_abstract".hash(hasher);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticConcreteItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_concrete".hash(hasher);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
