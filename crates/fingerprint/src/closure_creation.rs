use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for ClosureCreation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ClosureCreation::Function(closure) => closure.fingerprint_with_hasher(hasher, resolved_names, options),
            ClosureCreation::Method(closure) => closure.fingerprint_with_hasher(hasher, resolved_names, options),
            ClosureCreation::StaticMethod(closure) => closure.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for FunctionClosureCreation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "fn_closure".hash(hasher);
        self.function.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MethodClosureCreation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "method_closure".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticMethodClosureCreation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_method_closure".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
