use mago_names::ResolvedNames;
use mago_syntax::cst::Argument;
use mago_syntax::cst::ArgumentList;
use mago_syntax::cst::NamedArgument;
use mago_syntax::cst::NamedPlaceholderArgument;
use mago_syntax::cst::PartialArgument;
use mago_syntax::cst::PartialArgumentList;
use mago_syntax::cst::PlaceholderArgument;
use mago_syntax::cst::PositionalArgument;
use mago_syntax::cst::VariadicPlaceholderArgument;

use crate::FingerprintOptions;
use crate::Fingerprintable;

use std::hash::Hash;

impl Fingerprintable for ArgumentList<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        if self.arguments.is_empty() {
            return;
        }

        "args".hash(hasher);
        for argument in &self.arguments {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Argument<'_> {
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
            Argument::Positional(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            Argument::Named(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for PositionalArgument<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "pos_arg".hash(hasher);
        self.ellipsis.is_some().hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NamedArgument<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "named_arg".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for PartialArgumentList<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        if self.arguments.is_empty() {
            return;
        }

        "partial_args".hash(hasher);
        for argument in &self.arguments {
            argument.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for PartialArgument<'_> {
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
            PartialArgument::Positional(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialArgument::Named(arg) => arg.fingerprint_with_hasher(hasher, resolved_names, options),
            PartialArgument::Placeholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            PartialArgument::VariadicPlaceholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            PartialArgument::NamedPlaceholder(placeholder) => {
                placeholder.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for PlaceholderArgument {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "placeholder".hash(hasher);
    }
}

impl Fingerprintable for VariadicPlaceholderArgument {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "variadic_placeholder".hash(hasher);
    }
}

impl Fingerprintable for NamedPlaceholderArgument<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "named_placeholder".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
