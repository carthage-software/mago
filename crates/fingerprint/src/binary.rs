use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;
use std::hash::Hash;

impl Fingerprintable for Binary<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "binary".hash(hasher);
        self.lhs.fingerprint_with_hasher(hasher, resolved_names, options);
        self.operator.fingerprint_with_hasher(hasher, resolved_names, options);
        self.rhs.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for BinaryOperator<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        use BinaryOperator::*;

        match self {
            Addition(_) => "+".hash(hasher),
            Subtraction(_) => "-".hash(hasher),
            Multiplication(_) => "*".hash(hasher),
            Division(_) => "/".hash(hasher),
            Modulo(_) => "%".hash(hasher),
            Exponentiation(_) => "**".hash(hasher),
            BitwiseAnd(_) => "&".hash(hasher),
            BitwiseOr(_) => "|".hash(hasher),
            BitwiseXor(_) => "^".hash(hasher),
            LeftShift(_) => "<<".hash(hasher),
            RightShift(_) => ">>".hash(hasher),
            NullCoalesce(_) => "??".hash(hasher),
            Equal(_) => "==".hash(hasher),
            NotEqual(_) | AngledNotEqual(_) => "!=".hash(hasher),
            Identical(_) => "===".hash(hasher),
            NotIdentical(_) => "!==".hash(hasher),
            LessThan(_) => "<".hash(hasher),
            LessThanOrEqual(_) => "<=".hash(hasher),
            GreaterThan(_) => ">".hash(hasher),
            GreaterThanOrEqual(_) => ">=".hash(hasher),
            Spaceship(_) => "<=>".hash(hasher),
            StringConcat(_) => ".".hash(hasher),
            Instanceof(_) => "instanceof".hash(hasher),
            And(_) => "&&".hash(hasher),
            Or(_) => "||".hash(hasher),
            LowAnd(_) => "and".hash(hasher),
            LowOr(_) => "or".hash(hasher),
            LowXor(_) => "xor".hash(hasher),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_binary_operations_basic() {
        let add1 = fingerprint_code("<?php 1 + 2;");
        let add2 = fingerprint_code("<?php 3 + 4;");
        let sub = fingerprint_code("<?php 1 - 2;");

        assert_ne!(add1, add2);
        assert_ne!(add1, sub);
    }

    #[test]
    fn test_not_equal_equivalence() {
        let ne1 = fingerprint_code("<?php 1 != 2;");
        let ne2 = fingerprint_code("<?php 1 <> 2;");

        assert_eq!(ne1, ne2);
    }

    #[test]
    fn test_keyword_operators_case_insensitive() {
        let inst1 = fingerprint_code("<?php $x instanceof Foo;");
        let inst2 = fingerprint_code("<?php $x INSTANCEOF Foo;");

        assert_eq!(inst1, inst2);
    }
}
