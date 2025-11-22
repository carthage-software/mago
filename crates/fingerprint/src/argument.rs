use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

use std::hash::Hash;

impl Fingerprintable for ArgumentList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        if self.arguments.is_empty() {
            return;
        }

        "args".hash(hasher);
        for argument in self.arguments.iter() {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Argument<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Argument::Positional(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            Argument::Named(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for PositionalArgument<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "pos_arg".hash(hasher);
        self.ellipsis.is_some().hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NamedArgument<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "named_arg".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for PartialArgumentList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        if self.arguments.is_empty() {
            return;
        }

        "partial_args".hash(hasher);
        for argument in self.arguments.iter() {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for PartialArgument<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            PartialArgument::Positional(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialArgument::Named(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialArgument::Placeholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options)
            }
            PartialArgument::VariadicPlaceholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options)
            }
            PartialArgument::NamedPlaceholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options)
            }
        }
    }
}

impl Fingerprintable for PlaceholderArgument {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "placeholder".hash(hasher);
    }
}

impl Fingerprintable for VariadicPlaceholderArgument {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "variadic_placeholder".hash(hasher);
    }
}

impl Fingerprintable for NamedPlaceholderArgument<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "named_placeholder".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
