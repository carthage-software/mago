use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Closure<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "closure".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.r#static.is_some().hash(hasher);
        self.ampersand.is_some().hash(hasher);
        self.parameter_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.use_clause.fingerprint_with_hasher(hasher, resolved_names, options);
        self.return_type_hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ClosureUseClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_clause".hash(hasher);

        for variable in self.variables.iter() {
            variable.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for ClosureUseClauseVariable<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "use_var".hash(hasher);
        self.ampersand.is_some().hash(hasher);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for Function<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "function".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.ampersand.is_some().hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.parameter_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.return_type_hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ArrowFunction<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "arrow_fn".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.r#static.is_some().hash(hasher);
        self.ampersand.is_some().hash(hasher);
        self.parameter_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.return_type_hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for FunctionLikeParameterList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "params".hash(hasher);

        for parameter in self.parameters.iter() {
            parameter.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for FunctionLikeParameter<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "param".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        for modifier in self.modifiers.iter() {
            modifier.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.ampersand.is_some().hash(hasher);
        self.ellipsis.is_some().hash(hasher);
        self.variable.fingerprint_with_hasher(hasher, resolved_names, options);
        self.default_value.fingerprint_with_hasher(hasher, resolved_names, options);
        self.hooks.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for FunctionLikeParameterDefaultValue<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "default_value".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for FunctionLikeReturnTypeHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "return_type".hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
