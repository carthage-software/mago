use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_camel_case;
use mago_casing::is_snake_case;
use mago_casing::to_camel_case;
use mago_casing::to_snake_case;
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
pub struct FunctionNameRule {
    meta: &'static RuleMeta,
    cfg: FunctionNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct FunctionNameConfig {
    pub level: Level,
    pub camel: bool,
    pub either: bool,
}

impl Default for FunctionNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, camel: false, either: false }
    }
}

impl Config for FunctionNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for FunctionNameRule {
    type Config = FunctionNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Function Name",
            code: "function-name",
            description: indoc! {"
                Detects function declarations that do not follow camel or snake naming convention.

                Function names should be in camel case or snake case, depending on the configuration.
            "},
            good_example: indoc! {r#"
                <?php

                function my_function() {}
            "#},
            bad_example: indoc! {r#"
                <?php

                function MyFunction() {}

                function My_Function() {}
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Function];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Function(function) = node else { return };

        let name = function.name.value;
        let fqfn = ctx.lookup_name(&function.name);

        if self.cfg.either {
            if !is_camel_case(name) && !is_snake_case(name) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Function name `{}` should be in either camel case or snake case.", name),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function.name.span())
                            .with_message(format!("Function `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(function.span())
                            .with_message(format!("Function `{}` is defined here", fqfn)),
                    )
                    .with_note(format!(
                        "The function name `{}` does not follow either camel case or snake naming convention.",
                        name
                    ))
                    .with_help(format!(
                        "Consider renaming it to `{}` or `{}` to adhere to the naming convention.",
                        to_camel_case(name),
                        to_snake_case(name)
                    )),
                );
            }
        } else if self.cfg.camel {
            if !is_camel_case(name) {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Function name `{}` should be in camel case.", name))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(function.name.span())
                                .with_message(format!("Function `{}` is declared here", name)),
                        )
                        .with_annotation(
                            Annotation::secondary(function.span())
                                .with_message(format!("Function `{}` is defined here", fqfn)),
                        )
                        .with_note(format!("The function name `{}` does not follow camel naming convention.", name))
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            to_camel_case(name)
                        )),
                );
            }
        } else if !is_snake_case(name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Function name `{}` should be in snake case.", name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function.name.span())
                            .with_message(format!("Function `{}` is declared here", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(function.span())
                            .with_message(format!("Function `{}` is defined here", fqfn)),
                    )
                    .with_note(format!("The function name `{}` does not follow snake naming convention.", name))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        to_snake_case(name)
                    )),
            );
        }
    }
}
