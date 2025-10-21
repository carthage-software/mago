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
pub struct ReturnTypeRule {
    meta: &'static RuleMeta,
    cfg: ReturnTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReturnTypeConfig {
    pub level: Level,
    pub ignore_closure: bool,
    pub ignore_arrow_function: bool,
}

impl Default for ReturnTypeConfig {
    fn default() -> Self {
        Self { level: Level::Warning, ignore_closure: false, ignore_arrow_function: false }
    }
}

impl Config for ReturnTypeConfig {
    fn default_enabled() -> bool {
        // This rule is disabled by default due to its deprecation.
        // Use the analyzer's `check-missing-type-hints` setting instead.
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ReturnTypeRule {
    type Config = ReturnTypeConfig;

    fn deprecated() -> bool {
        true
    }

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Return Type",
            code: "return-type",
            description: indoc! {"
                Detects functions, methods, closures, and arrow functions that are missing a return type hint.
            "},
            good_example: indoc! {r#"
                <?php

                function foo(): int {
                    return 42;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function foo() {
                    return 42;
                }
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP70)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Function, NodeKind::Closure, NodeKind::ArrowFunction, NodeKind::Method];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Function(function) => {
                if function.return_type_hint.is_some() {
                    return;
                }

                let function_name = function.name.value;
                let function_fqn = ctx.lookup_name(&function.name);

                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Function `{}` is missing a return type hint.", function_name),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function.span())
                            .with_message(format!("Function `{}` defined here", function_fqn)),
                    )
                    .with_note("Type hints improve code readability and help prevent type-related errors.")
                    .with_help(format!("Consider adding a return type hint to function `{}`.", function_name)),
                );
            }
            Node::Closure(closure) => {
                if closure.return_type_hint.is_some() {
                    return;
                }

                if self.cfg.ignore_closure {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level(), "Closure is missing a return type hint.")
                        .with_code(self.meta.code)
                        .with_annotation(Annotation::primary(closure.span()).with_message("Closure defined here"))
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help("Consider adding a return type hint to the closure."),
                );
            }
            Node::ArrowFunction(arrow_function) => {
                if arrow_function.return_type_hint.is_some() {
                    return;
                }

                if self.cfg.ignore_arrow_function {
                    return;
                }

                ctx.collector.report(
                    Issue::new(self.cfg.level(), "Arrow function is missing a return type hint.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(arrow_function.span()).with_message("Arrow function defined here"),
                        )
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help("Consider adding a return type hint to the arrow function."),
                );
            }
            Node::Method(method) => {
                if method.return_type_hint.is_some() {
                    return;
                }

                if "__construct" == method.name.value || "__destruct" == method.name.value {
                    return;
                }

                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Method `{}` is missing a return type hint.", method.name.value),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(method.span())
                            .with_message(format!("Method `{}` defined here", method.name.value)),
                    )
                    .with_note("Type hints improve code readability and help prevent type-related errors.")
                    .with_help(format!("Consider adding a return type hint to method `{}`.", method.name.value)),
                );
            }
            _ => (),
        }
    }
}
