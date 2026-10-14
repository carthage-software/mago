use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferSelfRule {
    meta: &'static RuleMeta,
    cfg: PreferSelfConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferSelfConfig {
    pub level: Level,
}

impl Default for PreferSelfConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferSelfConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferSelfRule {
    type Config = PreferSelfConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Self",
            code: "prefer-self",
            description: indoc! {"
                Suggests using `self` when a class refers to itself by name.

                Using `self` keeps the code working when the class is renamed, and makes it
                obvious that the reference points at the enclosing class.

                A static call through the class name does not forward late static binding, while
                a call through `self` does. In a class that can be extended, replacing the name in
                a static method call or first-class callable can therefore change what `static`
                resolves to in the called method, so those fixes are marked as potentially unsafe.
                The same applies inside closures, which can be rebound to another class scope.

                References inside traits, anonymous classes, and attribute arguments are ignored.
            "},
            good_example: indoc! {r"
                <?php

                final class Box
                {
                    private const DEFAULT_SIZE = 10;

                    public function __construct(
                        public int $size,
                    ) {}

                    public static function create(): self
                    {
                        return new self(self::DEFAULT_SIZE);
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                final class Box
                {
                    private const DEFAULT_SIZE = 10;

                    public function __construct(
                        public int $size,
                    ) {}

                    public static function create(): self
                    {
                        return new Box(Box::DEFAULT_SIZE);
                    }
                }
            "},
            category: Category::BestPractices,
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
        let (class, is_static_call) = match node {
            Node::Instantiation(instantiation) => (instantiation.class, false),
            Node::StaticMethodCall(call) => (call.class, true),
            Node::StaticMethodPartialApplication(partial) => (partial.class, true),
            Node::StaticPropertyAccess(access) => (access.class, false),
            Node::ClassConstantAccess(access) => (access.class, false),
            _ => return,
        };

        let Expression::Identifier(identifier) = class else {
            return;
        };

        let Some(enclosing) = find_enclosing_class_like(ctx) else {
            return;
        };

        if !ctx.lookup_name(identifier).eq_ignore_ascii_case(enclosing.name) {
            return;
        }

        let forwards_late_static_binding = is_static_call && !enclosing.is_final;
        let used = mago_bytes::BytesDisplay(identifier.value());
        let mut issue =
            Issue::new(self.cfg.level(), format!("`{used}` refers to the enclosing class; use `self` instead."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span()).with_message(format!("Replace `{used}` with `self`")),
                )
                .with_note("Using `self` keeps the code working when the class is renamed.");

        if forwards_late_static_binding {
            issue = issue.with_note(
                "Unlike a call through the class name, a call through `self` forwards late static binding, so `static` in the called method may resolve to a subclass.",
            );
        }

        if enclosing.in_closure {
            issue =
                issue.with_note("Closures can be rebound to another class scope, which changes what `self` refers to.");
        }

        let safety =
            if forwards_late_static_binding || enclosing.in_closure { Safety::PotentiallyUnsafe } else { Safety::Safe };

        ctx.collector.propose(issue.with_help("Replace the class name with `self`."), |edits| {
            edits.push(TextEdit::replace(identifier.span(), "self").with_safety(safety));
        });
    }
}

struct EnclosingClassLike<'arena> {
    name: &'arena [u8],
    is_final: bool,
    in_closure: bool,
}

fn find_enclosing_class_like<'arena, A>(ctx: &LintContext<'_, 'arena, A>) -> Option<EnclosingClassLike<'arena>>
where
    A: Arena,
{
    let mut in_closure = false;
    let mut depth = 0;
    loop {
        match ctx.get_nth_parent(depth)? {
            Node::Class(class) => {
                return Some(EnclosingClassLike {
                    name: ctx.lookup_name(&class.name),
                    is_final: class.modifiers.contains_final(),
                    in_closure,
                });
            }
            Node::Enum(r#enum) => {
                return Some(EnclosingClassLike { name: ctx.lookup_name(&r#enum.name), is_final: true, in_closure });
            }
            Node::Interface(interface) => {
                return Some(EnclosingClassLike {
                    name: ctx.lookup_name(&interface.name),
                    is_final: false,
                    in_closure,
                });
            }
            Node::Closure(_) | Node::ArrowFunction(_) => in_closure = true,
            Node::Trait(_) | Node::AnonymousClass(_) | Node::Function(_) | Node::Attribute(_) => return None,
            _ => {}
        }

        depth += 1;
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use mago_text_edit::Safety;

    use super::PreferSelfRule;
    use crate::rule::tests::collect_issues;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = reports_every_class_name_reference_inside_the_class,
        rule = PreferSelfRule,
        count = 6,
        code = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const VALUE = 1;

                public static function create(): self
                {
                    Example::call();
                    $callable = Example::call(...);
                    Example::$value;
                    Example::VALUE;
                    Example::class;

                    return new Example();
                }

                private static function call(): void {}
            }
        "}
    }

    test_lint_fix! {
        name = replaces_every_class_name_reference_with_self,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const VALUE = 1;

                public static function create(): self
                {
                    Example::call();
                    $callable = Example::call(...);
                    Example::$value;
                    Example::VALUE;
                    Example::class;

                    return new Example();
                }

                private static function call(): void {}
            }
        "},
        fixed = indoc! {r"
            <?php

            final class Example
            {
                private static int $value = 1;
                private const VALUE = 1;

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

    test_lint_fix! {
        name = issue_2528_replaces_class_name_in_static_call,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            declare(strict_types=1);

            namespace Maximal\Test;

            final class FinalSelfStaticTest
            {
                private static function some(): int
                {
                    return 666;
                }

                public static function getSomeBySelf(): int
                {
                    return self::some();
                }

                public static function getSomeByName(): int
                {
                    return FinalSelfStaticTest::some();
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            declare(strict_types=1);

            namespace Maximal\Test;

            final class FinalSelfStaticTest
            {
                private static function some(): int
                {
                    return 666;
                }

                public static function getSomeBySelf(): int
                {
                    return self::some();
                }

                public static function getSomeByName(): int
                {
                    return self::some();
                }
            }
        "}
    }

    test_lint_fix! {
        name = resolves_qualified_and_aliased_names,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            namespace App\Model;

            use App\Model\Box as Alias;

            final class Box
            {
                public const SIZE = 1;

                public function sizes(): array
                {
                    return [Box::SIZE, \App\Model\Box::SIZE, Alias::SIZE, \Other\Box::SIZE];
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            namespace App\Model;

            use App\Model\Box as Alias;

            final class Box
            {
                public const SIZE = 1;

                public function sizes(): array
                {
                    return [self::SIZE, self::SIZE, self::SIZE, \Other\Box::SIZE];
                }
            }
        "}
    }

    test_lint_failure! {
        name = reports_enums_and_interfaces,
        rule = PreferSelfRule,
        count = 3,
        code = indoc! {r"
            <?php

            enum Status: string
            {
                case Ready = 'ready';

                public static function fallback(): self
                {
                    return Status::tryFrom('unknown') ?? Status::Ready;
                }
            }

            interface Limits
            {
                public const MIN = 0;
                public const MAX = Limits::MIN + 10;
            }
        "}
    }

    test_lint_success! {
        name = ignores_other_classes_and_keywords,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            class Other
            {
                public static function make(): self
                {
                    return new self();
                }
            }

            class Example extends Other
            {
                public static function create(): Other
                {
                    self::make();
                    static::make();
                    parent::make();

                    return new Other();
                }
            }

            Example::create();
        "}
    }

    test_lint_success! {
        name = ignores_traits_anonymous_classes_and_nested_functions,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            trait Greets
            {
                public function name(): string
                {
                    return Greets::class;
                }
            }

            final class Outer
            {
                public const VALUE = 1;

                public function make(): object
                {
                    return new class {
                        public function value(): int
                        {
                            return Outer::VALUE;
                        }
                    };
                }

                public function register(): void
                {
                    function outer_value(): int
                    {
                        return Outer::VALUE;
                    }
                }
            }
        "}
    }

    test_lint_success! {
        name = ignores_attribute_arguments,
        rule = PreferSelfRule,
        code = indoc! {r"
            <?php

            #[Name(Route::NAME)]
            final class Route
            {
                public const NAME = 'home';

                #[Name(Route::NAME)]
                public function home(): void {}
            }
        "}
    }

    #[test]
    fn marks_late_static_binding_and_closure_fixes_as_potentially_unsafe() {
        let code = indoc! {r"
            <?php

            class Base
            {
                public const NAME = 'base';

                public static function create(): static
                {
                    Base::boot();
                    $boot = Base::boot(...);
                    $name = Base::NAME;
                    $closure = fn (): string => Base::NAME;

                    return new Base();
                }

                public static function boot(): void {}
            }

            final class Leaf
            {
                public static function create(): void
                {
                    Leaf::boot();
                    $closure = static function (): void {
                        Leaf::boot();
                    };
                }

                public static function boot(): void {}
            }
        "};

        let safeties: Vec<_> = collect_issues::<PreferSelfRule, fn(&mut Settings)>(code, None, None)
            .into_iter()
            .flat_map(|issue| issue.edits.into_values().flatten())
            .map(|edit| edit.safety)
            .collect();

        assert_eq!(
            safeties,
            [
                Safety::PotentiallyUnsafe,
                Safety::PotentiallyUnsafe,
                Safety::Safe,
                Safety::PotentiallyUnsafe,
                Safety::Safe,
                Safety::Safe,
                Safety::PotentiallyUnsafe,
            ]
        );
    }

    #[test]
    fn is_disabled_by_default() {
        assert!(!Settings::default().rules.prefer_self.enabled);
    }
}
