use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoShortBoolCastRule {
    meta: &'static RuleMeta,
    cfg: NoShortBoolCastConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoShortBoolCastConfig {
    pub level: Level,
}

impl Default for NoShortBoolCastConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoShortBoolCastConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoShortBoolCastRule {
    type Config = NoShortBoolCastConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Short Bool Cast",
            code: "no-short-bool-cast",
            description: indoc! {"
                Detects the use of double negation `!!$expr` as a shorthand for `(bool) $expr`.

                The explicit `(bool)` cast is clearer about the intent to convert a value
                to a boolean.
            "},
            good_example: indoc! {r"
                <?php

                $active = (bool) $value;
            "},
            bad_example: indoc! {r"
                <?php

                $active = !!$value;
            "},
            category: Category::Clarity,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::UnaryPrefix];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::UnaryPrefix(outer) = node else {
            return;
        };

        // Check for `!` operator
        if !matches!(outer.operator, UnaryPrefixOperator::Not(_)) {
            return;
        }

        // Check if the operand is also a `!` prefix operation
        let Expression::UnaryPrefix(inner) = outer.operand else {
            return;
        };

        if !matches!(inner.operator, UnaryPrefixOperator::Not(_)) {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use `(bool)` cast instead of double negation `!!`")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(outer.span()).with_message("Double negation `!!` is a shorthand for boolean cast"),
            )
            .with_help("Replace `!!$expr` with `(bool) $expr`");

        let source_code = ctx.source_file.contents.as_ref();
        ctx.collector.propose(issue, |edits| {
            let inner_operand_span = inner.operand.span();
            let inner_operand_text =
                &source_code[inner_operand_span.start_offset() as usize..inner_operand_span.end_offset() as usize];

            let replacement = format!("(bool) {}", inner_operand_text);
            edits.push(TextEdit::replace(outer.span(), &replacement));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoShortBoolCastRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = bool_cast_is_ok,
        rule = NoShortBoolCastRule,
        code = indoc! {r"
            <?php

            $active = (bool) $value;
        "}
    }

    test_lint_success! {
        name = single_negation_is_ok,
        rule = NoShortBoolCastRule,
        code = indoc! {r"
            <?php

            $inactive = !$active;
        "}
    }

    test_lint_failure! {
        name = double_negation_is_bad,
        rule = NoShortBoolCastRule,
        code = indoc! {r"
            <?php

            $active = !!$value;
        "}
    }
}
