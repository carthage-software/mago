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
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct RequirePregQuoteDelimiterRule {
    meta: &'static RuleMeta,
    cfg: RequirePregQuoteDelimiterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RequirePregQuoteDelimiterConfig {
    pub level: Level,
}

impl Default for RequirePregQuoteDelimiterConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for RequirePregQuoteDelimiterConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for RequirePregQuoteDelimiterRule {
    type Config = RequirePregQuoteDelimiterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Require `preg_quote` Delimiter",
            code: "require-preg-quote-delimiter",
            description: indoc! {"
                This rule requires that when using `preg_quote()`, the second `$delimiter` argument is always provided.
                If the string being quoted contains the same character used for your regex delimiter (e.g., `/`),
                failing to provide the second argument will prevent that character from being escaped,
                which can break the regular expression.
            "},
            good_example: indoc! {r#"
                <?php

                // The delimiter is provided, ensuring it gets escaped if necessary.
                $pattern = '/' . preg_quote( $user_input, '/' ) . '/';
            "#},
            bad_example: indoc! {r#"
                <?php

                // If $user_input contains '/', the regex will be invalid.
                $pattern = '/' . preg_quote( $user_input ) . '/';
            "#},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        // If we already have 2 or more arguments, no need to check further
        if function_call.argument_list.arguments.len() >= 2 {
            return;
        }

        // Check if this is a call to preg_quote
        if !function_call_matches(ctx, function_call, "preg_quote") {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Missing delimiter argument in `preg_quote()` call")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(function_call.span()).with_message("Add delimiter as second argument"))
            .with_note("Without delimiter, regex chars may not be properly escaped")
            .with_help("Add delimiter: `preg_quote($str, '/')`");

        ctx.collector.report(issue);
    }
}
