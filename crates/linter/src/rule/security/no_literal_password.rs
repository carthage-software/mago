use indoc::indoc;
use mago_span::Span;
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
use crate::rule::utils::security::get_password;
use crate::rule::utils::security::is_password;
use crate::rule::utils::security::is_password_literal;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoLiteralPasswordRule {
    meta: &'static RuleMeta,
    cfg: NoLiteralPasswordConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoLiteralPasswordConfig {
    pub level: Level,
}

impl Default for NoLiteralPasswordConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoLiteralPasswordConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoLiteralPasswordRule {
    type Config = NoLiteralPasswordConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Literal Password",
            code: "no-literal-password",
            description: indoc! {r#"
                Detects the use of literal values for passwords or sensitive data.
                Storing passwords or sensitive information as literals in code is a security risk
                and should be avoided. Use environment variables or secure configuration management instead.
            "#},
            good_example: indoc! {r#"
                <?php

                $password = getenv('DB_PASSWORD');
            "#},
            bad_example: indoc! {r#"
                <?php

                $password = "supersecret";
            "#},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Assignment,
            NodeKind::ArrayElement,
            NodeKind::ConstantItem,
            NodeKind::ClassLikeConstantItem,
            NodeKind::PropertyConcreteItem,
            NodeKind::FunctionLikeParameter,
            NodeKind::NamedArgument,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Assignment(assignment) => {
                let Some(password) = get_password(assignment.lhs) else {
                    return;
                };

                check(password, assignment.rhs, ctx, self)
            }
            Node::ArrayElement(array_element) => {
                let ArrayElement::KeyValue(kv) = array_element else {
                    return;
                };

                let is_key_a_password = matches!(
                    kv.key,
                    Expression::Literal(Literal::String(literal_string)) if is_password_literal( literal_string),
                );

                if !is_key_a_password {
                    return;
                }

                check(kv.key.span(), kv.value, ctx, self)
            }
            Node::ConstantItem(constant_item) => {
                if !is_password(constant_item.name.value) {
                    return;
                }

                check(constant_item.name.span, &constant_item.value, ctx, self)
            }
            Node::ClassLikeConstantItem(class_like_constant_item) => {
                if !is_password(class_like_constant_item.name.value) {
                    return;
                }

                check(class_like_constant_item.name.span, &class_like_constant_item.value, ctx, self)
            }
            Node::PropertyConcreteItem(property_concrete_item) => {
                if !is_password(&property_concrete_item.variable.name[1..]) {
                    return;
                }

                check(property_concrete_item.variable.span, &property_concrete_item.value, ctx, self)
            }
            Node::FunctionLikeParameter(function_like_parameter) => {
                let Some(default_value) = function_like_parameter.default_value.as_ref() else {
                    return;
                };

                if !is_password(&function_like_parameter.variable.name[1..]) {
                    return;
                }

                check(function_like_parameter.variable.span, &default_value.value, ctx, self)
            }
            Node::NamedArgument(named_argument) => {
                if !is_password(named_argument.name.value) {
                    return;
                }

                check(named_argument.name.span, &named_argument.value, ctx, self)
            }
            _ => {}
        }
    }
}

#[inline]
fn check<'arena>(
    name: Span,
    value: &Expression<'arena>,
    ctx: &mut LintContext<'_, 'arena>,
    rule: &NoLiteralPasswordRule,
) {
    let is_literal_password = match value {
        Expression::Literal(Literal::String(literal_string)) => literal_string.raw.len() > 2,
        Expression::Literal(Literal::Integer(_)) => true,
        _ => false,
    };

    if !is_literal_password {
        return;
    }

    let issue = Issue::new(rule.cfg.level(), "Literal passwords or sensitive data should not be stored in code.")
        .with_code(rule.meta.code)
        .with_annotation(Annotation::primary(name).with_message("Sensitive item found here."))
        .with_annotation(Annotation::secondary(value.span()).with_message("Literal value used here."))
        .with_note("Storing passwords or sensitive information as literals in code is a security risk.")
        .with_note("This can lead to accidental exposure of sensitive data in version control or logs.")
        .with_help("Use environment variables or secure configuration management instead.");

    ctx.collector.report(issue);
}
