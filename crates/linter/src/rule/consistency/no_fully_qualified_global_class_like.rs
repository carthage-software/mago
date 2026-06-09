use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
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
pub struct NoFullyQualifiedGlobalClassLikeRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalClassLikeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoFullyQualifiedGlobalClassLikeConfig {
    pub level: Level,
}

impl Default for NoFullyQualifiedGlobalClassLikeConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoFullyQualifiedGlobalClassLikeConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl NoFullyQualifiedGlobalClassLikeRule {
    fn report_if_fq<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, identifier: Identifier<'arena>)
    where
        A: Arena,
    {
        if !identifier.is_fully_qualified() {
            return;
        }

        let class_name_bytes = mago_bytes::trim_start_byte(identifier.value(), b'\\');
        let short_name = class_name_bytes.rsplit(|&b| b == b'\\').next().unwrap_or(class_name_bytes);
        let fqn_span = identifier.span();

        let resolution = ctx.import_name(class_name_bytes);

        let class_name = mago_bytes::BytesDisplay(class_name_bytes);
        let short_name_display = mago_bytes::BytesDisplay(short_name);

        let (title, help) = match &resolution {
            Some(res) if res.is_already_available() && res.local_name.as_bytes() != short_name => (
                "Fully-qualified class-like reference can be replaced with an existing alias.",
                format!("`{class_name}` is already imported as `{}`; replace the reference with it.", res.local_name),
            ),
            Some(res) if res.is_already_available() => (
                "Fully-qualified class-like reference is already in scope.",
                format!("`{class_name}` is already reachable as `{}`; drop the leading `\\`.", res.local_name),
            ),
            Some(_) => (
                "Fully-qualified class-like reference detected.",
                format!("Add `use {class_name};` and reference `{short_name_display}` directly."),
            ),
            None => (
                "Fully-qualified class-like reference detected.",
                format!("Add `use {class_name};` and reference `{short_name_display}` directly."),
            ),
        };

        let issue = Issue::new(self.cfg.level, title)
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(fqn_span)
                    .with_message(format!("The reference to `\\{class_name}` uses a fully-qualified name")),
            )
            .with_note("Fully-qualified class-like references bypass the import system, making it harder to see which classes, interfaces, traits, and enums a file depends on.")
            .with_help(help);

        match resolution {
            Some(resolution) => {
                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(fqn_span, resolution.local_name.as_bytes()));

                    if let Some(use_edit) = resolution.use_statement_edit {
                        edits.push(use_edit.with_safety(Safety::Safe));
                    }
                });
            }
            None => {
                ctx.collector.report(issue);
            }
        }
    }
}

impl LintRule for NoFullyQualifiedGlobalClassLikeRule {
    type Config = NoFullyQualifiedGlobalClassLikeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global Class-Like",
            code: "no-fully-qualified-global-class-like",
            description: indoc! {"
                Disallows fully-qualified class-like references that could be imported instead.

                Instead of using the backslash prefix (e.g., `new \\DateTime()` or `\\Exception`
                in a type hint), prefer an explicit `use` import statement. This improves
                readability and keeps imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use DateTime;
                use Exception;

                $dt = new DateTime();

                function foo(DateTime $dt): Exception {}
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $dt = new \DateTime();

                function foo(\DateTime $dt): \Exception {}
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Attribute,
            NodeKind::Binary,
            NodeKind::ClassConstantAccess,
            NodeKind::Extends,
            NodeKind::Hint,
            NodeKind::Implements,
            NodeKind::Instantiation,
            NodeKind::StaticMethodCall,
            NodeKind::StaticMethodPartialApplication,
            NodeKind::StaticPropertyAccess,
            NodeKind::TraitUse,
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
        match node {
            Node::Attribute(attribute) => {
                self.report_if_fq(ctx, attribute.name);
            }
            Node::Extends(extends) => {
                for identifier in extends.types.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            Node::Implements(implements) => {
                for identifier in implements.types.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            Node::Instantiation(instantiation) => {
                let Expression::Identifier(identifier) = instantiation.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticMethodCall(call) => {
                let Expression::Identifier(identifier) = call.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticMethodPartialApplication(application) => {
                let Expression::Identifier(identifier) = application.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticPropertyAccess(access) => {
                let Expression::Identifier(identifier) = access.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::ClassConstantAccess(access) => {
                let Expression::Identifier(identifier) = access.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::Binary(binary) => {
                if !matches!(binary.operator, BinaryOperator::Instanceof(_)) {
                    return;
                }
                let Expression::Identifier(identifier) = binary.rhs else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::Hint(Hint::Identifier(identifier)) => {
                self.report_if_fq(ctx, *identifier);
            }
            Node::TraitUse(trait_use) => {
                for identifier in trait_use.trait_names.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalClassLikeRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_class_is_not_flagged,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use DateTime;

            $dt = new DateTime();
        "#}
    }

    test_lint_fix! {
        name = global_scope_single_segment_fq_drops_leading_slash,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $dt = new \DateTime();
        "#},
        fixed = indoc! {r#"
            <?php

            $dt = new DateTime();
        "#}
    }

    test_lint_failure! {
        name = fq_instantiation_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $dt = new \DateTime();
        "#}
    }

    test_lint_failure! {
        name = fq_static_method_call_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $dt = \DateTime::createFromFormat('Y-m-d', '2024-01-01');
        "#}
    }

    test_lint_failure! {
        name = fq_class_constant_access_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $format = \DateTime::ATOM;
        "#}
    }

    test_lint_failure! {
        name = fq_type_hint_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function foo(\DateTime $dt): void {}
        "#}
    }

    test_lint_failure! {
        name = fq_static_property_access_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x = \Foo::$bar;
        "#}
    }

    test_lint_failure! {
        name = fq_first_class_callable_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x = \Foo::bar(...);
        "#}
    }

    test_lint_failure! {
        name = fq_instanceof_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x instanceof \Foo;
        "#}
    }

    test_lint_failure! {
        name = fq_extends_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo extends \Bar {}
        "#}
    }

    test_lint_failure! {
        name = fq_implements_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo implements \Bar, \Baz {}
        "#}
    }

    test_lint_failure! {
        name = fq_attribute_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            #[\SomeAttribute]
            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = fq_trait_use_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo {
                use \SomeTrait;
            }
        "#}
    }

    test_lint_fix! {
        name = fix_single_fq_adds_use_after_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $dt = new \DateTime();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use DateTime;

            $dt = new DateTime();
        "#}
    }

    test_lint_fix! {
        name = fix_single_fq_appends_after_existing_use,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use Foo\Bar;

            $dt = new \DateTime();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Foo\Bar;
            use DateTime;

            $dt = new DateTime();
        "#}
    }

    test_lint_fix! {
        name = fix_uses_existing_alias_when_available,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use Foo\Bar as Baz;

            $thing = new \Foo\Bar();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Foo\Bar as Baz;

            $thing = new Baz();
        "#}
    }

    test_lint_fix! {
        name = fix_strips_leading_slash_when_already_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $thing = new \App\Thing();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            $thing = new Thing();
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_existing_import,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use Other\User;

            class Admin extends \Shop\User {}
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_class,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class User {}

            class Admin extends \Shop\User {}
        "#}
    }

    test_lint_fix! {
        name = fix_reuses_anchor_inside_braced_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App {
                $dt = new \DateTime();
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App {

            use DateTime;
                $dt = new DateTime();
            }
        "#}
    }

    test_lint_fix! {
        name = fix_two_distinct_fqns_in_one_pass,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = new \DateTime();
            $b = new \Exception();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use DateTime;

            use Exception;

            $a = new DateTime();
            $b = new Exception();
        "#}
    }

    test_lint_fix! {
        name = fix_three_fqns_in_one_pass,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = \Foo\A::class;
            $b = \Foo\B::class;
            $c = \Foo\C::class;
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Foo\A;

            use Foo\B;

            use Foo\C;

            $a = A::class;
            $b = B::class;
            $c = C::class;
        "#}
    }

    test_lint_fix! {
        name = fix_many_references_to_same_fqn_one_use,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $a = new \Foo\Bar();
            $b = new \Foo\Bar();
            $c = new \Foo\Bar();
            $d = new \Foo\Bar();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Foo\Bar;

            $a = new Bar();
            $b = new Bar();
            $c = new Bar();
            $d = new Bar();
        "#}
    }

    test_lint_fix! {
        name = fix_appends_after_last_of_several_existing_uses,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use Foo\Alpha;
            use Foo\Beta;
            use Foo\Gamma;

            $x = new \DateTime();
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use Foo\Alpha;
            use Foo\Beta;
            use Foo\Gamma;
            use DateTime;

            $x = new DateTime();
        "#}
    }

    test_lint_fix! {
        name = global_scope_single_segment_exception_drops_leading_slash,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            throw new \RuntimeException('boom');
        "#},
        fixed = indoc! {r#"
            <?php

            throw new RuntimeException('boom');
        "#}
    }

    test_lint_fix! {
        name = fix_multiple_braced_namespaces_independent_imports,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace A {
                $x = new \Foo\Bar();
            }

            namespace B {
                $y = new \Foo\Bar();
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace A {

            use Foo\Bar;
                $x = new Bar();
            }

            namespace B {

            use Foo\Bar;
                $y = new Bar();
            }
        "#}
    }

    test_lint_fix! {
        name = fix_fq_trait_use_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo {
                use \SomeTrait;
            }
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            use SomeTrait;

            class Foo {
                use SomeTrait;
            }
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_interface,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            interface User {}

            class Admin extends \Shop\User {}
        "#}
    }

    test_lint_failure! {
        name = fix_declined_when_short_name_conflicts_with_local_trait,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            trait Shop {}

            class Admin {
                use \Other\Shop;
            }
        "#}
    }

    test_lint_fix! {
        name = fix_global_multi_segment_class_constant,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            return [
                'activity_model' => \ExternalPackage\Models\Activity::class,
            ];
        "#},
        fixed = indoc! {r#"
            <?php

            declare(strict_types=1);

            use ExternalPackage\Models\Activity;

            return [
                'activity_model' => Activity::class,
            ];
        "#}
    }

    test_lint_fix! {
        name = fix_global_multi_segment_no_declare,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $x = new \Foo\Bar();
        "#},
        fixed = indoc! {r#"
            <?php

            use Foo\Bar;

            $x = new Bar();
        "#}
    }

    test_lint_fix! {
        name = fix_global_multi_segment_static_call,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $x = \App\Services\Mailer::make();
        "#},
        fixed = indoc! {r#"
            <?php

            use App\Services\Mailer;

            $x = Mailer::make();
        "#}
    }

    test_lint_fix! {
        name = fix_global_multi_segment_extends,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            class Admin extends \Shop\User {}
        "#},
        fixed = indoc! {r#"
            <?php

            use Shop\User;

            class Admin extends User {}
        "#}
    }

    test_lint_fix! {
        name = fix_global_multi_segment_type_hint,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            function handle(\App\Models\User $user): void {}
        "#},
        fixed = indoc! {r#"
            <?php

            use App\Models\User;

            function handle(User $user): void {}
        "#}
    }

    test_lint_failure! {
        name = global_multi_segment_instanceof_is_flagged,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $ok = $value instanceof \App\Contracts\Stateful;
        "#}
    }

    test_lint_fix! {
        name = global_single_segment_static_call_drops_leading_slash,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $dt = \DateTime::createFromFormat('Y', '2024');
        "#},
        fixed = indoc! {r#"
            <?php

            $dt = DateTime::createFromFormat('Y', '2024');
        "#}
    }

    test_lint_fix! {
        name = global_single_segment_type_hint_drops_leading_slash,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            function at(\DateTimeImmutable $when): void {}
        "#},
        fixed = indoc! {r#"
            <?php

            function at(DateTimeImmutable $when): void {}
        "#}
    }
}
