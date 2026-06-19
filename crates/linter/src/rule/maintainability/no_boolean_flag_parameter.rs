use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Assignment;
use mago_syntax::cst::Block;
use mago_syntax::cst::DirectVariable;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Hint;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Variable;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::variable_usage::function_like_parts;
use crate::rule_meta::RuleMeta;
use crate::scope::FunctionLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoBooleanFlagParameterRule {
    meta: &'static RuleMeta,
    cfg: NoBooleanFlagParameterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoBooleanFlagParameterConfig {
    pub level: Level,
    /// When enabled, methods whose name starts with `set` are exempt from this rule.
    pub exclude_setters: bool,
    /// When enabled, constructors are exempt from this rule.
    pub exclude_constructors: bool,
}

impl Default for NoBooleanFlagParameterConfig {
    fn default() -> Self {
        Self { level: Level::Help, exclude_setters: false, exclude_constructors: true }
    }
}

impl Config for NoBooleanFlagParameterConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoBooleanFlagParameterRule {
    type Config = NoBooleanFlagParameterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Boolean Flag Parameter",
            code: "no-boolean-flag-parameter",
            description: indoc! {r"
                Flags function-like parameters that use a boolean type and drive the function's behaviour.

                A boolean parameter is only reported when it is used as a flag — that is, referenced
                somewhere other than as the value stored by a plain assignment. Such flag parameters can
                indicate a violation of the Single Responsibility Principle (SRP); refactor by extracting
                the flag logic into its own class or method.

                A boolean parameter that is merely stored (e.g. `$this->enabled = $enabled;`) does not
                branch the function's behaviour and is not reported.
            "},
            good_example: indoc! {r"
                <?php

                function get_difference(string $a, string $b): string {
                    // ...
                }

                function get_difference_case_insensitive(string $a, string $b): string {
                    // ...
                }

                final class Connection
                {
                    private bool $secure;

                    // The boolean is only stored, never used to branch behaviour.
                    public function configure(bool $secure): void
                    {
                        $this->secure = $secure;
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                function get_difference(string $a, string $b, bool $ignore_case): string {
                    if ($ignore_case) {
                        return strtolower($a) === strtolower($b) ? '' : $a;
                    }

                    return $a === $b ? '' : $a;
                }
            "},
            category: Category::Maintainability,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::FunctionLikeParameter(parameter) = node else {
            return;
        };

        if parameter.is_promoted_property() {
            return;
        }

        let Some(Hint::Bool(bool_hint)) = &parameter.hint else {
            return;
        };

        if let Some(FunctionLikeScope::Method(name, _)) = ctx.scope.get_function_like_scope() {
            if self.cfg.exclude_constructors && name.eq_ignore_ascii_case(b"__construct") {
                return;
            }

            if self.cfg.exclude_setters && name.len() > 3 && name[..3].eq_ignore_ascii_case(b"set") {
                return;
            }
        }

        // Only report the parameter when it is actually used as a flag — that is,
        // referenced somewhere other than as the value stored by a plain assignment.
        // A boolean parameter that is merely assigned to a property or variable
        // (e.g. `$this->enabled = $enabled;`) does not drive any branching, so
        // reporting it would be a false positive. When there is no concrete body
        // to inspect (abstract methods, interfaces, arrow functions), the
        // parameter is reported as before.
        if let Some(body) = enclosing_function_like_body(ctx)
            && !is_used_as_flag(parameter.variable.name, body)
        {
            return;
        }

        let issue = Issue::new(self.cfg.level, "Avoid boolean flag parameters.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(parameter.variable.span())
                    .with_message("This parameter acts as a boolean flag"),
            )
            .with_annotation(Annotation::secondary(bool_hint.span).with_message("Boolean type declared here"))
            .with_note(
                "Boolean flags often indicate a function has more than one responsibility, making it harder to understand and test.",
            )
            .with_help(
                "Refactor by splitting the function into two separate methods, each with a clear, descriptive name.",
            );

        ctx.collector.report(issue);
    }
}

/// Returns the body of the nearest enclosing function-like (function, method, or
/// closure) for the node currently being linted, when it has a concrete block
/// body. Arrow functions and bodiless declarations (abstract methods,
/// interfaces) yield `None`.
fn enclosing_function_like_body<'ctx, 'arena, A>(
    ctx: &LintContext<'ctx, 'arena, A>,
) -> Option<&'ctx Block<'arena>>
where
    A: Arena,
{
    let mut depth = 0;
    while let Some(node) = ctx.get_nth_parent(depth) {
        if matches!(node, Node::Function(_) | Node::Method(_) | Node::Closure(_) | Node::ArrowFunction(_)) {
            return function_like_parts(node).map(|parts| parts.body);
        }

        depth += 1;
    }

    None
}

/// Determines whether `name` is used as a flag within `body` — i.e. referenced
/// anywhere other than as the value stored by a plain (`=`) assignment.
fn is_used_as_flag(name: &[u8], body: &Block<'_>) -> bool {
    let mut walker = FlagParameterWalker { target: name, used_as_flag: false };
    walker.walk_block(body, &mut ());
    walker.used_as_flag
}

/// Walks a function body looking for any reference to a parameter that is not a
/// plain "store" (the right-hand side of a simple assignment). Such a reference
/// means the parameter influences behaviour and is therefore a flag.
struct FlagParameterWalker<'target> {
    target: &'target [u8],
    used_as_flag: bool,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for FlagParameterWalker<'_> {
    fn walk_in_direct_variable(&mut self, variable: &'ast DirectVariable<'arena>, _: &mut ()) {
        if variable.name == self.target {
            self.used_as_flag = true;
        }
    }

    fn walk_assignment(&mut self, assignment: &'ast Assignment<'arena>, context: &mut ()) {
        if self.used_as_flag {
            return;
        }

        // `$x = $param;` stores the parameter without using it as a flag, so the
        // right-hand side is not counted. Every other position is a real use.
        let is_plain_store = assignment.operator.is_assign()
            && matches!(
                assignment.rhs,
                Expression::Variable(Variable::Direct(variable)) if variable.name == self.target
            );

        if !is_plain_store {
            self.walk_expression(assignment.rhs, context);
        }

        self.walk_expression(assignment.lhs, context);
    }
}
