use std::cell::Cell;

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
    in_reset_class: Cell<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoServiceStateMutationConfig {
    pub level: Level,
    pub include_namespaces: Vec<String>,
    pub exclude_namespaces: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub reset_interfaces: Vec<String>,
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
            reset_interfaces: vec!["Symfony\\Contracts\\Service\\ResetInterface".to_string()],
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

/// Returns `Some(span)` if the expression is `self::$prop` or `static::$prop`,
/// possibly through nested array access or array append operations.
fn is_static_property_mutation<'arena>(expr: &Expression<'arena>) -> Option<Span> {
    match expr {
        Expression::Access(Access::StaticProperty(prop)) => match prop.class {
            Expression::Self_(_) | Expression::Static(_) => Some(expr.span()),
            _ => None,
        },
        Expression::ArrayAccess(access) => is_static_property_mutation(access.array),
        Expression::ArrayAppend(append) => is_static_property_mutation(append.array),
        _ => None,
    }
}

/// Checks both `$this->prop` and `self::$prop` / `static::$prop` mutations.
fn is_property_mutation<'arena>(expr: &Expression<'arena>) -> Option<Span> {
    is_this_property_mutation(expr).or_else(|| is_static_property_mutation(expr))
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
            NodeKind::Class,
            NodeKind::Assignment,
            NodeKind::UnaryPrefix,
            NodeKind::UnaryPostfix,
            NodeKind::Unset,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone(), in_reset_class: Cell::new(false) }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        // When entering a class, check if it implements a reset interface.
        if let Node::Class(class) = node {
            let is_reset_class = class.implements.as_ref().is_some_and(|implements| {
                implements.types.iter().any(|iface| {
                    let name = ctx.lookup_name(iface);
                    self.cfg.reset_interfaces.iter().any(|ri| name.ends_with(ri.as_str()))
                })
            });

            self.in_reset_class.set(is_reset_class);
            return;
        }

        // Skip mutations in classes implementing a reset interface.
        if self.in_reset_class.get() {
            return;
        }

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
        // Patterns may end with a backslash (e.g. `App\Entity\`). A namespace like `App\Entity`
        // should match `App\Entity\` because it is that namespace itself, not just a sub-namespace.
        // We compare against both the raw namespace and namespace + `\` to cover both cases.
        let namespace = ctx.scope.get_namespace();
        if !namespace.is_empty() {
            let namespace_with_sep = format!("{namespace}\\");
            let matches_pattern =
                |ns: &str, pattern: &str| ns.starts_with(pattern) || namespace_with_sep.starts_with(pattern);

            let in_include = self.cfg.include_namespaces.iter().any(|p| matches_pattern(namespace, p.as_str()));
            if !in_include {
                return;
            }

            let in_exclude = self.cfg.exclude_namespaces.iter().any(|p| matches_pattern(namespace, p.as_str()));
            if in_exclude {
                return;
            }
        } else {
            // No namespace means not in any included namespace.
            return;
        }

        // Extract the mutated expression and check if it involves `$this->property`.
        // Unset is handled separately because it can have multiple targets.
        if let Node::Unset(unset) = node {
            for value in unset.values.iter() {
                if let Some(span) = is_property_mutation(value) {
                    let issue = Issue::new(self.cfg.level, "Service state mutation detected.")
                        .with_code(self.meta.code)
                        .with_annotation(Annotation::primary(span).with_message("Service property is mutated here"))
                        .with_note(
                            "In worker-mode runtimes (FrankenPHP, RoadRunner, Swoole), services persist across requests.",
                        )
                        .with_note("Mutating instance or static properties causes shared state that leaks between requests.")
                        .with_help("Use a local variable, a DTO, or a request-scoped service instead.");

                    ctx.collector.report(issue);
                }
            }
            return;
        }

        let mutation_span = match node {
            Node::Assignment(assignment) => is_property_mutation(assignment.lhs),
            Node::UnaryPrefix(prefix) => {
                if prefix.operator.is_increment_or_decrement() {
                    is_property_mutation(prefix.operand)
                } else {
                    None
                }
            }
            Node::UnaryPostfix(postfix) => is_property_mutation(postfix.operand),
            _ => None,
        };

        let Some(span) = mutation_span else {
            return;
        };

        let issue = Issue::new(self.cfg.level, "Service state mutation detected.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message("Service property is mutated here"))
            .with_note("In worker-mode runtimes (FrankenPHP, RoadRunner, Swoole), services persist across requests.")
            .with_note("Mutating instance or static properties causes shared state that leaks between requests.")
            .with_help("Use a local variable, a DTO, or a request-scoped service instead.");

        ctx.collector.report(issue);
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::integration::Integration;
    use crate::rule::safety::no_service_state_mutation::NoServiceStateMutationRule;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    fn symfony_settings(s: &mut Settings) {
        s.integrations.insert(Integration::Symfony);
    }

    // ── Success cases ──────────────────────────────────────────────────

    test_lint_success! {
        name = assignment_in_constructor_is_allowed,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function __construct()
                {
                    $this->items = [];
                }
            }
        "#},
    }

    test_lint_success! {
        name = assignment_in_reset_method_is_allowed,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function reset(): void
                {
                    $this->items = [];
                }
            }
        "#},
    }

    test_lint_success! {
        name = excluded_namespace_entity,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Entity;

            class Order
            {
                public function setTotal(int $total): void
                {
                    $this->total = $total;
                }
            }
        "#},
    }

    test_lint_success! {
        name = excluded_namespace_dto,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\DTO;

            class OrderData
            {
                public function setTotal(int $total): void
                {
                    $this->total = $total;
                }
            }
        "#},
    }

    test_lint_success! {
        name = outside_included_namespace,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace Vendor\Library;

            class SomeClass
            {
                public function doSomething(): void
                {
                    $this->count = 0;
                }
            }
        "#},
    }

    test_lint_success! {
        name = local_variable_assignment_is_fine,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function process(): void
                {
                    $count = 0;
                }
            }
        "#},
    }

    test_lint_success! {
        name = reading_property_is_fine,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function findAll(): array
                {
                    return $this->repository->findAll();
                }
            }
        "#},
    }

    test_lint_success! {
        name = no_namespace_is_skipped,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            class OrderService
            {
                public function process(): void
                {
                    $this->count = 0;
                }
            }
        "#},
    }

    // ── Failure cases ──────────────────────────────────────────────────

    test_lint_failure! {
        name = direct_assignment_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function process(int $orderId): void
                {
                    $this->lastOrderId = $orderId;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = compound_assignment_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class LogService
            {
                public function append(string $message): void
                {
                    $this->log .= $message;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = post_increment_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CounterService
            {
                public function increment(): void
                {
                    $this->count++;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = pre_decrement_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CounterService
            {
                public function decrement(): void
                {
                    --$this->count;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = array_push_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CollectorService
            {
                public function collect(mixed $item): void
                {
                    $this->items[] = $item;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = unset_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CacheService
            {
                public function remove(string $key): void
                {
                    unset($this->data[$key]);
                }
            }
        "#},
    }

    test_lint_failure! {
        name = multiple_mutations_in_method,
        rule = NoServiceStateMutationRule,
        count = 2,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class OrderService
            {
                public function process(int $orderId): void
                {
                    $this->lastOrderId = $orderId;
                    $this->count++;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = array_key_assignment_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class RegistryService
            {
                public function register(string $key, mixed $value): void
                {
                    $this->items[$key] = $value;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = addition_assignment_in_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class TotalService
            {
                public function add(float $amount): void
                {
                    $this->total += $amount;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = multiple_unset_targets,
        rule = NoServiceStateMutationRule,
        count = 2,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CacheService
            {
                public function clear(): void
                {
                    unset($this->a, $this->b);
                }
            }
        "#},
    }

    test_lint_failure! {
        name = static_property_post_increment_self,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CounterService
            {
                private static int $counter = 0;

                public static function increment(): void
                {
                    self::$counter++;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = static_property_assignment_via_static,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CounterService
            {
                private static int $counter = 0;

                public static function setCounter(int $value): void
                {
                    static::$counter = $value;
                }
            }
        "#},
    }

    test_lint_success! {
        name = class_implementing_reset_interface_is_skipped,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            use Symfony\Contracts\Service\ResetInterface;

            class CacheService implements ResetInterface
            {
                public function warmUp(): void
                {
                    $this->data = [];
                }

                public function reset(): void
                {
                    $this->data = null;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = static_property_compound_assignment,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class LogService
            {
                private static string $log = '';

                public static function append(string $msg): void
                {
                    self::$log .= $msg;
                }
            }
        "#},
    }

    // ── Trait tests ───────────────────────────────────────────────────

    test_lint_failure! {
        name = mutation_in_trait_method,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            trait CounterTrait
            {
                public function increment(): void
                {
                    $this->count++;
                }
            }
        "#},
    }

    test_lint_success! {
        name = trait_constructor_is_allowed,
        rule = NoServiceStateMutationRule,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            trait Initializable
            {
                public function __construct()
                {
                    $this->items = [];
                }
            }
        "#},
    }

    // ── Edge-case tests ───────────────────────────────────────────────

    test_lint_failure! {
        name = mutation_in_return_expression,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CounterService
            {
                private int $counter = 0;

                public function getAndIncrement(): int
                {
                    return ++$this->counter;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = mutation_via_null_coalescing_assignment,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class CacheService
            {
                private ?int $value = null;

                public function getOrSet(int $default): int
                {
                    return $this->value ??= $default;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = mutation_in_nested_array_key,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class RegistryService
            {
                private array $data = [];

                public function set(string $group, string $key, mixed $value): void
                {
                    $this->data[$group][$key] = $value;
                }
            }
        "#},
    }

    test_lint_failure! {
        name = static_mutation_in_return,
        rule = NoServiceStateMutationRule,
        count = 1,
        settings = symfony_settings,
        code = indoc! {r#"
            <?php

            namespace App\Service;

            class IdService
            {
                private static int $nextId = 0;

                public static function next(): int
                {
                    return ++self::$nextId;
                }
            }
        "#},
    }
}
