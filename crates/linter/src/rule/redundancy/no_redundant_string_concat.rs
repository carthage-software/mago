use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasPosition;
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
pub struct NoRedundantStringConcatRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantStringConcatConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        let Binary { lhs, operator, rhs } = binary;

        if !operator.is_concatenation() {
            return;
        }

        let (Expression::Literal(Literal::String(left)), Expression::Literal(Literal::String(right))) = (lhs, rhs)
        else {
            return;
        };

        if left.kind == right.kind {
            if ctx.source_file.line_number(left.offset()) != ctx.source_file.line_number(right.offset()) {
                return;
            }

            let dangerous = matches!(&right.raw.as_bytes()[1..], [b'{', ..]);
            if dangerous {
                return;
            }

            let issue = Issue::new(self.cfg.level(), "String concatenation can be simplified.")
                .with_code(self.meta.code)
                .with_annotations(vec![
                    Annotation::primary(operator.span()).with_message("Redundant string concatenation"),
                    Annotation::secondary(left.span()).with_message("Left string"),
                    Annotation::secondary(right.span()).with_message("Right string"),
                ])
                .with_help("Consider combining these strings into a single string.");

            ctx.collector.report(issue);
        }
    }
}
