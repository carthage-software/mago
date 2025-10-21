use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::*;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoVoidReferenceReturnRule {
    meta: &'static RuleMeta,
    cfg: NoVoidReferenceReturnConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoVoidReferenceReturnConfig {
    pub level: Level,
}

impl Default for NoVoidReferenceReturnConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoVoidReferenceReturnConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoVoidReferenceReturnRule {
    type Config = NoVoidReferenceReturnConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Void Reference Return",
            code: "no-void-reference-return",
            description: indoc! {"
                Detects functions, methods, closures, arrow functions, and set property hooks that return by reference from a void function.
                Such functions are considered deprecated; returning by reference from a void function is deprecated since PHP 8.0.
            "},
            good_example: indoc! {r#"
                <?php

                function &foo(): string {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function &foo(): void {
                    // ...
                }
            "#},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP82)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Function, NodeKind::Method, NodeKind::Closure, NodeKind::ArrowFunction, NodeKind::PropertyHook];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Function(function) => {
                let Some(amperstand) = function.ampersand.as_ref() else {
                    return;
                };

                let Some(return_type) = function.return_type_hint.as_ref() else {
                    return;
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return;
                }

                self.report(ctx, "function", function.span(), amperstand, false);
            }
            Node::Method(method) => {
                let Some(amperstand) = method.ampersand.as_ref() else {
                    return;
                };

                let Some(return_type) = method.return_type_hint.as_ref() else {
                    return;
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return;
                };

                self.report(ctx, "method", method.span(), amperstand, false);
            }
            Node::Closure(closure) => {
                let Some(amperstand) = closure.ampersand.as_ref() else {
                    return;
                };

                let Some(return_type) = closure.return_type_hint.as_ref() else {
                    return;
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return;
                };

                self.report(ctx, "closure", closure.span(), amperstand, false);
            }
            Node::ArrowFunction(arrow_function) => {
                let Some(amperstand) = arrow_function.ampersand.as_ref() else {
                    return;
                };

                let Some(return_type) = arrow_function.return_type_hint.as_ref() else {
                    return;
                };

                if !matches!(return_type.hint, Hint::Void(_)) {
                    return;
                };

                self.report(ctx, "arrow function", arrow_function.span(), amperstand, false);
            }
            Node::PropertyHook(property_hook) => {
                if "set" != property_hook.name.value {
                    return;
                }

                let Some(amperstand) = property_hook.ampersand.as_ref() else {
                    return;
                };

                self.report(ctx, "set property hook", property_hook.span(), amperstand, true);
            }
            _ => (),
        }
    }
}

impl NoVoidReferenceReturnRule {
    fn report(&self, ctx: &mut LintContext, kind: &'static str, span: Span, ampersand: &Span, is_set_hook: bool) {
        let message = if !is_set_hook {
            format!("Returning by reference from a void {} is deprecated since PHP 8.0.", kind)
        } else {
            "Returning by reference from a set property hook is deprecated since PHP 8.0.".to_string()
        };

        let issue = Issue::new(self.cfg.level(), message)
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(*ampersand)
                    .with_message(format!("The `&` indicates that the {} returns by reference", kind)),
            )
            .with_annotation(Annotation::secondary(span))
            .with_help("Consider removing the `&` to comply with PHP 8.0 standards and avoid future issues.");

        ctx.collector.report(issue);
    }
}
