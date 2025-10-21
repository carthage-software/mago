use indoc::indoc;
use mago_atom::atom;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::AtomSet;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::security::is_user_input;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const PRINTF_FUNCTION: &str = "printf";

#[derive(Debug, Clone)]
pub struct TaintedDataToSinkRule {
    meta: &'static RuleMeta,
    cfg: TaintedDataToSinkConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TaintedDataToSinkConfig {
    pub level: Level,
    pub known_sink_functions: AtomSet,
}

impl Default for TaintedDataToSinkConfig {
    fn default() -> Self {
        Self { level: Level::Error, known_sink_functions: AtomSet::from_iter([atom(PRINTF_FUNCTION)]) }
    }
}

impl Config for TaintedDataToSinkConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TaintedDataToSinkRule {
    type Config = TaintedDataToSinkConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Tainted Data to Sink",
            code: "tainted-data-to-sink",
            description: indoc! {r#"
                Detects user (tainted) data being passed directly to sink functions or constructs
                (such as `echo`, `print`, or user-defined "log" functions). If these functions emit
                or store data without sanitization, it could lead to Cross-Site Scripting (XSS)
                or other injection attacks.
            "#},
            good_example: indoc! {r#"
                <?php

                // Properly escape data before using a sink like `echo`
                echo htmlspecialchars($_GET['name'] ?? '', ENT_QUOTES, 'UTF-8');
            "#},
            bad_example: indoc! {r#"
                <?php

                // This is considered unsafe:
                echo $_GET['name'] ?? '';
            "#},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Echo, NodeKind::PrintConstruct, NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Echo(echo) => {
                for value in echo.values.iter() {
                    self.check_tainted_data_to_sink(ctx, echo.echo.span, value);
                }
            }
            Node::EchoTag(echo) => {
                for value in echo.values.iter() {
                    self.check_tainted_data_to_sink(ctx, echo.tag, value);
                }
            }
            Node::PrintConstruct(print_construct) => {
                self.check_tainted_data_to_sink(ctx, print_construct.print.span, print_construct.value);
            }
            Node::FunctionCall(function_call) => {
                let sinks = self.cfg.known_sink_functions.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
                if function_call_matches_any(ctx, function_call, &sinks).is_none() {
                    return;
                }

                for argument in function_call.argument_list.arguments.iter() {
                    self.check_tainted_data_to_sink(ctx, function_call.function.span(), argument.value());
                }
            }
            _ => (),
        }
    }
}

impl TaintedDataToSinkRule {
    fn check_tainted_data_to_sink<'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        used_in: Span,
        value: &Expression<'arena>,
    ) {
        if !is_user_input(value) {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Tainted data passed to a sink function/construct.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(value.span()).with_message("This value originates from user input."))
                .with_annotation(
                    Annotation::secondary(used_in.span()).with_message("Data is passed here without sanitization."),
                )
                .with_note("Tainted (user-supplied) data must be sanitized or escaped before being passed to sinks, or risk injection vulnerabilities.")
                .with_help("Ensure the data is validated or escaped prior to using this sink.")
        );
    }
}
