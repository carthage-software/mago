use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;

impl Fingerprintable for Expression<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Expression::Binary(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::UnaryPrefix(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::UnaryPostfix(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Parenthesized(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Literal(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::CompositeString(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Assignment(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Conditional(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Array(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::LegacyArray(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::List(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::ArrayAccess(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::ArrayAppend(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::AnonymousClass(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Closure(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::ArrowFunction(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Variable(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::ConstantAccess(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Identifier(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Match(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Yield(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Construct(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Throw(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Clone(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Call(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Access(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::ClosureCreation(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Parent(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Static(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Self_(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Instantiation(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::MagicConstant(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression::Pipe(node) => node.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for Parenthesized<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_parentheses_do_not_affect_fingerprint() {
        let expr1 = fingerprint_code("<?php 1 + 2;");
        let expr2 = fingerprint_code("<?php (1 + 2);");
        let expr3 = fingerprint_code("<?php ((1 + 2));");

        assert_eq!(expr1, expr2);
        assert_eq!(expr1, expr3);
    }

    #[test]
    fn test_expression_delegation() {
        let lit = fingerprint_code("<?php 42;");
        let var = fingerprint_code("<?php $x;");
        let bin = fingerprint_code("<?php 1 + 2;");

        assert_ne!(lit, var);
        assert_ne!(lit, bin);
        assert_ne!(var, bin);
    }
}
