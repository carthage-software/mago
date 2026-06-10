use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::cst::Array;
use mago_syntax::cst::ArrayAccess;
use mago_syntax::cst::ArrayAppend;
use mago_syntax::cst::ArrayElement;
use mago_syntax::cst::KeyValueArrayElement;
use mago_syntax::cst::LegacyArray;
use mago_syntax::cst::List;
use mago_syntax::cst::MissingArrayElement;
use mago_syntax::cst::ValueArrayElement;
use mago_syntax::cst::VariadicArrayElement;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Array<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "array".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for LegacyArray<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "array".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for List<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "list".hash(hasher);
        for element in &self.elements {
            element.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for ArrayElement<'_> {
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
            ArrayElement::KeyValue(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Value(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Variadic(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
            ArrayElement::Missing(element) => element.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for KeyValueArrayElement<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "key_value".hash(hasher);
        self.key.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ValueArrayElement<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "value".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for VariadicArrayElement<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "variadic".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MissingArrayElement {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "missing".hash(hasher);
    }
}

impl Fingerprintable for ArrayAccess<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "array_access".hash(hasher);
        self.array.fingerprint_with_hasher(hasher, resolved_names, options);
        self.index.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ArrayAppend<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "array_append".hash(hasher);
        self.array.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
