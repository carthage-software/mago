use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::is_plain_reference;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferNullCoalescingAssignmentRule {
    meta: &'static RuleMeta,
    cfg: PreferNullCoalescingAssignmentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferNullCoalescingAssignmentConfig {
    pub level: Level,
}

impl Default for PreferNullCoalescingAssignmentConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferNullCoalescingAssignmentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferNullCoalescingAssignmentRule {
    type Config = PreferNullCoalescingAssignmentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Null Coalescing Assignment",
            code: "prefer-null-coalescing-assignment",
            description: indoc! {"
                Suggests the null coalescing assignment operator (`??=`) when the right operand
                of `??` assigns to the same variable or property as its left operand.

                `$value ?? ($value = $default)` and `$value ??= $default` do the same work, but
                the latter names the target once.
            "},
            good_example: indoc! {r"
                <?php

                return $this->ids ??= $this->readIds();
            "},
            bad_example: indoc! {r"
                <?php

                return $this->ids ?? ($this->ids = $this->readIds());
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP74)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Binary(binary) = node else {
            return;
        };

        if !binary.operator.is_null_coalesce() {
            return;
        }

        let mut rhs = binary.rhs;
        while let Expression::Parenthesized(parenthesized) = rhs {
            rhs = parenthesized.expression;
        }

        let Expression::Assignment(assignment) = rhs else {
            return;
        };

        let contents = &ctx.source_file.contents;
        if !assignment.operator.is_assign()
            || contents[binary.lhs.span().to_range_usize()] != contents[assignment.lhs.span().to_range_usize()]
        {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use the null coalescing assignment operator `??=`.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(binary.span()).with_message("This can be written as `??=`"))
            .with_note("The right side of `??` assigns to the same target as its left side.")
            .with_help("Replace `$value ?? ($value = $default)` with `$value ??= $default`.");

        let safety = if is_plain_reference(binary.lhs) { Safety::Safe } else { Safety::PotentiallyUnsafe };

        ctx.collector.propose(issue, |edits| {
            edits.push(
                TextEdit::replace(binary.operator.span().to_end(assignment.rhs.span().start), "??= ")
                    .with_safety(safety),
            );
            edits.push(TextEdit::delete(binary.span().from_start(assignment.rhs.span().end)).with_safety(safety));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use mago_text_edit::Safety;

    use super::PreferNullCoalescingAssignmentRule;
    use crate::rule::tests::collect_issues;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = reports_coalesce_that_assigns_its_left_operand,
        rule = PreferNullCoalescingAssignmentRule,
        count = 4,
        code = indoc! {r"
            <?php

            final class Ids
            {
                private ?array $ids = null;
                private static ?array $cache = null;

                public function get(): array
                {
                    $local = $this->ids ?? ($this->ids = $this->read());
                    $cached = self::$cache ?? self::$cache = $this->read();
                    $item = $local['key'] ?? ($local['key'] = 1);

                    return $this->load()->ids ?? ($this->load()->ids = []);
                }
            }
        "}
    }

    test_lint_success! {
        name = ignores_other_targets_and_operators,
        rule = PreferNullCoalescingAssignmentRule,
        code = indoc! {r"
            <?php

            $a = $b ?? ($c = 1);
            $a = $b ?? ($b += 1);
            $a = $b ?: ($b = 1);
            $a ??= 1;
        "}
    }

    test_lint_fix! {
        name = rewrites_to_null_coalescing_assignment,
        rule = PreferNullCoalescingAssignmentRule,
        code = indoc! {r"
            <?php

            return $this->ids ?? ($this->ids = $this->readIds());
            return $ids ?? $ids = [];
            return $map[$key] ?? (($map[$key] = compute($key)));
        "},
        fixed = indoc! {r"
            <?php

            return $this->ids ??= $this->readIds();
            return $ids ??= [];
            return $map[$key] ??= compute($key);
        "}
    }

    #[test]
    fn marks_fixes_unsafe_when_the_target_is_evaluated_with_effects() {
        let code = indoc! {r"
            <?php

            $a = $this->ids ?? ($this->ids = []);
            $b = $this->load()->ids ?? ($this->load()->ids = []);
            $c = $map[next_key()] ?? ($map[next_key()] = []);
        "};

        let safeties: Vec<_> =
            collect_issues::<PreferNullCoalescingAssignmentRule, fn(&mut Settings)>(code, None, None)
                .into_iter()
                .flat_map(|issue| issue.edits.into_values().flatten())
                .map(|edit| edit.safety)
                .collect();

        assert_eq!(
            safeties,
            [
                Safety::Safe,
                Safety::Safe,
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe,
            ]
        );
    }
}
