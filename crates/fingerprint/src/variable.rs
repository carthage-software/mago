use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Variable<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Variable::Direct(var) => var.fingerprint_with_hasher(hasher, resolved_names, options),
            Variable::Indirect(var) => var.fingerprint_with_hasher(hasher, resolved_names, options),
            Variable::Nested(var) => var.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for DirectVariable<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "var".hash(hasher);
        self.name.hash(hasher);
    }
}

impl Fingerprintable for IndirectVariable<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "var_indirect".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NestedVariable<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "var_nested".hash(hasher);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_direct_variables() {
        let var1 = fingerprint_code("<?php $foo;");
        let var2 = fingerprint_code("<?php $bar;");
        let var3 = fingerprint_code("<?php $foo;");

        assert_eq!(var1, var3);
        assert_ne!(var1, var2);
    }

    #[test]
    fn test_variable_names_are_case_sensitive() {
        let var1 = fingerprint_code("<?php $foo;");
        let var2 = fingerprint_code("<?php $Foo;");
        let var3 = fingerprint_code("<?php $FOO;");

        assert_ne!(var1, var2);
        assert_ne!(var1, var3);
        assert_ne!(var2, var3);
    }

    #[test]
    fn test_nested_variables() {
        let var1 = fingerprint_code("<?php $$foo;");
        let var2 = fingerprint_code("<?php $foo;");

        assert_ne!(var1, var2);
    }
}
