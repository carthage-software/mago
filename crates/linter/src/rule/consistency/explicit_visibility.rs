use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Modifier;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Property;
use mago_syntax::cst::Sequence;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ExplicitVisibilityRule {
    meta: &'static RuleMeta,
    cfg: ExplicitVisibilityConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct ExplicitVisibilityConfig {
    pub level: Level,
}

impl Default for ExplicitVisibilityConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for ExplicitVisibilityConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExplicitVisibilityRule {
    type Config = ExplicitVisibilityConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Explicit Visibility",
            code: "explicit-visibility",
            description: indoc! {"
                Requires an explicit visibility (`public`, `protected` or `private`) on every class-like
                method, property (including promoted constructor properties) and constant.

                Members without a visibility are implicitly `public`; spelling it out makes the intent
                clear. The `var` keyword is also reported, since it is a legacy alias for `public`.
            "},
            good_example: indoc! {r"
                <?php

                class Foo
                {
                    public const BAR = 1;

                    public static int $count = 0;

                    public function __construct(
                        public readonly string $name,
                    ) {}

                    public function baz(): void {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                class Foo
                {
                    const BAR = 1;

                    static int $count = 0;

                    var $legacy;

                    function __construct(
                        readonly string $name,
                    ) {}

                    function baz(): void {}
                }
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Method, NodeKind::Property, NodeKind::ClassLikeConstant, NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        match node {
            Node::Method(method) => {
                if let Some(start) = missing_visibility_start(&method.modifiers, method.function.span()) {
                    self.report(ctx, "Method", method.name.span(), Fix::Insert(start));
                }
            }
            Node::ClassLikeConstant(constant) => {
                if let Some(start) = missing_visibility_start(&constant.modifiers, constant.r#const.span()) {
                    self.report(ctx, "Constant", constant.span(), Fix::Insert(start));
                }
            }
            Node::Property(property) => {
                let modifiers = property.modifiers();
                if modifiers.get_first_read_visibility().is_some() {
                    return;
                }

                let span = property.first_variable().span();
                if let Some(var) = property.var() {
                    self.report(ctx, "Property", span, Fix::ReplaceVar(var.span()));
                    return;
                }

                let fallback = match property {
                    Property::Plain(plain) => plain.hint.as_ref().map_or(span, HasSpan::span),
                    Property::Hooked(hooked) => hooked.hint.as_ref().map_or(span, HasSpan::span),
                };

                if let Some(start) = missing_visibility_start(modifiers, fallback) {
                    self.report(ctx, "Property", span, Fix::Insert(start));
                }
            }
            Node::FunctionLikeParameter(parameter) if parameter.is_promoted_property() => {
                let span = parameter.variable.span();
                let fallback = parameter
                    .hint
                    .as_ref()
                    .map(HasSpan::span)
                    .or(parameter.ampersand)
                    .or(parameter.ellipsis)
                    .unwrap_or(span);

                if let Some(start) = missing_visibility_start(&parameter.modifiers, fallback) {
                    self.report(ctx, "Promoted property", span, Fix::Insert(start));
                }
            }
            _ => {}
        }
    }
}

enum Fix {
    /// Insert `public ` at this offset.
    Insert(u32),
    /// Replace the legacy `var` keyword with `public`.
    ReplaceVar(Span),
}

impl ExplicitVisibilityRule {
    fn report<A>(&self, ctx: &mut LintContext<'_, '_, A>, kind: &str, span: Span, fix: Fix)
    where
        A: Arena,
    {
        let issue = Issue::new(self.cfg.level(), format!("{kind} is missing an explicit visibility."))
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(format!("{kind} is implicitly `public`")))
            .with_help("Add an explicit visibility such as `public`, `protected` or `private`.");

        ctx.collector.propose(issue, |edits| match fix {
            Fix::Insert(offset) => edits.push(TextEdit::insert(offset, "public ")),
            Fix::ReplaceVar(var) => edits.push(TextEdit::replace(var, "public")),
        });
    }
}

/// Returns the offset to insert `public ` at when `modifiers` has no read visibility, or `None` if it has one.
///
/// The insertion goes before the first modifier, or before `fallback` (the start of the declaration after
/// attributes) when there are no modifiers.
fn missing_visibility_start(modifiers: &Sequence<'_, Modifier<'_>>, fallback: Span) -> Option<u32> {
    if modifiers.get_first_read_visibility().is_some() {
        return None;
    }

    Some(modifiers.first().map_or(fallback, HasSpan::span).start_offset())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::ExplicitVisibilityRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = all_members_have_visibility,
        rule = ExplicitVisibilityRule,
        code = indoc! {r#"
            <?php

            interface I {
                public const A = 1;
                public function a(): void;
            }

            abstract class C {
                final protected const B = 2;
                private static ?int $x = null;
                public private(set) string $y = '';
                public string $z { get => 'z'; }

                public function __construct(private readonly int $a, int $notPromoted) {}
                abstract protected function b(): void;
            }

            enum E {
                private const C = 3;
                public function c(): void {}
            }

            function f(int $notPromoted): void {}
            $closure = function (int $x) {};
        "#}
    }

    test_lint_failure! {
        name = members_without_visibility,
        rule = ExplicitVisibilityRule,
        count = 10,
        code = indoc! {r#"
            <?php

            interface I {
                const A = 1;
                function a(): void;
            }

            abstract class C {
                final const B = 2;
                static ?int $x = null;
                var $legacy;
                private(set) string $y = '';

                function __construct(readonly int $a) {}
                abstract function b(): void;
            }

            trait T {
                function t(): void {}
            }
        "#}
    }

    test_lint_fix! {
        name = adds_public_to_members,
        rule = ExplicitVisibilityRule,
        code = indoc! {r#"
            <?php

            class C {
                const A = 1;
                final const B = 2;
                #[Attr]
                static ?int $x = null;
                var $legacy;
                int $typed;
                private(set) string $y = '';

                function __construct(readonly int $a, #[Attr] private(set) int $b) {}
                abstract static function b(): void;
            }
        "#},
        fixed = indoc! {r#"
            <?php

            class C {
                public const A = 1;
                public final const B = 2;
                #[Attr]
                public static ?int $x = null;
                public $legacy;
                public int $typed;
                public private(set) string $y = '';

                public function __construct(public readonly int $a, #[Attr] public private(set) int $b) {}
                public abstract static function b(): void;
            }
        "#}
    }
}
