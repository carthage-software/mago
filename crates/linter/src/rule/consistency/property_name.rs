use indoc::indoc;
use mago_span::Span;
use schemars::JsonSchema;
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
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::PropertyItem;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PropertyNameRule {
    meta: &'static RuleMeta,
    cfg: PropertyNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PropertyNameConfig {
    pub level: Level,
    pub camel: bool,
    pub either: bool,
}

impl Default for PropertyNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, camel: true, either: false }
    }
}

impl Config for PropertyNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PropertyNameRule {
    type Config = PropertyNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Property Name",
            code: "property-name",
            description: indoc! {"
                Detects class property declarations that do not follow camel or snake naming convention.

                Property names should be in camel case or snake case, depending on the configuration.
            "},
            good_example: indoc! {r"
                <?php

                final class Foo {
                    public string $myProperty;

                    public function __construct(
                        public int $myPromotedProperty,
                    ) {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                final class Foo {
                    public string $My_Property;

                    public function __construct(
                        public int $My_Promoted_Property,
                    ) {}
                }
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::PropertyItem, NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match node {
            Node::PropertyItem(property_item) => {
                self.check_property_item(ctx, property_item);
            }
            Node::FunctionLikeParameter(parameter) => {
                self.check_promoted_property(ctx, parameter);
            }
            _ => {}
        }
    }
}

impl PropertyNameRule {
    fn check_property_item<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, property_item: &PropertyItem<'arena>) {
        let variable = property_item.variable();

        self.check_property_name(ctx, variable.name, variable.span());
    }

    fn check_promoted_property<'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        parameter: &FunctionLikeParameter<'arena>,
    ) {
        if !parameter.is_promoted_property() {
            return;
        }

        self.check_property_name(ctx, parameter.variable.name, parameter.variable.span());
    }

    fn check_property_name(&self, ctx: &mut LintContext<'_, '_>, name: &str, span: Span) {
        let name_without_dollar = name.strip_prefix('$').unwrap_or(name);

        if self.cfg.either {
            if !is_camel_case(name_without_dollar) && !is_snake_case(name_without_dollar) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Property name `{name}` should be in either camel case or snake case."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Property `{name}` is declared here")),
                    )
                    .with_note(format!(
                        "The property name `{name}` does not follow either camel case or snake naming convention."
                    ))
                    .with_help(format!(
                        "Consider renaming it to `${}` or `${}` to adhere to the naming convention.",
                        to_camel_case(name_without_dollar),
                        to_snake_case(name_without_dollar)
                    )),
                );
            }

            return;
        }

        if self.cfg.camel && !is_camel_case(name_without_dollar) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Property name `{name}` should be in camel case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Property `{name}` is declared here")),
                    )
                    .with_note(format!("The property name `{name}` does not follow camel naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_camel_case(name_without_dollar)
                    )),
            );
        } else if !self.cfg.camel && !is_snake_case(name_without_dollar) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Property name `{name}` should be in snake case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(span).with_message(format!("Property `{name}` is declared here")),
                    )
                    .with_note(format!("The property name `{name}` does not follow snake naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `${}` to adhere to the naming convention.",
                        to_snake_case(name_without_dollar)
                    )),
            );
        }
    }
}
