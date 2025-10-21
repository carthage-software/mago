use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
pub struct ExplicitOctalRule {
    meta: &'static RuleMeta,
    cfg: ExplicitOctalConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExplicitOctalConfig {
    pub level: Level,
}

impl Default for ExplicitOctalConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for ExplicitOctalConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExplicitOctalRule {
    type Config = ExplicitOctalConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Explicit Octal",
            code: "explicit-octal",
            description: indoc! {"
                Detects implicit octal numeral notation and suggests replacing it with explicit octal numeral notation.
            "},
            good_example: indoc! {r#"
                <?php

                $a = 0o123;
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = 0123;
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP81)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LiteralInteger];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::LiteralInteger(literal_integer) = node else {
            return;
        };

        let literal_text = literal_integer.raw;
        if !literal_text.starts_with('0') {
            return;
        }

        if !literal_text.as_bytes().get(1).copied().is_some_and(|c| {
            // check for `0o`, `0x`, or `0b` prefix
            c != b'o' && c != b'O' && c != b'x' && c != b'X' && c != b'b' && c != b'B'
        }) {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use explicit octal numeral notation.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(literal_integer.span()).with_message("Implicit octal numeral notation used here"),
            )
            .with_note("Using `0o` makes the octal intent explicit and avoids confusion with other formats.")
            .with_help("Replace the leading `0` with `0o` to make the octal intent explicit.");

        ctx.collector.report(issue);
    }
}
