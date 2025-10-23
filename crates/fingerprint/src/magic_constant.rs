use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for MagicConstant<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        match self {
            MagicConstant::Line(_) => "magic_line".hash(hasher),
            MagicConstant::File(_) => "magic_file".hash(hasher),
            MagicConstant::Directory(_) => "magic_directory".hash(hasher),
            MagicConstant::Trait(_) => "magic_trait".hash(hasher),
            MagicConstant::Method(_) => "magic_method".hash(hasher),
            MagicConstant::Function(_) => "magic_function".hash(hasher),
            MagicConstant::Property(_) => "magic_property".hash(hasher),
            MagicConstant::Namespace(_) => "magic_namespace".hash(hasher),
            MagicConstant::Class(_) => "magic_class".hash(hasher),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_magic_constants_by_variant() {
        let line1 = fingerprint_code("<?php __LINE__;");
        let line2 = fingerprint_code("<?php __LINE__;");
        let file = fingerprint_code("<?php __FILE__;");

        assert_eq!(line1, line2);

        assert_ne!(line1, file);
    }

    #[test]
    fn test_all_magic_constant_variants_unique() {
        let fingerprints = [
            fingerprint_code("<?php __LINE__;"),
            fingerprint_code("<?php __FILE__;"),
            fingerprint_code("<?php __DIR__;"),
            fingerprint_code("<?php __TRAIT__;"),
            fingerprint_code("<?php __METHOD__;"),
            fingerprint_code("<?php __FUNCTION__;"),
            fingerprint_code("<?php __NAMESPACE__;"),
            fingerprint_code("<?php __CLASS__;"),
        ];

        for i in 0..fingerprints.len() {
            for j in (i + 1)..fingerprints.len() {
                assert_ne!(
                    fingerprints[i], fingerprints[j],
                    "Magic constants at index {} and {} have the same fingerprint",
                    i, j
                );
            }
        }
    }
}
