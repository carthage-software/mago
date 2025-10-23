use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Use<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        if !options.include_use_statements {
            return;
        }

        "use".hash(hasher);
        self.items.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UseItems<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            UseItems::Sequence(items) => items.fingerprint_with_hasher(hasher, resolved_names, options),
            UseItems::TypedSequence(items) => items.fingerprint_with_hasher(hasher, resolved_names, options),
            UseItems::TypedList(items) => items.fingerprint_with_hasher(hasher, resolved_names, options),
            UseItems::MixedList(items) => items.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for UseType<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        match self {
            UseType::Function(_) => "function".hash(hasher),
            UseType::Const(_) => "const".hash(hasher),
        }
    }
}

impl Fingerprintable for UseItemSequence<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_sequence".hash(hasher);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for TypedUseItemSequence<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_typed_sequence".hash(hasher);
        self.r#type.fingerprint_with_hasher(hasher, resolved_names, options);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for TypedUseItemList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_typed_list".hash(hasher);
        self.r#type.fingerprint_with_hasher(hasher, resolved_names, options);
        self.namespace.fingerprint_with_hasher(hasher, resolved_names, options);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for MixedUseItemList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_mixed_list".hash(hasher);
        self.namespace.fingerprint_with_hasher(hasher, resolved_names, options);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for MaybeTypedUseItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_maybe_typed_item".hash(hasher);
        self.r#type.fingerprint_with_hasher(hasher, resolved_names, options);
        self.item.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UseItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_item".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.alias.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UseItemAlias<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "as".hash(hasher);
        self.identifier.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
