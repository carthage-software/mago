use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for For<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "for".hash(hasher);
        for init in self.initializations.iter() {
            init.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        for condition in self.conditions.iter() {
            condition.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        for increment in self.increments.iter() {
            increment.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ForBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ForBody::Statement(statement) => statement.fingerprint_with_hasher(hasher, resolved_names, options),
            ForBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for ForColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "for_colon_body".hash(hasher);
        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Foreach<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "foreach".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.target.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ForeachTarget<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ForeachTarget::Value(target) => target.fingerprint_with_hasher(hasher, resolved_names, options),
            ForeachTarget::KeyValue(target) => target.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for ForeachValueTarget<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "foreach_value".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ForeachKeyValueTarget<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "foreach_key_value".hash(hasher);
        self.key.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ForeachBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ForeachBody::Statement(statement) => statement.fingerprint_with_hasher(hasher, resolved_names, options),
            ForeachBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for ForeachColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "foreach_colon_body".hash(hasher);
        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for While<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "while".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for WhileBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            WhileBody::Statement(statement) => statement.fingerprint_with_hasher(hasher, resolved_names, options),
            WhileBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for WhileColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "while_colon_body".hash(hasher);
        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for DoWhile<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "do_while".hash(hasher);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for Continue<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "continue".hash(hasher);
        self.level.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for Break<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "break".hash(hasher);
        self.level.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
