use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::*;
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
pub struct ParameterTypeRule {
    meta: &'static RuleMeta,
    cfg: ParameterTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ParameterTypeConfig {
    pub level: Level,
    pub ignore_closure: bool,
    pub ignore_arrow_function: bool,
}

impl Default for ParameterTypeConfig {
    fn default() -> Self {
        Self { level: Level::Warning, ignore_closure: false, ignore_arrow_function: false }
    }
}

impl Config for ParameterTypeConfig {
    fn default_enabled() -> bool {
        // This rule is disabled by default due to its deprecation.
        // Use the analyzer's `check-missing-type-hints` setting instead.
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ParameterTypeRule {
    type Config = ParameterTypeConfig;

    fn deprecated() -> bool {
        true
    }

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Parameter Type",
            code: "parameter-type",
            description: indoc! {"
                Detects parameters that are missing a type hint.
            "},
            good_example: indoc! {r#"
                <?php

                function foo(string $bar): void
                {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function foo($bar): void
                {
                    // ...
                }
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP70)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Function,
            NodeKind::Closure,
            NodeKind::ArrowFunction,
            NodeKind::Interface,
            NodeKind::Class,
            NodeKind::Enum,
            NodeKind::Trait,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Function(function) => {
                for parameter in function.parameter_list.parameters.iter() {
                    self.check_function_like_parameter(parameter, ctx);
                }
            }
            Node::Closure(closure) => {
                if self.cfg.ignore_closure {
                    return;
                }

                for parameter in closure.parameter_list.parameters.iter() {
                    self.check_function_like_parameter(parameter, ctx);
                }
            }
            Node::ArrowFunction(arrow_function) => {
                if self.cfg.ignore_arrow_function {
                    return;
                }

                for parameter in arrow_function.parameter_list.parameters.iter() {
                    self.check_function_like_parameter(parameter, ctx);
                }
            }
            Node::Interface(interface) => {
                self.check_class_like_members(interface.members.as_slice(), ctx);
            }
            Node::Class(class) => {
                self.check_class_like_members(class.members.as_slice(), ctx);
            }
            Node::Enum(r#enum) => {
                self.check_class_like_members(r#enum.members.as_slice(), ctx);
            }
            Node::Trait(r#trait) => {
                self.check_class_like_members(r#trait.members.as_slice(), ctx);
            }
            _ => (),
        }
    }
}

impl ParameterTypeRule {
    fn check_function_like_parameter(&self, function_like_parameter: &FunctionLikeParameter, ctx: &mut LintContext) {
        if function_like_parameter.hint.is_some() || function_like_parameter.variable.name.starts_with("$_") {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                format!("Parameter `{}` is missing a type hint.", function_like_parameter.variable.name),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_like_parameter.span())
                    .with_message(format!("Parameter `{}` is declared here", function_like_parameter.variable.name)),
            )
            .with_note("Type hints improve code readability and help prevent type-related errors.")
            .with_help(format!(
                "Consider adding a type hint to parameter `{}`.",
                function_like_parameter.variable.name
            )),
        );
    }

    fn check_class_like_members(&self, members: &[ClassLikeMember], ctx: &mut LintContext) {
        for member in members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            for parameter in method.parameter_list.parameters.iter() {
                self.check_function_like_parameter(parameter, ctx);
            }
        }
    }
}
