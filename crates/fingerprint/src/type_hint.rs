use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Hint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Hint::Identifier(id) => {
                id.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            Hint::Parenthesized(p) => p.fingerprint_with_hasher(hasher, resolved_names, options),
            Hint::Nullable(n) => n.fingerprint_with_hasher(hasher, resolved_names, options),
            Hint::Union(u) => u.fingerprint_with_hasher(hasher, resolved_names, options),
            Hint::Intersection(i) => i.fingerprint_with_hasher(hasher, resolved_names, options),
            Hint::Null(_) => "null".hash(hasher),
            Hint::True(_) => "true".hash(hasher),
            Hint::False(_) => "false".hash(hasher),
            Hint::Array(_) => "array".hash(hasher),
            Hint::Callable(_) => "callable".hash(hasher),
            Hint::Static(_) => "static".hash(hasher),
            Hint::Self_(_) => "self".hash(hasher),
            Hint::Parent(_) => "parent".hash(hasher),
            Hint::Void(_) => "void".hash(hasher),
            Hint::Never(_) => "never".hash(hasher),
            Hint::Float(_) => "float".hash(hasher),
            Hint::Bool(_) => "bool".hash(hasher),
            Hint::Integer(_) => "int".hash(hasher),
            Hint::String(_) => "string".hash(hasher),
            Hint::Object(_) => "object".hash(hasher),
            Hint::Mixed(_) => "mixed".hash(hasher),
            Hint::Iterable(_) => "iterable".hash(hasher),
        }
    }
}

impl Fingerprintable for ParenthesizedHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NullableHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "nullable".hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UnionHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "union".hash(hasher);
        self.left.fingerprint_with_hasher(hasher, resolved_names, options);
        self.right.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IntersectionHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "intersection".hash(hasher);
        self.left.fingerprint_with_hasher(hasher, resolved_names, options);
        self.right.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
