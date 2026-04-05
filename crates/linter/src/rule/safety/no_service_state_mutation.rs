use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::ast::access::Access;
use mago_syntax::ast::ast::expression::Expression;
use mago_syntax::ast::ast::variable::Variable;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::scope::FunctionLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoServiceStateMutationRule {
    meta: &'static RuleMeta,
    cfg: NoServiceStateMutationConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoServiceStateMutationConfig {
    pub level: Level,
    pub include_namespaces: Vec<String>,
    pub exclude_namespaces: Vec<String>,
    pub allowed_methods: Vec<String>,
}

impl Default for NoServiceStateMutationConfig {
    fn default() -> Self {
        Self {
            level: Level::Warning,
            include_namespaces: vec!["App\\".to_string()],
            exclude_namespaces: vec![
                "App\\Entity\\".to_string(),
                "App\\DTO\\".to_string(),
                "App\\ValueObject\\".to_string(),
            ],
            allowed_methods: vec!["__construct".to_string(), "reset".to_string()],
        }
    }
}

impl Config for NoServiceStateMutationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

/// Returns `true` if the given expression ultimately refers to `$this->property`,
/// possibly through nested array access or array append operations.
fn is_this_property_mutation<'arena>(expr: &Expression<'arena>) -> Option<Span> {
    match expr {
        Expression::Access(Access::Property(prop)) => {
            if is_this(prop.object) {
                Some(expr.span())
            } else {
                None
            }
        }
        Expression::ArrayAccess(access) => is_this_property_mutation(access.array),
        Expression::ArrayAppend(append) => is_this_property_mutation(append.array),
        _ => None,
    }
}

/// Returns `true` if the expression is `$this`.
fn is_this(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Variable(Variable::Direct(var)) if var.name == "$this")
}

impl LintRule for NoServiceStateMutationRule {
    type Config = NoServiceStateMutationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Service State Mutation",
            code: "no-service-state-mutation",
            description: indoc! {r"
                Detects mutations to `$this->property` inside service methods.

                In worker-mode PHP runtimes (FrankenPHP, RoadRunner, Swoole), services persist across
                requests. Mutating `$this->property` in a service method introduces shared mutable state
                that leaks between requests, leading to subtle and hard-to-reproduce bugs.

                Mutations include direct assignment (`$this->count = 0`), compound assignment
                (`$this->count += 1`), increment/decrement (`$this->count++`, `++$this->count`),
                array append (`$this->items[] = $item`), and `unset($this->cache)`.

                The `__construct` and `reset` methods are allowed by default.
            "},
            good_example: indoc! {r"
                <?php

                namespace App\Service;

                final class InvoiceService
                {
                    public function __construct(
                        private readonly InvoiceRepository $repository,
                    ) {}

                    public function process(Invoice $invoice): void
                    {
                        $total = $invoice->getTotal();
                        $this->repository->save($invoice);
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                namespace App\Service;

                final class InvoiceService
                {
                    private int $processedCount = 0;

                    public function process(Invoice $invoice): void
                    {
                        $this->processedCount++;
                    }
                }
            "},
            category: Category::Safety,
            requirements: RuleRequirements::Integration(Integration::Symfony),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Assignment,
            NodeKind::UnaryPrefix,
            NodeKind::UnaryPostfix,
            NodeKind::Unset,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        // Must be inside a method; skip allowed methods.
        let Some(FunctionLikeScope::Method(method_name)) = ctx.scope.get_function_like_scope() else {
            return;
        };

        if self.cfg.allowed_methods.iter().any(|m| m == method_name) {
            return;
        }

        // Must be inside a class.
        if ctx.scope.get_class_like_scope().is_none() {
            return;
        }

        // Check namespace filters.
        let namespace = ctx.scope.get_namespace();
        if !namespace.is_empty() {
            let in_include = self.cfg.include_namespaces.iter().any(|ns| namespace.starts_with(ns.as_str()));
            if !in_include {
                return;
            }

            let in_exclude = self.cfg.exclude_namespaces.iter().any(|ns| namespace.starts_with(ns.as_str()));
            if in_exclude {
                return;
            }
        } else {
            // No namespace means not in any included namespace.
            return;
        }

        // Extract the mutated expression and check if it involves `$this->property`.
        let mutation_span = match node {
            Node::Assignment(assignment) => is_this_property_mutation(assignment.lhs),
            Node::UnaryPrefix(prefix) => {
                if prefix.operator.is_increment_or_decrement() {
                    is_this_property_mutation(prefix.operand)
                } else {
                    None
                }
            }
            Node::UnaryPostfix(postfix) => is_this_property_mutation(postfix.operand),
            Node::Unset(unset) => {
                let mut found = None;
                for value in unset.values.iter() {
                    if let Some(span) = is_this_property_mutation(value) {
                        found = Some(span);
                        break;
                    }
                }
                found
            }
            _ => None,
        };

        let Some(span) = mutation_span else {
            return;
        };

        let issue = Issue::new(self.cfg.level, "Service state mutation detected.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message("`$this->` property is mutated here"))
            .with_note("In worker-mode runtimes (FrankenPHP, RoadRunner, Swoole), services persist across requests.")
            .with_note("Mutating `$this->` properties causes shared state that leaks between requests.")
            .with_help("Use a local variable, a DTO, or a request-scoped service instead.");

        ctx.collector.report(issue);
    }
}
