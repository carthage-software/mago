use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Binary;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Conditional;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::Variable;
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
pub struct PreferNullCoalescingRule {
    meta: &'static RuleMeta,
    cfg: PreferNullCoalescingConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferNullCoalescingConfig {
    pub level: Level,
}

impl Default for PreferNullCoalescingConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferNullCoalescingConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferNullCoalescingRule {
    type Config = PreferNullCoalescingConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Null Coalescing",
            code: "prefer-null-coalescing",
            description: indoc! {"
                Suggests the null coalescing operator (`??`) for ternaries that return a value
                when it is set or not null, and a fallback otherwise.

                `isset($value) ? $value : $default`, `$value !== null ? $value : $default` and
                `$value === null ? $default : $value` all read as `$value ?? $default`, which
                names the value once.
            "},
            good_example: indoc! {r"
                <?php

                $name = $user->name ?? 'anonymous';
            "},
            bad_example: indoc! {r"
                <?php

                $name = isset($user->name) ? $user->name : 'anonymous';
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP70)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Conditional];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Conditional(conditional) = node else {
            return;
        };

        let Some(candidate) = match_candidate(conditional) else {
            return;
        };

        let contents = &ctx.source_file.contents;
        let value = unwrap_parentheses(candidate.value).span();
        let checked = &contents[candidate.checked.span().to_range_usize()];
        if *checked != contents[value.to_range_usize()] {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use the null coalescing operator `??`.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(conditional.span()).with_message("This can be written with `??`"))
            .with_note("The ternary returns the value it checks when it is set or not null, and a fallback otherwise.")
            .with_help("Replace the ternary with `$value ?? $default`.");

        // `??` evaluates its left operand once, through `isset()` semantics. That only matches the ternary when
        // reading the value twice is free of side effects, and, for a `null` comparison, when the value is a
        // variable: a property or an offset may go through `__isset()` or `offsetExists()` instead.
        let safe = if candidate.from_isset {
            is_plain_reference(candidate.checked)
        } else {
            matches!(candidate.checked, Expression::Variable(Variable::Direct(_)))
        };
        let safety = if safe { Safety::Safe } else { Safety::PotentiallyUnsafe };

        let span = conditional.span();
        let fallback = candidate.fallback.span();
        ctx.collector.propose(issue, |edits| {
            if value.start < fallback.start {
                // `cond ? $value : fallback`: drop the condition and turn the colon into `??`.
                edits.push(TextEdit::delete(span.to_end(value.start)).with_safety(safety));
                edits.push(
                    TextEdit::replace(span.from_start(value.end).to_end(fallback.start), " ?? ").with_safety(safety),
                );
            } else {
                // `cond ? fallback : $value`: the value moves in front of the fallback.
                let value = String::from_utf8_lossy(&contents[value.to_range_usize()]);
                edits.push(TextEdit::replace(span.to_end(fallback.start), format!("{value} ?? ")).with_safety(safety));
                edits.push(TextEdit::delete(span.from_start(fallback.end)).with_safety(safety));
            }
        });
    }
}

struct Candidate<'ast, 'arena> {
    /// The expression the condition checks.
    checked: &'ast Expression<'arena>,
    /// The branch taken when it is set or not null.
    value: &'ast Expression<'arena>,
    /// The branch taken otherwise.
    fallback: &'ast Expression<'arena>,
    from_isset: bool,
}

fn match_candidate<'ast, 'arena>(conditional: &'ast Conditional<'arena>) -> Option<Candidate<'ast, 'arena>> {
    let then = conditional.then?;
    let r#else = conditional.r#else;

    match unwrap_parentheses(conditional.condition) {
        Expression::Construct(Construct::Isset(isset)) => {
            let [checked] = isset.values.nodes else {
                return None;
            };

            Some(Candidate { checked, value: then, fallback: r#else, from_isset: true })
        }
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) if operator.is_not() => {
            let Expression::Construct(Construct::Isset(isset)) = unwrap_parentheses(operand) else {
                return None;
            };
            let [checked] = isset.values.nodes else {
                return None;
            };

            Some(Candidate { checked, value: r#else, fallback: then, from_isset: true })
        }
        Expression::Binary(Binary { lhs, operator, rhs }) => {
            let checked = if is_null(rhs) {
                unwrap_parentheses(lhs)
            } else if is_null(lhs) {
                unwrap_parentheses(rhs)
            } else {
                return None;
            };

            match operator {
                BinaryOperator::NotIdentical(_) => {
                    Some(Candidate { checked, value: then, fallback: r#else, from_isset: false })
                }
                BinaryOperator::Identical(_) => {
                    Some(Candidate { checked, value: r#else, fallback: then, from_isset: false })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn unwrap_parentheses<'ast, 'arena>(mut expression: &'ast Expression<'arena>) -> &'ast Expression<'arena> {
    while let Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression;
    }

    expression
}

fn is_null(expression: &Expression<'_>) -> bool {
    matches!(unwrap_parentheses(expression), Expression::Literal(Literal::Null(_)))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use mago_text_edit::Safety;

    use super::PreferNullCoalescingRule;
    use crate::rule::tests::collect_issues;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = reports_ternaries_that_coalesce_null,
        rule = PreferNullCoalescingRule,
        count = 7,
        code = indoc! {r"
            <?php

            $a = isset($user->name) ? $user->name : 'anonymous';
            $b = !isset($options['key']) ? null : $options['key'];
            $c = $value !== null ? $value : $default;
            $d = null !== $value ? $value : $default;
            $e = $value === null ? $default : $value;
            $f = (null === $this->find()) ? [] : ($this->find());
            $g = isset(self::$cache) ? self::$cache : compute();
        "}
    }

    test_lint_success! {
        name = ignores_other_ternaries,
        rule = PreferNullCoalescingRule,
        code = indoc! {r"
            <?php

            $a = isset($a, $b) ? $a : $c;
            $b = isset($x) ? $y : $z;
            $c = $value != null ? $value : $default;
            $d = $value !== null ? $default : $value;
            $e = $value === null ? $value : $default;
            $f = $value !== false ? $value : $default;
            $g = $value ?: $default;
            $h = isset($x) ? $x->name : null;
            $i = empty($x) ? $y : $x;
        "}
    }

    test_lint_fix! {
        name = rewrites_to_null_coalescing,
        rule = PreferNullCoalescingRule,
        code = indoc! {r"
            <?php

            $a = isset($user->name) ? $user->name : 'anonymous';
            $b = !isset($options['key']) ? null : $options['key'];
            $c = $value !== null ? ($value) : $default;
            $d = $value === null ? throw new Exception() : $value;
            $e = 1 + (isset($n) ? $n : 0);
            $f = isset($n) ? $n : compute(/* lazily */ 1);
            $g = $n === null ? compute(/* lazily */ 1) : $n;
        "},
        fixed = indoc! {r"
            <?php

            $a = $user->name ?? 'anonymous';
            $b = $options['key'] ?? null;
            $c = $value ?? $default;
            $d = $value ?? throw new Exception();
            $e = 1 + ($n ?? 0);
            $f = $n ?? compute(/* lazily */ 1);
            $g = $n ?? compute(/* lazily */ 1);
        "}
    }

    #[test]
    fn marks_fixes_unsafe_when_the_value_is_not_a_plain_reference() {
        let code = indoc! {r"
            <?php

            $a = isset($this->ids) ? $this->ids : [];
            $b = $value !== null ? $value : [];
            $c = isset($map[next_key()]) ? $map[next_key()] : [];
            $d = $this->ids !== null ? $this->ids : [];
            $e = $this->find() !== null ? $this->find() : [];
        "};

        let safeties: Vec<_> = collect_issues::<PreferNullCoalescingRule, fn(&mut Settings)>(code, None, None)
            .into_iter()
            .filter_map(|issue| issue.edits.into_values().flatten().map(|edit| edit.safety).next())
            .collect();

        assert_eq!(
            safeties,
            [
                Safety::Safe,
                Safety::Safe,
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe
            ]
        );
    }
}
