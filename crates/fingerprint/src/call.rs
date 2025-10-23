use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Call<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Call::Function(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::Method(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::NullSafeMethod(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::StaticMethod(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for FunctionCall<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "fn_call".hash(hasher);
        self.function.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MethodCall<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "method_call".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NullSafeMethodCall<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "null_safe_method_call".hash(hasher);

        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticMethodCall<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_method_call".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
