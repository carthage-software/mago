use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::Array;
use mago_syntax::ast::ArrayAccess;
use mago_syntax::ast::ArrayAppend;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::KeyValueArrayElement;
use mago_syntax::ast::LegacyArray;
use mago_syntax::ast::List;
use mago_syntax::ast::MissingArrayElement;
use mago_syntax::ast::ValueArrayElement;
use mago_syntax::ast::VariadicArrayElement;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Array<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "array".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for LegacyArray<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "array".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for List<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "list".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for ArrayElement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ArrayElement::KeyValue(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Value(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Variadic(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Missing(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for KeyValueArrayElement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "key_value".hash(hasher);
        self.key.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ValueArrayElement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "value".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for VariadicArrayElement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "variadic".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MissingArrayElement {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "missing".hash(hasher);
    }
}

impl Fingerprintable for ArrayAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "array_access".hash(hasher);
        self.array.fingerprint_with_hasher(hasher, resolved_names, options);
        self.index.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ArrayAppend<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "array_append".hash(hasher);
        self.array.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
