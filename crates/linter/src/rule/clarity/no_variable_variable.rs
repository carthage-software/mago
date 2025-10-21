use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoVariableVariableRule {
    meta: &'static RuleMeta,
    cfg: NoVariableVariableConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoVariableVariableConfig {
    pub level: Level,
}

impl Default for NoVariableVariableConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoVariableVariableConfig {
    fn default_enabled() -> bool {
        // TODO(azjezz): enable by default in the next major release
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoVariableVariableRule {
    type Config = NoVariableVariableConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Variable Variable",
            code: "no-variable-variable",
            description: indoc! {r#"
                Discourages usage of PHP's variable variables feature.

                Variable variables can make code harder to read and maintain, as they introduce a level of indirection that can confuse readers and complicate static analysis.
            "#},
            good_example: indoc! {r#"
                <?php

                $foo = 'bar';

                echo $foo; // Outputs 'bar'
            "#},
            bad_example: indoc! {r#"
                <?php

                $foo = 'bar';
                $varName = 'foo';

                echo $$varName; // Outputs 'bar'
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Variable];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Variable(variable) = node else {
            return;
        };

        let Variable::Nested(nested_variable) = variable else {
            return;
        };

        ctx.collector.report(
            Issue::new(
                self.cfg.level,
                "Usage of variable variables (e.g., `$$var`) is discouraged.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(nested_variable.span())
                    .with_message("This is a variable variable"),
            )
            .with_note(
                "Variable variables can make code harder to read and maintain. Consider using arrays or other data structures instead.",
            )
            .with_help("Refactor the code to avoid using variable variables."),
        );
    }
}
