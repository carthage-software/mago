use mago_names::ResolvedNames;
use mago_syntax::ast::Construct;
use mago_syntax::ast::DieConstruct;
use mago_syntax::ast::EmptyConstruct;
use mago_syntax::ast::EvalConstruct;
use mago_syntax::ast::ExitConstruct;
use mago_syntax::ast::IncludeConstruct;
use mago_syntax::ast::IncludeOnceConstruct;
use mago_syntax::ast::IssetConstruct;
use mago_syntax::ast::PrintConstruct;
use mago_syntax::ast::RequireConstruct;
use mago_syntax::ast::RequireOnceConstruct;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Construct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        use Construct::Die;
        use Construct::Empty;
        use Construct::Eval;
        use Construct::Exit;
        use Construct::Include;
        use Construct::IncludeOnce;
        use Construct::Isset;
        use Construct::Print;
        use Construct::Require;
        use Construct::RequireOnce;

        match self {
            Isset(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Empty(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Eval(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Include(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            IncludeOnce(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Require(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            RequireOnce(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Print(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Exit(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
            Die(c) => c.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for IssetConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "isset".hash(hasher);
        for value in &self.values {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for EmptyConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "empty".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for EvalConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "eval".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IncludeConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "include".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IncludeOnceConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "include_once".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for RequireConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "require".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for RequireOnceConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "require_once".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for PrintConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "print".hash(hasher);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ExitConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "exit".hash(hasher);

        self.arguments.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for DieConstruct<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "exit".hash(hasher);

        self.arguments.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
