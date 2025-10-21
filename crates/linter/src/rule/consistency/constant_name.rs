use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_constant_case;
use mago_casing::to_constant_case;
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
pub struct ConstantNameRule {
    meta: &'static RuleMeta,
    cfg: ConstantNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConstantNameConfig {
    pub level: Level,
}

impl Default for ConstantNameConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for ConstantNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ConstantNameRule {
    type Config = ConstantNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Constant Name",
            code: "constant-name",
            description: indoc! {"
                Detects constant declarations that do not follow constant naming convention.

                Constant names should be in constant case, also known as UPPER_SNAKE_CASE.
            "},
            good_example: indoc! {r#"
                <?php

                const MY_CONSTANT = 42;

                class MyClass {
                    public const int MY_CONSTANT = 42;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                const myConstant = 42;
                const my_constant = 42;
                const My_Constant = 42;

                class MyClass {
                    public const int myConstant = 42;
                    public const int my_constant = 42;
                    public const int My_Constant = 42;
                }
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Constant, NodeKind::ClassLikeConstant];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Constant(constant) => {
                for item in constant.items.iter() {
                    let name = item.name.value;
                    if !is_constant_case(name) {
                        ctx.collector.report(
                            Issue::new(
                                self.cfg.level(),
                                format!("Constant name `{}` should be in constant case.", name),
                            )
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(item.name.span())
                                    .with_message(format!("Constant item `{}` is declared here", name)),
                            )
                            .with_note(format!(
                                "The constant name `{}` does not follow constant naming convention.",
                                name
                            ))
                            .with_help(format!(
                                "Consider renaming it to `{}` to adhere to the naming convention.",
                                to_constant_case(name)
                            )),
                        );
                    }
                }
            }
            Node::ClassLikeConstant(class_like_constant) => {
                for item in class_like_constant.items.iter() {
                    let name = item.name.value;

                    if !is_constant_case(name) {
                        ctx.collector.report(
                            Issue::new(
                                self.cfg.level(),
                                format!("Constant name `{}` should be in constant case.", name),
                            )
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(item.name.span())
                                    .with_message(format!("Constant item `{}` is declared here", name)),
                            )
                            .with_note(format!(
                                "The constant name `{}` does not follow constant naming convention.",
                                name
                            ))
                            .with_help(format!(
                                "Consider renaming it to `{}` to adhere to the naming convention.",
                                to_constant_case(name)
                            )),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
