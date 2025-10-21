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
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::format_replacements;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PslOutputRule {
    meta: &'static RuleMeta,
    cfg: PslOutputConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslOutputConfig {
    pub level: Level,
}

impl Default for PslOutputConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for PslOutputConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslOutputRule {
    type Config = PslOutputConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Output",
            code: "psl-output",
            description: indoc! {"
                This rule enforces the usage of Psl output functions over their PHP counterparts.
                Psl output functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                Psl\IO\write_line("Hello, world!");
            "#},
            bad_example: indoc! {r#"
                <?php

                echo "Hello, world!";
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Psl),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Echo, NodeKind::PrintConstruct, NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let (used_directive, is_stdout) = match node {
            Node::Echo(_) => ("echo", true),
            Node::PrintConstruct(_) => ("print", true),
            Node::FunctionCall(call) => {
                if function_call_matches(ctx, call, "printf") {
                    ("printf", true)
                } else if !function_call_matches(ctx, call, "fwrite") {
                    let Some(arg) = call.argument_list.arguments.get(0) else { return };
                    let Expression::ConstantAccess(constant) = arg.value() else { return };

                    if constant.name.value().eq_ignore_ascii_case("STDOUT") {
                        ("fwrite", true)
                    } else if constant.name.value().eq_ignore_ascii_case("STDERR") {
                        ("fwrite", false)
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            _ => return,
        };

        let replacements = if is_stdout { &STDOUT_FUNCTIONS } else { &STDERR_FUNCTIONS };

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use the Psl output function instead of the PHP counterpart.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(node.span()).with_message(format!("Using PHP's `{}`", used_directive)),
            )
            .with_note("Psl output functions are preferred because they are type-safe and provide more consistent behavior.")
            .with_help(format!(
                "Use {} instead.",
                format_replacements(replacements)
            )),
        );
    }
}

const STDOUT_FUNCTIONS: [&str; 2] = ["Psl\\IO\\write", "Psl\\IO\\write_line"];
const STDERR_FUNCTIONS: [&str; 2] = ["Psl\\IO\\write_error", "Psl\\IO\\write_error_line"];
