use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::GenericArgumentList;
use mago_syntax::ast::GenericParameter;
use mago_syntax::ast::GenericParameterList;
use mago_syntax::ast::GenericVariance;
use mago_syntax::ast::Turbofish;

use crate::Fingerprintable;
use crate::FingerprintOptions;

impl Fingerprintable for GenericParameterList<'_> {
    #[inline]
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "generic_param_list".hash(hasher);
        (self.parameters.as_slice().len() as u32).hash(hasher);
        for parameter in self.parameters.iter() {
            parameter.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for GenericParameter<'_> {
    #[inline]
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self.variance {
            Some(GenericVariance::Covariant(_)) => "covariant".hash(hasher),
            Some(GenericVariance::Contravariant(_)) => "contravariant".hash(hasher),
            None => "invariant".hash(hasher),
        }
        self.name.value.hash(hasher);
        if let Some(bound) = &self.bound {
            "bound".hash(hasher);
            bound.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        if let Some(default) = &self.default {
            "default".hash(hasher);
            default.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for GenericArgumentList<'_> {
    #[inline]
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "generic_arg_list".hash(hasher);
        (self.arguments.as_slice().len() as u32).hash(hasher);
        for argument in self.arguments.iter() {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Turbofish<'_> {
    #[inline]
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "turbofish".hash(hasher);
        (self.arguments.as_slice().len() as u32).hash(hasher);
        for argument in self.arguments.iter() {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
