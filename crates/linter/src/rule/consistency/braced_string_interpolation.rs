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
pub struct BracedStringInterpolationRule {
    meta: &'static RuleMeta,
    cfg: BracedStringInterpolationConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct BracedStringInterpolationConfig {
    pub level: Level,
}

impl Default for BracedStringInterpolationConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for BracedStringInterpolationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for BracedStringInterpolationRule {
    type Config = BracedStringInterpolationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Braced String Interpolation",
            code: "braced-string-interpolation",
            description: indoc! {"
                Enforces the use of curly braces around variables within string interpolation.

                Using curly braces (`{$variable}`) within interpolated strings ensures clarity and avoids potential ambiguity,
                especially when variables are followed by alphanumeric characters. This rule promotes consistent and predictable code.
            "},
            good_example: indoc! {r#"
                <?php

                $a = "Hello, {$name}!";
                $b = "Hello, {$name}!";
                $c = "Hello, {$$name}!";
                $d = "Hello, {${$object->getMethod()}}!";
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = "Hello, $name!";
                $b = "Hello, ${name}!";
                $c = "Hello, ${$name}!";
                $d = "Hello, ${$object->getMethod()}!";
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::CompositeString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::CompositeString(composite_string) = node else {
            return;
        };

        let mut unbraced_expressions = vec![];
        for part in composite_string.parts().iter() {
            let StringPart::Expression(expression) = part else {
                continue;
            };

            unbraced_expressions.push((
                expression.span(),
                !matches!(
                    expression,
                    Expression::Variable(Variable::Indirect(variable))
                    if matches!(
                        variable.expression,
                        Expression::Identifier(_) | Expression::Variable(_)
                    )
                ),
            ));
        }

        if unbraced_expressions.is_empty() {
            return;
        }

        let mut issue = Issue::new(self.cfg.level(), "Unbraced variable in string interpolation.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(composite_string.span())
                    .with_message("String interpolation contains unbraced variables"),
            );

        for (span, _) in &unbraced_expressions {
            issue = issue.with_annotation(
                Annotation::secondary(*span).with_message("Variable should be enclosed in curly braces"),
            );
        }

        issue = issue.with_note("Using curly braces around variables in interpolated strings improves readability and prevents potential parsing issues.")
            .with_help("Wrap the variable in curly braces, e.g., `{$variable}`.");

        ctx.collector.report(issue);
    }
}
