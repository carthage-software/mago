use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct LiteralNamedArgumentRule {
    meta: &'static RuleMeta,
    cfg: LiteralNamedArgumentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LiteralNamedArgumentConfig {
    pub level: Level,
}

impl Default for LiteralNamedArgumentConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for LiteralNamedArgumentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LiteralNamedArgumentRule {
    type Config = LiteralNamedArgumentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Literal Named Argument",
            code: "literal-named-argument",
            description: indoc! {r#"
                Enforces that literal values used as arguments in function or method calls
                are passed as **named arguments**.

                This improves readability by clarifying the purpose of the literal value at the call site.
                It is particularly helpful for boolean flags, numeric constants, and `null` values
                where the intent is often ambiguous without the parameter name.
            "#},
            good_example: indoc! {r#"
                <?php

                function set_option(string $key, bool $enable_feature) {}

                set_option(key: 'feature_x', enable_feature: true); // ✅ clear intent
            "#},
            bad_example: indoc! {r#"
                <?php

                function set_option(string $key, bool $enable_feature) {}

                set_option('feature_x', true); // ❌ intent unclear
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::PositionalArgument];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::PositionalArgument(positional_argument) = node else {
            return;
        };

        let Expression::Literal(literal) = &positional_argument.value else {
            return;
        };

        let literal_value = match literal {
            Literal::String(lit_str) => lit_str.raw,
            Literal::Integer(lit_int) => lit_int.raw,
            Literal::Float(lit_float) => lit_float.raw,
            Literal::True(_) => "true",
            Literal::False(_) => "false",
            Literal::Null(_) => "null",
        };

        ctx.collector.report(
            Issue::new(
                self.cfg.level,
                format!("Literal argument `{literal_value}` should be passed as a named argument for clarity."),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(literal.span()).with_message("This literal is being passed positionally."),
            )
            .with_note(
                "Passing literals positionally can make code less clear, especially with booleans, numbers, or `null`.",
            )
            .with_help(format!("Consider using a named argument instead: `function_name(param: {literal_value})`.")),
        );
    }
}
