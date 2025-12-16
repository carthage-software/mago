use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::If;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::IfColonDelimitedBody;
use mago_syntax::ast::IfColonDelimitedBodyElseClause;
use mago_syntax::ast::IfColonDelimitedBodyElseIfClause;
use mago_syntax::ast::IfStatementBody;
use mago_syntax::ast::IfStatementBodyElseClause;
use mago_syntax::ast::IfStatementBodyElseIfClause;
use mago_syntax::ast::Match;
use mago_syntax::ast::MatchArm;
use mago_syntax::ast::MatchDefaultArm;
use mago_syntax::ast::MatchExpressionArm;
use mago_syntax::ast::Switch;
use mago_syntax::ast::SwitchBody;
use mago_syntax::ast::SwitchBraceDelimitedBody;
use mago_syntax::ast::SwitchCase;
use mago_syntax::ast::SwitchCaseSeparator;
use mago_syntax::ast::SwitchColonDelimitedBody;
use mago_syntax::ast::SwitchDefaultCase;
use mago_syntax::ast::SwitchExpressionCase;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Match<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "match".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        for arm in &self.arms {
            arm.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for MatchArm<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            MatchArm::Expression(arm) => arm.fingerprint_with_hasher(hasher, resolved_names, options),
            MatchArm::Default(arm) => arm.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for MatchExpressionArm<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "match_expr_arm".hash(hasher);
        for condition in &self.conditions {
            condition.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MatchDefaultArm<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "match_default_arm".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for If<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "if".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            IfBody::Statement(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
            IfBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for IfStatementBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "if_stmt_body".hash(hasher);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
        for elseif in &self.else_if_clauses {
            elseif.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.else_clause.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfStatementBodyElseIfClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "elseif".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfStatementBodyElseClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "else".hash(hasher);
        self.statement.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for IfColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
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
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "elseif_colon".hash(hasher);
        self.condition.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for IfColonDelimitedBodyElseClause<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "else_colon".hash(hasher);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Switch<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "switch".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for SwitchBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            SwitchBody::BraceDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
            SwitchBody::ColonDelimited(body) => body.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for SwitchBraceDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "switch_brace_body".hash(hasher);
        self.optional_terminator.fingerprint_with_hasher(hasher, resolved_names, options);
        for case in &self.cases {
            case.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchColonDelimitedBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "switch_colon_body".hash(hasher);
        self.optional_terminator.fingerprint_with_hasher(hasher, resolved_names, options);
        for case in &self.cases {
            case.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for SwitchCase<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            SwitchCase::Expression(case) => case.fingerprint_with_hasher(hasher, resolved_names, options),
            SwitchCase::Default(case) => case.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for SwitchExpressionCase<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "case".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.separator.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchDefaultCase<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "default".hash(hasher);
        self.separator.fingerprint_with_hasher(hasher, resolved_names, options);
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for SwitchCaseSeparator {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "switch_case_separator".hash(hasher);
    }
}
