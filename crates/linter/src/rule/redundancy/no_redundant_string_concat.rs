use indoc::indoc;
use mago_allocator::Arena;
use mago_syntax::cst::LiteralString;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::cst::Binary;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantStringConcatRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantStringConcatConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoRedundantStringConcatConfig {
    pub level: Level,
}

impl Default for NoRedundantStringConcatConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantStringConcatConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantStringConcatRule {
    type Config = NoRedundantStringConcatConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant String Concat",
            code: "no-redundant-string-concat",
            description: indoc! {"
                Detects redundant string concatenation expressions.
            "},
            good_example: indoc! {r#"
                <?php

                $foo = "Hello World";
            "#},
            bad_example: indoc! {r#"
                <?php

                $foo = "Hello" . " World";
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
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

        if !binary.operator.is_concatenation() {
            return;
        }

        let enclosing = match ctx.get_parent() {
            Some(Node::Expression(_)) => ctx.get_nth_parent(1),
            other => other,
        };

        if let Some(Node::Binary(parent)) = enclosing
            && parent.operator.is_concatenation()
        {
            return;
        }

        let mut operands = Vec::new();
        collect_concatenation_operands(binary, &mut operands);

        let mut index = 0;
        while index < operands.len() {
            let operand = operands[index];
            let Expression::Literal(Literal::String(first)) = operand else {
                index += 1;
                continue;
            };

            let mut run = vec![first];
            let mut last = first;
            let mut next_index = index + 1;
            while next_index < operands.len()
                && let Expression::Literal(Literal::String(next)) = operands[next_index]
                && pair_can_be_merged(ctx, last, next)
            {
                run.push(next);
                last = next;
                next_index += 1;
            }

            index = next_index;

            if run.len() < 2 {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), "String concatenation can be simplified.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(first.span().join(last.span())).with_message("Redundant string concatenation"),
                )
                .with_help("Consider combining these strings into a single string.");

            let ranges: Vec<TextRange> = run
                .iter()
                .zip(run.iter().skip(1))
                .map(|(left, right)| TextRange::new(left.end_offset() - 1, right.start_offset() + 1))
                .collect();

            ctx.collector.propose(issue, |edits| {
                for range in ranges {
                    edits.push(TextEdit::delete(range));
                }
            });
        }
    }
}

fn collect_concatenation_operands<'arena>(binary: &Binary<'arena>, operands: &mut Vec<&'arena Expression<'arena>>) {
    match binary.lhs {
        Expression::Binary(inner) if inner.operator.is_concatenation() => {
            collect_concatenation_operands(inner, operands);
        }
        other => operands.push(other),
    }

    operands.push(binary.rhs);
}

fn pair_can_be_merged<A>(ctx: &LintContext<'_, '_, A>, left: &LiteralString<'_>, right: &LiteralString<'_>) -> bool
where
    A: Arena,
{
    if left.kind != right.kind {
        return false;
    }

    if ctx.source_file.line_number(left.offset()) != ctx.source_file.line_number(right.offset()) {
        return false;
    }

    !matches!(&right.raw[1..], [b'{', ..])
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_fix;

    test_lint_failure! {
        name = chained_concatenation_reports_a_single_issue,
        rule = NoRedundantStringConcatRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $foo = 'a' . 'b' . 'c' . 'd' . 'e' . 'f' . 'g' . 'h' . 'i' . 'j' . 'k';
        "#},
    }

    test_lint_failure! {
        name = each_mergeable_run_reports_its_own_issue,
        rule = NoRedundantStringConcatRule,
        count = 2,
        code = indoc! {r#"
            <?php

            $foo = 'a' . 'b' . $middle . 'c' . 'd';
        "#},
    }

    test_lint_fix! {
        name = single_concatenation_is_merged,
        rule = NoRedundantStringConcatRule,
        code = indoc! {r#"
            <?php

            $foo = 'Hello' . ' World';
        "#},
        fixed = indoc! {r#"
            <?php

            $foo = 'Hello World';
        "#},
    }

    test_lint_fix! {
        name = chained_concatenation_is_merged_in_one_pass,
        rule = NoRedundantStringConcatRule,
        code = indoc! {r#"
            <?php

            $foo = 'mything' . 'myotherthing' . 'somethingelse';
        "#},
        fixed = indoc! {r#"
            <?php

            $foo = 'mythingmyotherthingsomethingelse';
        "#},
    }
}
