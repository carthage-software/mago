use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;
use std::hash::Hash;

impl Fingerprintable for Modifier<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        match self {
            Modifier::Static(_) => "static".hash(hasher),
            Modifier::Final(_) => "final".hash(hasher),
            Modifier::Abstract(_) => "abstract".hash(hasher),
            Modifier::Readonly(_) => "readonly".hash(hasher),
            Modifier::Public(_) => "public".hash(hasher),
            Modifier::PublicSet(_) => "public_set".hash(hasher),
            Modifier::Protected(_) => "protected".hash(hasher),
            Modifier::ProtectedSet(_) => "protected_set".hash(hasher),
            Modifier::Private(_) => "private".hash(hasher),
            Modifier::PrivateSet(_) => "private_set".hash(hasher),
        }
    }
}

pub fn fingerprint_modifiers<'a, H: std::hash::Hasher>(
    modifiers: impl IntoIterator<Item = &'a Modifier<'a>>,
    hasher: &mut H,
    _resolved_names: &ResolvedNames,
    _options: &FingerprintOptions<'_>,
) {
    let mut modifier_strings: Vec<&str> = modifiers
        .into_iter()
        .filter_map(|m| match m {
            Modifier::Public(_) => None, // Skip public modifier
            Modifier::Static(_) => Some("static"),
            Modifier::Final(_) => Some("final"),
            Modifier::Abstract(_) => Some("abstract"),
            Modifier::Readonly(_) => Some("readonly"),
            Modifier::PublicSet(_) => Some("public_set"),
            Modifier::Protected(_) => Some("protected"),
            Modifier::ProtectedSet(_) => Some("protected_set"),
            Modifier::Private(_) => Some("private"),
            Modifier::PrivateSet(_) => Some("private_set"),
        })
        .collect();

    modifier_strings.sort_unstable();

    for modifier_str in modifier_strings {
        modifier_str.hash(hasher);
    }
}
