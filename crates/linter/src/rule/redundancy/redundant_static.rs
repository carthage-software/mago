use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Hint;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct RedundantStaticRule {
    meta: &'static RuleMeta,
    cfg: RedundantStaticConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct RedundantStaticConfig {
    pub level: Level,
}

impl Default for RedundantStaticConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for RedundantStaticConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for RedundantStaticRule {
    type Config = RedundantStaticConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Redundant Static",
            code: "redundant-static",
            description: indoc! {"
                Detects uses of `static` for late-static binding inside final classes.

                A final class cannot be extended, so `static` and `self` resolve to the same class.
                Using `self` states that intent directly and avoids unnecessary late-static binding.
            "},
            good_example: indoc! {r"
                <?php

                final class User
                {
                    public static function create(): self
                    {
                        return new self();
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                final class User
                {
                    public static function create(): static
                    {
                        return new static();
                    }
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Instantiation,
            NodeKind::StaticMethodCall,
            NodeKind::StaticMethodPartialApplication,
            NodeKind::StaticPropertyAccess,
            NodeKind::ClassConstantAccess,
            NodeKind::Method,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        if !is_inside_final_class(ctx) {
            return;
        }

        if let Node::Method(method) = node {
            if let Some(return_type_hint) = &method.return_type_hint {
                self.check_hint(ctx, &return_type_hint.hint);
            }

            return;
        }

        let static_span = match node {
            Node::Instantiation(instantiation) => get_static_span(instantiation.class),
            Node::StaticMethodCall(call) => get_static_span(call.class),
            Node::StaticMethodPartialApplication(partial) => get_static_span(partial.class),
            Node::StaticPropertyAccess(access) => get_static_span(access.class),
            Node::ClassConstantAccess(access) => get_static_span(access.class),
            _ => None,
        };

        if let Some(static_span) = static_span {
            self.report(ctx, static_span);
        }
    }
}

impl RedundantStaticRule {
    fn check_hint<A>(&self, ctx: &mut LintContext<'_, '_, A>, hint: &Hint<'_>)
    where
        A: Arena,
    {
        match hint {
            Hint::Static(keyword) => self.report(ctx, keyword.span()),
            Hint::Parenthesized(parenthesized) => self.check_hint(ctx, parenthesized.hint),
            Hint::Nullable(nullable) => self.check_hint(ctx, nullable.hint),
            Hint::Union(union) => {
                self.check_hint(ctx, union.left);
                self.check_hint(ctx, union.right);
            }
            Hint::Intersection(intersection) => {
                self.check_hint(ctx, intersection.left);
                self.check_hint(ctx, intersection.right);
            }
            _ => {}
        }
    }

    fn report<A>(&self, ctx: &mut LintContext<'_, '_, A>, static_span: Span)
    where
        A: Arena,
    {
        let issue =
            Issue::new(self.cfg.level(), "The use of `static` is redundant because the enclosing class is final.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(static_span).with_message("This `static` can be replaced with `self`"),
                )
                .with_note("Final classes cannot be extended, so late-static binding has no effect here.")
                .with_help("Replace `static` with `self`.");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(static_span, "self"));
        });
    }
}

fn get_static_span(expression: &Expression<'_>) -> Option<Span> {
    match expression {
        Expression::Static(keyword) => Some(keyword.span()),
        _ => None,
    }
}

fn is_inside_final_class<A>(ctx: &LintContext<'_, '_, A>) -> bool
where
    A: Arena,
{
    let mut depth = 0;
    loop {
        match ctx.get_nth_parent(depth) {
            Some(Node::Class(class)) => return class.modifiers.contains_final(),
            Some(Node::Interface(_) | Node::Trait(_) | Node::Enum(_) | Node::AnonymousClass(_)) | None => return false,
            Some(_) => depth += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::RedundantStaticRule;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = reports_all_static_references_in_final_class,
        rule = RedundantStaticRule,
        count = 7,
        code = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const int VALUE = 1;

                public static function create(): static
                {
                    static::call();
                    $callable = static::call(...);
                    static::$value;
                    static::VALUE;
                    static::class;

                    return new static();
                }

                private static function call(): void {}
            }
        "}
    }

    test_lint_fix! {
        name = replaces_all_static_references_with_self,
        rule = RedundantStaticRule,
        code = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const int VALUE = 1;

                public static function create(): static
                {
                    static::call();
                    $callable = static::call(...);
                    static::$value;
                    static::VALUE;
                    static::class;

                    return new static();
                }

                private static function call(): void {}
            }
        "},
        fixed = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const int VALUE = 1;

                public static function create(): self
                {
                    self::call();
                    $callable = self::call(...);
                    self::$value;
                    self::VALUE;
                    self::class;

                    return new self();
                }

                private static function call(): void {}
            }
        "}
    }

    test_lint_success! {
        name = allows_static_references_in_extensible_class,
        rule = RedundantStaticRule,
        code = indoc! {r"
            <?php

            class Example
            {
                public static function create(): static
                {
                    static::boot();

                    return new static();
                }
            }
        "}
    }

    test_lint_success! {
        name = ignores_static_modifiers_and_local_variables,
        rule = RedundantStaticRule,
        code = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;

                public static function run(): void
                {
                    static $calls = 0;
                    $closure = static function (): void {};
                }
            }
        "}
    }

    test_lint_success! {
        name = stops_at_nested_class_like_scope,
        rule = RedundantStaticRule,
        code = indoc! {r"
            <?php

            final class Outer
            {
                public function make(): object
                {
                    return new class {
                        public function copy(): static
                        {
                            return new static();
                        }
                    };
                }
            }
        "}
    }

    test_lint_success! {
        name = ignores_enums_traits_and_interfaces,
        rule = RedundantStaticRule,
        code = indoc! {r"
            <?php

            interface Factory
            {
                public static function create(): static;
            }

            trait Creates
            {
                public static function create(): static
                {
                    return new static();
                }
            }

            enum Status
            {
                case Ready;

                public static function current(): static
                {
                    return static::Ready;
                }
            }
        "}
    }

    #[test]
    fn is_disabled_by_default() {
        assert!(!Settings::default().rules.redundant_static.enabled);
    }
}
