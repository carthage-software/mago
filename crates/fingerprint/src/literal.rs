use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Literal<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        use Literal::*;

        match self {
            True(_) => "true".hash(hasher),
            False(_) => "false".hash(hasher),
            Null(_) => "null".hash(hasher),
            Integer(lit) => {
                "int".hash(hasher);
                if let Some(v) = lit.value {
                    v.hash(hasher);
                } else {
                    lit.raw.hash(hasher);
                }
            }
            Float(lit) => {
                "float".hash(hasher);
                lit.value.hash(hasher);
            }
            String(lit) => {
                "string".hash(hasher);
                if let Some(v) = lit.value {
                    v.hash(hasher);
                } else {
                    lit.raw.hash(hasher);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_boolean_literals_normalized() {
        let true_fp = fingerprint_code("<?php true;");
        let false_fp = fingerprint_code("<?php false;");

        assert_ne!(true_fp, false_fp);

        let true_fp2 = fingerprint_code("<?php True;");
        assert_eq!(true_fp, true_fp2);
    }

    #[test]
    fn test_null_literal() {
        let null1 = fingerprint_code("<?php null;");
        let null2 = fingerprint_code("<?php NULL;");

        assert_eq!(null1, null2);
    }

    #[test]
    fn test_integer_literals() {
        let int1 = fingerprint_code("<?php 42;");
        let int2 = fingerprint_code("<?php 4_2;");
        let int3 = fingerprint_code("<?php 100;");

        assert_eq!(int1, int2);

        assert_ne!(int1, int3);
    }

    #[test]
    fn test_string_literals() {
        let str1 = fingerprint_code("<?php 'hello';");
        let str2 = fingerprint_code("<?php \"hello\";");
        let str3 = fingerprint_code("<?php 'world';");

        assert_eq!(str1, str2);
        assert_ne!(str1, str3);
    }
}
