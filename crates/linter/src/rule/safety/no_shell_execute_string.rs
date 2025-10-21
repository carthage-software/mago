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
pub struct NoShellExecuteStringRule {
    meta: &'static RuleMeta,
    cfg: NoShellExecuteStringConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoShellExecuteStringConfig {
    pub level: Level,
}

impl Default for NoShellExecuteStringConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoShellExecuteStringConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoShellExecuteStringRule {
    type Config = NoShellExecuteStringConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Shell Execute String",
            code: "no-shell-execute-string",
            description: indoc! {"
                Detects the use of shell execute strings (`...`) in PHP code.
            "},
            good_example: indoc! {r#"
                <?php

                $output = shell_exec('ls -l');
            "#},
            bad_example: indoc! {r#"
                <?php

                $output = `ls -l`;
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ShellExecuteString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::ShellExecuteString(shell_execute_string) = node else {
            return;
        };

        let mut is_interpolated = false;
        for part in shell_execute_string.parts.iter() {
            if !matches!(part, StringPart::Literal(..)) {
                is_interpolated = true;

                break;
            }
        }

        let issue = if is_interpolated {
            Issue::new(
                self.cfg.level(),
                "Unsafe use of interpolated shell execute string.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(shell_execute_string.span())
                    .with_message("This shell execute string is interpolated"),
            )
            .with_note("Interpolating shell execute strings (`...`) is a potential security vulnerability, as it allows executing arbitrary shell commands.")
            .with_help("Consider using `shell_exec()` along with `escapeshellarg()` or `escapeshellcmd()` to escape arguments instead.")
        } else {
            Issue::new(self.cfg.level(), "Potentilly unsafe use of shell execute string.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(shell_execute_string.span()).with_message("Shell execute string used here"),
                )
                .with_note("Shell execute strings (`...`) can often be replaced with safer alternatives.")
                .with_help("Consider using `shell_exec()` instead.")
        };

        ctx.collector.report(issue);
    }
}
