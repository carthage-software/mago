use indoc::indoc;
use mago_fixer::SafetyClassification;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoSelfAssignmentRule {
    meta: &'static RuleMeta,
    cfg: NoSelfAssignmentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoSelfAssignmentConfig {
    pub level: Level,
}

impl Default for NoSelfAssignmentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoSelfAssignmentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoSelfAssignmentRule {
    type Config = NoSelfAssignmentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Self Assignment",
            code: "no-self-assignment",
            description: indoc! {"
                Detects and removes self-assignments where a variable or property is assigned to itself.

                Self-assignments have no effect and are typically mistakes or leftover from refactoring.
                For object properties, the fix is marked as potentially unsafe because reading or writing
                properties may have side effects through magic methods (__get, __set) or property hooks (PHP 8.4+).
            "},
            good_example: indoc! {r"
                <?php

                $a = $b;
                $this->x = $other->x;
                $foo->bar = $baz->bar;
            "},
            bad_example: indoc! {r"
                <?php

                $a = $a;
                $this->x = $this->x;
                $foo->bar = $foo->bar;
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ExpressionStatement];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::ExpressionStatement(statement) = node else { return };
        let Expression::Assignment(assignment) = statement.expression else { return };

        // Only check plain assignment operator (=), not compound assignments (+=, -=, etc.)
        if !assignment.operator.is_assign() {
            return;
        }

        // Check if LHS and RHS are semantically equivalent
        if !are_expressions_equivalent(assignment.lhs, assignment.rhs) {
            return;
        }

        // Determine safety classification based on expression type
        let safety = if is_object_property_access(assignment.lhs) {
            // Object property access may have side effects through __get/__set or property hooks
            SafetyClassification::PotentiallyUnsafe
        } else {
            // Variable assignments are safe to remove
            SafetyClassification::Safe
        };

        let issue = Issue::new(self.cfg.level(), "Self-assignment has no effect.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(assignment.span()).with_message("This assignment assigns a value to itself"),
            )
            .with_note("Self-assignments are redundant and typically indicate a mistake or leftover from refactoring.")
            .with_help("Remove this self-assignment statement.");

        ctx.collector.propose(issue, |plan| {
            // Delete the entire statement including the semicolon
            plan.delete(statement.span().to_range(), safety);
        });
    }
}

/// Check if two expressions are semantically equivalent.
#[inline]
fn are_expressions_equivalent<'arena>(lhs: &Expression<'arena>, rhs: &Expression<'arena>) -> bool {
    match (lhs, rhs) {
        // Variable: $a = $a
        (Expression::Variable(lhs_var), Expression::Variable(rhs_var)) => variables_equal(lhs_var, rhs_var),
        // Object property access: $foo->bar = $foo->bar
        (Expression::Access(lhs_access), Expression::Access(rhs_access)) => {
            property_accesses_equal(lhs_access, rhs_access)
        }

        _ => false,
    }
}

/// Check if a variable expression is an object property access.
const fn is_object_property_access(expr: &Expression) -> bool {
    matches!(expr, Expression::Access(_))
}

/// Check if two variables are the same.
#[inline]
fn variables_equal<'arena>(lhs: &Variable<'arena>, rhs: &Variable<'arena>) -> bool {
    match (lhs, rhs) {
        (Variable::Direct(lhs_direct), Variable::Direct(rhs_direct)) => lhs_direct.name == rhs_direct.name,
        _ => false,
    }
}

/// Check if two property access expressions are equivalent.
#[inline]
fn property_accesses_equal<'arena>(lhs: &Access<'arena>, rhs: &Access<'arena>) -> bool {
    match (lhs, rhs) {
        (Access::Property(lhs_prop), Access::Property(rhs_prop)) => {
            // Check if the objects are the same
            if !are_expressions_equivalent(lhs_prop.object, rhs_prop.object) {
                return false;
            }

            // Check if the property names are the same
            match (&lhs_prop.property, &rhs_prop.property) {
                (ClassLikeMemberSelector::Identifier(lhs_id), ClassLikeMemberSelector::Identifier(rhs_id)) => {
                    lhs_id.value == rhs_id.value
                }
                _ => false,
            }
        }
        _ => false,
    }
}
