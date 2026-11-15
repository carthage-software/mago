use indoc::indoc;
use mago_allocator::Arena;
use mago_span::Span;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Call;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::ExpressionStatement;
use mago_syntax::cst::Hint;
use mago_syntax::cst::LiteralString;
use mago_syntax::cst::Method;
use mago_syntax::cst::MethodBody;
use mago_syntax::cst::MethodCall;
use mago_syntax::cst::MethodPartialApplication;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::NullSafeMethodCall;
use mago_syntax::cst::StaticMethodCall;
use mago_syntax::cst::StaticMethodPartialApplication;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnusedReturnValueRule {
    meta: &'static RuleMeta,
    cfg: NoUnusedReturnValueConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoUnusedReturnValueConfig {
    pub level: Level,
}

impl Default for NoUnusedReturnValueConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoUnusedReturnValueConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnusedReturnValueRule {
    type Config = NoUnusedReturnValueConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unused Return Value",
            code: "no-unused-return-value",
            description: indoc! {"
                Detects private methods whose return value is thrown away at every call site.

                Only the class itself can call a private method, so when each call is a bare
                statement, nothing reads the value the method returns. Declaring the method
                `void` makes that clear.

                The rule stays silent when the class uses a trait, calls methods by a dynamic
                name, takes the method as a callable, or mentions its name in a string.
            "},
            good_example: indoc! {r"
                <?php

                final class Counter
                {
                    private int $count = 0;

                    public function increment(): void
                    {
                        $this->advance();
                    }

                    private function advance(): void
                    {
                        ++$this->count;
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                final class Counter
                {
                    private int $count = 0;

                    public function increment(): void
                    {
                        $this->advance();
                    }

                    private function advance(): int
                    {
                        return ++$this->count;
                    }
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class, NodeKind::Enum, NodeKind::AnonymousClass];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let members = match node {
            Node::Class(class) => &class.members,
            Node::Enum(r#enum) => &r#enum.members,
            Node::AnonymousClass(class) => &class.members,
            _ => return,
        };

        let mut candidates = Vec::new();
        for member in members.iter() {
            match member {
                ClassLikeMember::TraitUse(_) => return,
                ClassLikeMember::Method(method)
                    if method.modifiers.contains_private()
                        && matches!(method.body, MethodBody::Concrete(_))
                        && !method.name.value.starts_with(b"__")
                        && method
                            .return_type_hint
                            .as_ref()
                            .is_some_and(|hint| !matches!(hint.hint, Hint::Void(_) | Hint::Never(_))) =>
                {
                    candidates.push(Candidate { method, discarded: 0, consumed: false });
                }
                _ => {}
            }
        }

        if candidates.is_empty() {
            return;
        }

        let mut walker = CallSites { candidates, discarded_call: None, dynamic: false };
        for member in members.iter() {
            walker.walk_class_like_member(member, &mut ());
        }

        if walker.dynamic {
            return;
        }

        for candidate in walker.candidates {
            let (Some(return_type_hint), 1..) = (&candidate.method.return_type_hint, candidate.discarded) else {
                continue;
            };

            if candidate.consumed {
                continue;
            }

            let name = mago_bytes::BytesDisplay(candidate.method.name.value);
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("The return value of `{name}()` is never used."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(return_type_hint.hint.span())
                            .with_message("No caller reads this return value"),
                    )
                    .with_annotation(
                        Annotation::secondary(candidate.method.name.span)
                            .with_message("This private method's result is discarded at every call"),
                    )
                    .with_help("Declare the method `void` and stop returning a value, or use the result where the method is called."),
            );
        }
    }
}

struct Candidate<'ast, 'arena> {
    method: &'ast Method<'arena>,
    discarded: usize,
    consumed: bool,
}

struct CallSites<'ast, 'arena> {
    candidates: Vec<Candidate<'ast, 'arena>>,
    discarded_call: Option<Span>,
    dynamic: bool,
}

impl<'ast, 'arena> CallSites<'ast, 'arena> {
    fn find(&mut self, name: &[u8]) -> Option<&mut Candidate<'ast, 'arena>> {
        self.candidates.iter_mut().find(|candidate| candidate.method.name.value.eq_ignore_ascii_case(name))
    }

    fn reference(&mut self, selector: &ClassLikeMemberSelector<'arena>, span: Span) {
        let ClassLikeMemberSelector::Identifier(name) = selector else {
            self.dynamic = true;

            return;
        };

        if self.discarded_call == Some(span) {
            self.discarded_call = None;
        } else if let Some(candidate) = self.find(name.value) {
            candidate.consumed = true;
        }
    }
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for CallSites<'ast, 'arena> {
    fn walk_in_statement_expression(&mut self, statement: &'ast ExpressionStatement<'arena>, _: &mut ()) {
        let mut expression = statement.expression;
        while let Expression::Parenthesized(parenthesized) = expression {
            expression = parenthesized.expression;
        }

        let Expression::Call(call) = expression else {
            return;
        };

        let (Call::Method(MethodCall { method: ClassLikeMemberSelector::Identifier(name), .. })
        | Call::NullSafeMethod(NullSafeMethodCall { method: ClassLikeMemberSelector::Identifier(name), .. })
        | Call::StaticMethod(StaticMethodCall { method: ClassLikeMemberSelector::Identifier(name), .. })) = call
        else {
            return;
        };

        if let Some(candidate) = self.find(name.value) {
            candidate.discarded += 1;
            self.discarded_call = Some(call.span());
        }
    }

    fn walk_in_method_call(&mut self, call: &'ast MethodCall<'arena>, _: &mut ()) {
        self.reference(&call.method, call.span());
    }

    fn walk_in_null_safe_method_call(&mut self, call: &'ast NullSafeMethodCall<'arena>, _: &mut ()) {
        self.reference(&call.method, call.span());
    }

    fn walk_in_static_method_call(&mut self, call: &'ast StaticMethodCall<'arena>, _: &mut ()) {
        self.reference(&call.method, call.span());
    }

    fn walk_in_method_partial_application(&mut self, partial: &'ast MethodPartialApplication<'arena>, _: &mut ()) {
        self.reference(&partial.method, partial.span());
    }

    fn walk_in_static_method_partial_application(
        &mut self,
        partial: &'ast StaticMethodPartialApplication<'arena>,
        _: &mut (),
    ) {
        self.reference(&partial.method, partial.span());
    }

    fn walk_in_literal_string(&mut self, string: &'ast LiteralString<'arena>, _: &mut ()) {
        if let Some(candidate) = string.value.and_then(|value| self.find(value)) {
            candidate.consumed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnusedReturnValueRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = reports_private_method_whose_result_every_call_discards,
        rule = NoUnusedReturnValueRule,
        count = 2,
        code = indoc! {r"
            <?php

            final class Counter
            {
                private int $count = 0;

                public function increment(): void
                {
                    $this->advance();
                    ($this->advance());
                    self::reset();
                }

                public function getCount(): int
                {
                    return $this->count;
                }

                private function advance(): int
                {
                    return ++$this->count;
                }

                private static function reset(): bool
                {
                    return true;
                }
            }
        "}
    }

    test_lint_failure! {
        name = reports_in_enums_and_anonymous_classes,
        rule = NoUnusedReturnValueRule,
        count = 2,
        code = indoc! {r"
            <?php

            enum Status
            {
                case Active;

                public function touch(): void
                {
                    $this->log();
                }

                private function log(): string
                {
                    return 'logged';
                }
            }

            $object = new class {
                public function run(): void
                {
                    $this?->prepare();
                }

                private function prepare(): array
                {
                    return [];
                }
            };
        "}
    }

    test_lint_success! {
        name = ignores_results_that_are_read,
        rule = NoUnusedReturnValueRule,
        code = indoc! {r"
            <?php

            final class Reader
            {
                public function run(): int
                {
                    $this->first();
                    $value = $this->first();

                    $this->second();
                    echo $this->second();

                    $this->third();
                    $callable = $this->third(...);

                    $this->fourth();
                    array_map([$this, 'fourth'], []);

                    $this->fifth();
                    $values = [fn() => $this->fifth()];

                    return $value;
                }

                private function first(): int { return 1; }
                private function second(): string { return ''; }
                private function third(): int { return 3; }
                private function fourth(): int { return 4; }
                private function fifth(): int { return 5; }
            }
        "}
    }

    test_lint_success! {
        name = ignores_methods_the_rule_cannot_follow,
        rule = NoUnusedReturnValueRule,
        code = indoc! {r"
            <?php

            final class Ignored
            {
                public function run(): void
                {
                    $this->nothing();
                    $this->untyped();
                    $this->open();
                }

                private function nothing(): void {}
                private function untyped() { return 1; }
                public function open(): int { return 1; }
                private function never(): int { return 1; }
            }

            final class UsesTrait
            {
                use SomeTrait;

                public function run(): void
                {
                    $this->advance();
                }

                private function advance(): int { return 1; }
            }

            final class CallsDynamically
            {
                public function run(string $method): void
                {
                    $this->advance();
                    $this->{$method}();
                }

                private function advance(): int { return 1; }
            }
        "}
    }
}
