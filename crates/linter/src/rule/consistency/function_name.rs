use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_casing::is_camel_case;
use mago_casing::is_snake_case;
use mago_casing::to_camel_case;
use mago_casing::to_snake_case;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use mago_bytes::BytesDisplay;

#[derive(Debug, Clone)]
pub struct FunctionNameRule {
    meta: &'static RuleMeta,
    cfg: FunctionNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
            good_example: indoc! {r"
                <?php

                function my_function() {}
            "},
            bad_example: indoc! {r"
                <?php

                function MyFunction() {}

                function My_Function() {}
            "},
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Function(function) = node else { return };

        let Some(name) = std::str::from_utf8(function.name.value).ok() else { return };
        let fqfn = BytesDisplay(ctx.lookup_name(&function.name));

        if self.cfg.either {
            if !is_camel_case(name) && !is_snake_case(name) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Function name `{name}` should be in either camel case or snake case."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function.name.span())
                            .with_message(format!("Function `{name}` is declared here")),
                    )
                    .with_annotation(
                        Annotation::secondary(function.span())
                            .with_message(format!("Function `{fqfn}` is defined here")),
                    )
                    .with_note(format!(
                        "The function name `{name}` does not follow either camel case or snake naming convention."
                    ))
                    .with_help(format!(
                        "Consider renaming it to `{}` or `{}` to adhere to the naming convention.",
                        String::from_utf8_lossy(&to_camel_case(name)),
                        String::from_utf8_lossy(&to_snake_case(name))
                    )),
                );
            }
        } else if self.cfg.camel {
            if !is_camel_case(name) {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Function name `{name}` should be in camel case."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(function.name.span())
                                .with_message(format!("Function `{name}` is declared here")),
                        )
                        .with_annotation(
                            Annotation::secondary(function.span())
                                .with_message(format!("Function `{fqfn}` is defined here")),
                        )
                        .with_note(format!("The function name `{name}` does not follow camel naming convention."))
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            String::from_utf8_lossy(&to_camel_case(name))
                        )),
                );
            }
        } else if !is_snake_case(name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Function name `{name}` should be in snake case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function.name.span())
                            .with_message(format!("Function `{name}` is declared here")),
                    )
                    .with_annotation(
                        Annotation::secondary(function.span())
                            .with_message(format!("Function `{fqfn}` is defined here")),
                    )
                    .with_note(format!("The function name `{name}` does not follow snake naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        String::from_utf8_lossy(&to_snake_case(name))
                    )),
            );
        }
    }
}
