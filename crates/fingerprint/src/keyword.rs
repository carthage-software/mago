use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Keyword<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        mago_atom::ascii_lowercase_atom(self.value).hash(hasher);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_keyword_case_insensitive() {
        let fp1 = fingerprint_code("<?php function foo() {}");
        let fp2 = fingerprint_code("<?php FUNCTION foo() {}");
        let fp3 = fingerprint_code("<?php Function foo() {}");

        assert_eq!(fp1, fp2);
        assert_eq!(fp1, fp3);
    }

    #[test]
    fn test_different_keywords() {
        let fn_fp = fingerprint_code("<?php function foo() {}");
        let class_fp = fingerprint_code("<?php class Foo {}");

        assert_ne!(fn_fp, class_fp);
    }

    #[test]
    fn test_type_keywords_normalized() {
        let fp1 = fingerprint_code("<?php function foo(): string {}");
        let fp2 = fingerprint_code("<?php function foo(): STRING {}");
        let fp3 = fingerprint_code("<?php function foo(): String {}");

        assert_eq!(fp1, fp2);
        assert_eq!(fp1, fp3);
    }
}
