use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::cst::If;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::IfColonDelimitedBody;
use mago_syntax::cst::IfColonDelimitedBodyElseClause;
use mago_syntax::cst::IfColonDelimitedBodyElseIfClause;
use mago_syntax::cst::IfStatementBody;
use mago_syntax::cst::IfStatementBodyElseClause;
use mago_syntax::cst::IfStatementBodyElseIfClause;
use mago_syntax::cst::Match;
use mago_syntax::cst::MatchArm;
use mago_syntax::cst::MatchDefaultArm;
use mago_syntax::cst::MatchExpressionArm;
use mago_syntax::cst::Switch;
use mago_syntax::cst::SwitchBody;
use mago_syntax::cst::SwitchBraceDelimitedBody;
use mago_syntax::cst::SwitchCase;
use mago_syntax::cst::SwitchCaseSeparator;
use mago_syntax::cst::SwitchColonDelimitedBody;
use mago_syntax::cst::SwitchDefaultCase;
use mago_syntax::cst::SwitchExpressionCase;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Match<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "match".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        for arm in &self.arms {
            arm.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for MatchArm<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            MatchArm::Expression(arm) => arm.fingerprint_with_hasher(hasher, resolved_names, options),
            MatchArm::Default(arm) => arm.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for MatchExpressionArm<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "match_expr_arm".hash(hasher);
        for condition in &self.conditions {
            condition.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MatchDefaultArm<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "match_default_arm".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for If<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "if".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            IfBody::Statement(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
            IfBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for IfStatementBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "if_stmt_body".hash(hasher);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
        for elseif in &self.else_if_clauses {
            elseif.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.else_clause.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfStatementBodyElseIfClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "elseif".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfStatementBodyElseClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "else".hash(hasher);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfColonDelimitedBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "if_colon_body".hash(hasher);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        for elseif in &self.else_if_clauses {
            elseif.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.else_clause.fingerprint_with_hasher(hasher, resolved_names, options);
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfColonDelimitedBodyElseIfClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "elseif_colon".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for IfColonDelimitedBodyElseClause<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "else_colon".hash(hasher);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Switch<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "switch".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for SwitchBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            SwitchBody::BraceDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
            SwitchBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for SwitchBraceDelimitedBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "switch_brace_body".hash(hasher);
        self.optional_terminator.fingerprint_with_hasher(hasher, resolved_names, options);
        for case in &self.cases {
            case.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchColonDelimitedBody<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "switch_colon_body".hash(hasher);
        self.optional_terminator.fingerprint_with_hasher(hasher, resolved_names, options);
        for case in &self.cases {
            case.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for SwitchCase<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            SwitchCase::Expression(case) => case.fingerprint_with_hasher(hasher, resolved_names, options),
            SwitchCase::Default(case) => case.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for SwitchExpressionCase<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "case".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.separator.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchDefaultCase<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "default".hash(hasher);
        self.separator.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchCaseSeparator {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        "switch_case_separator".hash(hasher);
    }
}
