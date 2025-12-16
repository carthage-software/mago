use mago_names::ResolvedNames;
use mago_syntax::ast::FunctionPartialApplication;
use mago_syntax::ast::MethodPartialApplication;
use mago_syntax::ast::PartialApplication;
use mago_syntax::ast::StaticMethodPartialApplication;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for PartialApplication<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            PartialApplication::Function(partial) => partial.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialApplication::Method(partial) => partial.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialApplication::StaticMethod(partial) => {
                partial.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for FunctionPartialApplication<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "fn_partial".hash(hasher);
        self.function.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MethodPartialApplication<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "method_partial".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticMethodPartialApplication<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_method_partial".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method.fingerprint_with_hasher(hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
