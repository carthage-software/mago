use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;
use std::hash::Hash;

impl Fingerprintable for UnaryPrefix<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "unary_prefix".hash(hasher);
        self.operator.fingerprint_with_hasher(hasher, resolved_names, options);
        self.operand.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UnaryPostfix<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "unary_postfix".hash(hasher);
        self.operand.fingerprint_with_hasher(hasher, resolved_names, options);
        self.operator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for UnaryPrefixOperator<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        use UnaryPrefixOperator::*;

        match self {
            ErrorControl(_) => "@".hash(hasher),
            Reference(_) => "&".hash(hasher),
            BoolCast(..) | BooleanCast(..) => "(bool)".hash(hasher),
            DoubleCast(..) | RealCast(..) | FloatCast(..) => "(float)".hash(hasher),
            IntCast(..) | IntegerCast(..) => "(int)".hash(hasher),
            ArrayCast(..) => "(array)".hash(hasher),
            ObjectCast(..) => "(object)".hash(hasher),
            UnsetCast(..) => "(unset)".hash(hasher),
            StringCast(..) => "(string)".hash(hasher),
            BinaryCast(..) => "(binary)".hash(hasher),
            VoidCast(..) => "(void)".hash(hasher),
            BitwiseNot(_) => "~".hash(hasher),
            Not(_) => "!".hash(hasher),
            PreIncrement(_) => "++".hash(hasher),
            PreDecrement(_) => "--".hash(hasher),
            Plus(_) => "+".hash(hasher),
            Negation(_) => "-".hash(hasher),
        }
    }
}

impl Fingerprintable for UnaryPostfixOperator {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        use UnaryPostfixOperator::*;

        match self {
            PostIncrement(_) => "++".hash(hasher),
            PostDecrement(_) => "--".hash(hasher),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_unary_prefix_operations() {
        let not = fingerprint_code("<?php !true;");
        let neg = fingerprint_code("<?php -5;");
        let plus = fingerprint_code("<?php +5;");

        assert_ne!(not, neg);
        assert_ne!(neg, plus);
    }

    #[test]
    fn test_cast_normalization() {
        let bool1 = fingerprint_code("<?php (bool)$x;");
        let bool2 = fingerprint_code("<?php (boolean)$x;");
        assert_eq!(bool1, bool2, "bool and boolean casts should be equivalent");

        let int1 = fingerprint_code("<?php (int)$x;");
        let int2 = fingerprint_code("<?php (integer)$x;");
        assert_eq!(int1, int2, "int and integer casts should be equivalent");

        let float1 = fingerprint_code("<?php (float)$x;");
        let float2 = fingerprint_code("<?php (double)$x;");
        let float3 = fingerprint_code("<?php (real)$x;");
        assert_eq!(float1, float2, "float and double casts should be equivalent");
        assert_eq!(float1, float3, "float and real casts should be equivalent");
    }

    #[test]
    fn test_unary_postfix_operations() {
        let inc = fingerprint_code("<?php $x++;");
        let dec = fingerprint_code("<?php $x--;");

        assert_ne!(inc, dec);
    }
}
