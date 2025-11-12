use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferStaticClosureRule {
    meta: &'static RuleMeta,
    cfg: PreferStaticClosureConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferStaticClosureConfig {
    pub level: Level,
}

impl Default for PreferStaticClosureConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferStaticClosureConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferStaticClosureRule {
    type Config = PreferStaticClosureConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Static Closure",
            code: "prefer-static-closure",
            description: indoc! {"
                Suggests adding the `static` keyword to closures and arrow functions that don't use `$this`.

                Static closures don't bind `$this`, making them more memory-efficient and their intent clearer.
            "},
            good_example: indoc! {r#"
                <?php

                class Foo {
                    public function bar() {
                        // Static closure - doesn't use $this
                        $fn = static fn($x) => $x * 2;

                        // Non-static - uses $this
                        $fn2 = fn() => $this->doSomething();

                        // Static function - doesn't use $this
                        $closure = static function($x) {
                            return $x * 2;
                        };
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                class Foo {
                    public function bar() {
                        // Missing static - doesn't use $this
                        $fn = fn($x) => $x * 2;

                        // Missing static - doesn't use $this
                        $closure = function($x) {
                            return $x * 2;
                        };
                    }
                }
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Closure, NodeKind::ArrowFunction];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        // Must be inside a class to have $this available
        if ctx.scope.get_class_like_scope().is_none() {
            return;
        }

        match node {
            Node::Closure(closure) => {
                // Already static - skip
                if closure.r#static.is_some() {
                    return;
                }

                // Check if body contains $this
                if contains_this_reference(Node::Block(&closure.body)) {
                    return;
                }

                self.report_issue(ctx, closure.function.span(), "closure");
            }
            Node::ArrowFunction(arrow) => {
                // Already static - skip
                if arrow.r#static.is_some() {
                    return;
                }

                // Check if expression contains $this
                if contains_this_reference(Node::Expression(arrow.expression)) {
                    return;
                }

                self.report_issue(ctx, arrow.r#fn.span(), "arrow function");
            }
            _ => {}
        }
    }
}

impl PreferStaticClosureRule {
    fn report_issue(&self, ctx: &mut LintContext, keyword_span: Span, kind: &str) {
        let issue =
            Issue::new(self.cfg.level(), format!("This {} does not use `$this` and should be declared static.", kind))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(keyword_span)
                        .with_message(format!("add `static` before this {} keyword", kind)),
                )
                .with_note("Static closures are more memory-efficient and make it clear that `$this` is not used.")
                .with_help(format!(
                    "Add the `static` keyword before `{}` to make this {} static.",
                    if kind == "closure" { "function" } else { "fn" },
                    kind
                ));

        ctx.collector.propose(issue, |plan| {
            // Insert "static " before the function/fn keyword
            plan.insert(keyword_span.to_range().start, "static ", SafetyClassification::Safe);
        });
    }
}

fn contains_this_reference<'ast, 'arena>(node: Node<'ast, 'arena>) -> bool {
    // Check current node
    if let Node::Expression(Expression::Variable(Variable::Direct(var))) = node
        && var.name == "$this"
    {
        return true;
    }

    // Don't recurse into nested closures/arrow functions/anonymous classes,
    // or nested declarations (they have their own $this binding)
    match node {
        Node::Closure(_) | Node::ArrowFunction(_) | Node::AnonymousClass(_) => return false,
        node if node.is_declaration() => return false,
        _ => {}
    }

    // Recursively check children
    for child in node.children() {
        if contains_this_reference(child) {
            return true;
        }
    }

    false
}
