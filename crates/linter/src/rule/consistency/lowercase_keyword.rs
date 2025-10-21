use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::*;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct LowercaseKeywordRule {
    meta: &'static RuleMeta,
    cfg: LowercaseKeywordConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LowercaseKeywordConfig {
    pub level: Level,
}

impl Default for LowercaseKeywordConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for LowercaseKeywordConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LowercaseKeywordRule {
    type Config = LowercaseKeywordConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Lowercase Keyword",
            code: "lowercase-keyword",
            description: indoc! {"
                Enforces that PHP keywords (like `if`, `else`, `return`, `function`, etc.) be written
                in lowercase. Using uppercase or mixed case is discouraged for consistency and readability.
            "},
            good_example: indoc! {r#"
                <?php

                if (true) {
                    echo "All keywords in lowercase";
                } else {
                    return;
                }
           "#},
            bad_example: indoc! {r#"
                <?PHP

                IF (TRUE) {
                    ECHO "Keywords not in lowercase";
                } ELSE {
                    RETURN;
                }
           "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Keyword];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Keyword(keyword) = node else {
            return;
        };

        if keyword.value.chars().all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase()) {
            return; // Already in lowercase, no issue to report
        }

        let lowercase = keyword.value.to_ascii_lowercase();

        let issue = Issue::new(self.cfg.level(), format!("Keyword `{}` should be in lowercase.", keyword.value))
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(keyword.span()))
            .with_note(format!("The keyword `{}` does not follow lowercase convention.", keyword.value))
            .with_help(format!("Consider using `{}` instead of `{}`.", lowercase, keyword.value));

        ctx.collector.report(issue);
    }
}
