use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::FunctionLikeReturnTypeHint;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::scope::ClassLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferSelfReturnTypeRule {
    meta: &'static RuleMeta,
    cfg: PreferSelfReturnTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferSelfReturnTypeConfig {
    pub level: Level,
}

impl Default for PreferSelfReturnTypeConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferSelfReturnTypeConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferSelfReturnTypeRule {
    type Config = PreferSelfReturnTypeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Self Return Type",
            code: "prefer-self-return-type",
            description: indoc! {"
                Suggests using `self` when a method's return type refers to its own enclosing
                class by name.

                Using `self` decouples the signature from the class name, so renaming the class
                doesn't require updating return types. It also communicates intent more clearly:
                'this returns an instance of the same class'.

                Note: this rule does not apply to traits, because `self` inside a trait resolves
                to the using class, not the trait itself. If you want to return a subclass in
                inheritance-aware factory patterns, use `static` instead of `self`.
            "},
            good_example: indoc! {r"
                <?php

                final class Box
                {
                    public static function create(): self
                    {
                        return new self();
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                final class Box
                {
                    public static function create(): Box
                    {
                        return new Box();
                    }
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::Function, NodeKind::Method, NodeKind::Closure, NodeKind::ArrowFunction];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let return_type_hint = match node {
            Node::Function(function) => function.return_type_hint.as_ref(),
            Node::Method(method) => method.return_type_hint.as_ref(),
            Node::Closure(closure) => closure.return_type_hint.as_ref(),
            Node::ArrowFunction(arrow_function) => arrow_function.return_type_hint.as_ref(),
            _ => return,
        };

        let Some(FunctionLikeReturnTypeHint { hint, .. }) = return_type_hint else {
            return;
        };

        let class_fqn = match ctx.scope.get_class_like_scope() {
            Some(ClassLikeScope::Class(name))
            | Some(ClassLikeScope::Interface(name))
            | Some(ClassLikeScope::Enum(name)) => name,
            Some(ClassLikeScope::Trait(_)) | Some(ClassLikeScope::AnonymousClass(_)) | None => return,
        };

        self.check_hint(ctx, class_fqn, hint);
    }
}

impl PreferSelfReturnTypeRule {
    fn check_hint<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, class_fqn: &str, hint: &Hint<'arena>) {
        match hint {
            Hint::Identifier(identifier) => {
                let resolved = ctx.lookup_name(identifier);
                if resolved.eq_ignore_ascii_case(class_fqn) {
                    let used = identifier.value();
                    let issue = Issue::new(
                        self.cfg.level(),
                        format!("Return type `{used}` refers to the enclosing class; use `self` instead."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(identifier.span())
                            .with_message(format!("Replace `{used}` with `self`")),
                    )
                    .with_note(
                        "Using `self` decouples the signature from the class name, so renaming the class does not require updating return types.",
                    )
                    .with_help(
                        "Replace the explicit class name with `self`, or with `static` if the method should return instances of subclasses.",
                    );

                    ctx.collector.propose(issue, |edits| {
                        edits.push(TextEdit::replace(identifier.span(), "self"));
                    });
                }
            }
            Hint::Parenthesized(parenthesized) => self.check_hint(ctx, class_fqn, parenthesized.hint),
            Hint::Nullable(nullable) => self.check_hint(ctx, class_fqn, nullable.hint),
            Hint::Union(union) => {
                self.check_hint(ctx, class_fqn, union.left);
                self.check_hint(ctx, class_fqn, union.right);
            }
            Hint::Intersection(intersection) => {
                self.check_hint(ctx, class_fqn, intersection.left);
                self.check_hint(ctx, class_fqn, intersection.right);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferSelfReturnTypeRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = self_return_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public static function create(): self
                {
                    return new self();
                }
            }
        "}
    }

    test_lint_success! {
        name = static_return_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            class Box
            {
                public static function create(): static
                {
                    return new static();
                }
            }
        "}
    }

    test_lint_success! {
        name = unrelated_class_return_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            class Box
            {
                public function transform(): Other
                {
                    return new Other();
                }
            }

            class Other
            {
            }
        "}
    }

    test_lint_success! {
        name = parent_class_return_in_child_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            class Parent_
            {
            }

            class Child extends Parent_
            {
                public function factory(): Parent_
                {
                    return new Parent_();
                }
            }
        "}
    }

    test_lint_success! {
        name = trait_self_like_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            trait BoxTrait
            {
                public function clone_(): BoxTrait
                {
                    return $this;
                }
            }
        "}
    }

    test_lint_success! {
        name = free_function_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            class Box
            {
            }

            function make_box(): Box
            {
                return new Box();
            }
        "}
    }

    test_lint_success! {
        name = anonymous_class_is_ok,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            class Container
            {
            }

            $c = new class {
                public function grab(): Container
                {
                    return new Container();
                }
            };
        "}
    }

    test_lint_failure! {
        name = class_name_return_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public static function create(): Box
                {
                    return new Box();
                }
            }
        "}
    }

    test_lint_failure! {
        name = nullable_class_name_return_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public static function find(): ?Box
                {
                    return null;
                }
            }
        "}
    }

    test_lint_failure! {
        name = union_containing_class_name_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public function sibling(): Box|null
                {
                    return null;
                }
            }
        "}
    }

    test_lint_failure! {
        name = intersection_containing_class_name_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box implements \Countable
            {
                public function refine(): Box&\Countable
                {
                    return $this;
                }

                public function count(): int
                {
                    return 0;
                }
            }
        "}
    }

    test_lint_failure! {
        name = namespaced_class_name_return_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            namespace Foo;

            final class Box
            {
                public function other(): \Foo\Box
                {
                    return $this;
                }
            }
        "}
    }

    test_lint_failure! {
        name = interface_self_reference_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            interface Cloneable
            {
                public function duplicate(): Cloneable;
            }
        "}
    }

    test_lint_failure! {
        name = method_on_enum_self_reference_is_flagged,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            enum Color
            {
                case Red;
                case Green;

                public function with(): Color
                {
                    return $this;
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_simple_class_name_return,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public static function create(): Box
                {
                    return new Box();
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            final class Box
            {
                public static function create(): self
                {
                    return new Box();
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_nullable_class_name_return,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public static function find(): ?Box
                {
                    return null;
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            final class Box
            {
                public static function find(): ?self
                {
                    return null;
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_union_class_name_return,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            final class Box
            {
                public function sibling(): Box|null
                {
                    return null;
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            final class Box
            {
                public function sibling(): self|null
                {
                    return null;
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_namespaced_class_name_return,
        rule = PreferSelfReturnTypeRule,
        code = indoc! {r"
            <?php

            namespace Foo;

            final class Box
            {
                public function other(): \Foo\Box
                {
                    return $this;
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            namespace Foo;

            final class Box
            {
                public function other(): self
                {
                    return $this;
                }
            }
        "}
    }
}
