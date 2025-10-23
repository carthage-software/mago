use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for ConstantAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for Access<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Access::Property(access) => access.fingerprint_with_hasher(hasher, resolved_names, options),
            Access::NullSafeProperty(access) => access.fingerprint_with_hasher(hasher, resolved_names, options),
            Access::StaticProperty(access) => access.fingerprint_with_hasher(hasher, resolved_names, options),
            Access::ClassConstant(access) => access.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for PropertyAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "prop_access".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.property.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for NullSafePropertyAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "null_safe_prop_access".hash(hasher);
        self.object.fingerprint_with_hasher(hasher, resolved_names, options);
        self.property.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for StaticPropertyAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "static_prop_access".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.property.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ClassConstantAccess<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "class_const_access".hash(hasher);
        self.class.fingerprint_with_hasher(hasher, resolved_names, options);
        self.constant.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ClassLikeMemberSelector<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ClassLikeMemberSelector::Identifier(id) => id.fingerprint_with_hasher(hasher, resolved_names, options),
            ClassLikeMemberSelector::Variable(var) => var.fingerprint_with_hasher(hasher, resolved_names, options),
            ClassLikeMemberSelector::Expression(expr) => expr.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for ClassLikeConstantSelector<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ClassLikeConstantSelector::Identifier(id) => id.fingerprint_with_hasher(hasher, resolved_names, options),
            ClassLikeConstantSelector::Expression(expr) => {
                expr.fingerprint_with_hasher(hasher, resolved_names, options)
            }
        }
    }
}

impl Fingerprintable for ClassLikeMemberExpressionSelector<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "expr_selector".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_property_access_vs_null_safe() {
        let regular = fingerprint_code("<?php $obj->prop;");
        let null_safe = fingerprint_code("<?php $obj?->prop;");

        assert_ne!(regular, null_safe);
    }

    #[test]
    fn test_property_names_case_insensitive() {
        let lower = fingerprint_code("<?php $obj->prop;");
        let upper = fingerprint_code("<?php $obj->PROP;");

        assert_ne!(lower, upper);
    }

    #[test]
    fn test_class_constant_case_sensitive() {
        let lower = fingerprint_code("<?php Foo::const;");
        let upper = fingerprint_code("<?php Foo::CONST;");

        assert_ne!(lower, upper);
    }

    #[test]
    fn test_different_access_types() {
        let prop = fingerprint_code("<?php $obj->prop;");
        let static_prop = fingerprint_code("<?php Foo::$prop;");
        let const_access = fingerprint_code("<?php Foo::CONST;");

        assert_ne!(prop, static_prop);
        assert_ne!(prop, const_access);
        assert_ne!(static_prop, const_access);
    }
}
