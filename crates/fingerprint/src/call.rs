use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::Call;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::NullSafeMethodCall;
use mago_syntax::ast::StaticMethodCall;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Call<'_> {
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
            Call::Function(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::Method(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::NullSafeMethod(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
            Call::StaticMethod(call) => call.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for FunctionCall<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "fn_call".hash(hasher);
        self.function.fingerprint_with_hasher(hasher, resolved_names, options);
        self.turbofish.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MethodCall<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "method_call".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.turbofish.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NullSafeMethodCall<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "null_safe_method_call".hash(hasher);

        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.turbofish.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticMethodCall<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "static_method_call".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.turbofish.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
