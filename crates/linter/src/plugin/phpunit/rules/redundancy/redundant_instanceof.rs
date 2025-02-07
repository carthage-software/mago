use indoc::indoc;

use mago_ast::*;
use mago_ast_utils::reference::MethodReference;
use mago_fixer::SafetyClassification;
use mago_interner::StringIdentifier;
use mago_reporting::*;
use mago_span::HasPosition;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::phpunit::rules::utils::find_assertion_references_in_method;
use crate::rule::Rule;

/// A PHPUnit rule that enforces the use of strict assertions.
#[derive(Clone, Debug)]
pub struct RedundantInstanceOfRule;

impl Rule for RedundantInstanceOfRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant InstanceOf", Level::Warning)
            .with_description(indoc! {"
                Detects redundant `instanceof` assertions in test methods.
                An `instanceof` assertion is redundant if the subject is always an instance of the class being checked.
            "})
            .with_example(RuleUsageExample::valid(
                "A non-redundant `instanceof` assertion",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            $this->assertInstanceOf(Foo::class, get_foo_or_bar());
                            $this->assertInstanceOf(Bar::class, get_foo_or_bar());
                        }
                    }

                    final class Foo {}
                    final class Bar {}

                    function get_foo_or_bar(): Foo|Bar {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant `instanceof` assertion that always passes",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            $this->assertInstanceOf(Foo::class, new Foo());
                        }
                    }

                    final class Foo {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant `instanceof` assertion that always fails",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            $this->assertInstanceOf(Foo::class, new Bar());
                        }
                    }

                    final class Foo {}
                    final class Bar {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant `instanceof` assertion with an interface",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            $this->assertInstanceOf(FooInterface::class, new Foo());
                        }
                    }

                    interface FooInterface {}
                    final class Foo implements FooInterface {}
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Method(method) = node else { return LintDirective::default() };

        let name = context.lookup(&method.name.value);
        if !name.starts_with("test") || name.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase()) {
            return LintDirective::Prune;
        }

        for reference in find_assertion_references_in_method(method, context, &["assertInstanceOf"]) {
            let arguments = match reference {
                MethodReference::MethodCall(method_call) => &method_call.argument_list.arguments,
                MethodReference::StaticMethodCall(static_method_call) => &static_method_call.argument_list.arguments,
                _ => continue,
            };

            let Some(class_name) = arguments.get(0).map(|arg| arg.value()).and_then(|v| get_class_name(v, context))
            else {
                continue;
            };

            let Some(object_name) = arguments.get(1).map(|arg| arg.value()).and_then(|v| get_object_name(v, context))
            else {
                continue;
            };

            let Some(class_reflection) = context.codebase.get_named_class_like(context.interner, class_name) else {
                continue;
            };

            let Some(object_reflection) = context.codebase.get_named_class_like(context.interner, object_name) else {
                continue;
            };

            if object_reflection.inheritance.is_instance_of(context.interner, class_reflection) {
                let issue = Issue::new(context.level(), "Redundant `instanceof` assertion.")
                    .with_annotation(Annotation::primary(reference.span()).with_message(format!(
                        "This `instanceof` assertion is redundant because `{}` is always an instance of `{}`.",
                        context.interner.lookup(object_name),
                        context.interner.lookup(class_name)
                    )))
                    .with_help("Remove this `instanceof` assertion because it is redundant and always true.");

                context.propose(issue, |fix| {
                    fix.delete(reference.span().to_range(), SafetyClassification::Safe);
                });
            } else {
                let issue = Issue::new(context.level(), "Redundant `instanceof` assertion.")
                    .with_annotation(Annotation::primary(reference.span()).with_message(format!(
                        "This `instanceof` assertion is redundant because `{}` is never an instance of `{}`.",
                        context.interner.lookup(object_name),
                        context.interner.lookup(class_name)
                    )))
                    .with_help("Remove this `instanceof` assertion because it is redundant and always false.");

                context.propose(issue, |fix| {
                    // we consider this to be potentially unsafe because this
                    // change can alter the behavior of the result of the test
                    // (e.g. if the test is expecting an exception to be thrown)
                    fix.delete(reference.span().to_range(), SafetyClassification::PotentiallyUnsafe);
                });
            }
        }

        LintDirective::Prune
    }
}

/// Returns the class name of the given expression if it is a class constant access to the `class` constant.
///
/// Given `Foo::class`, this function would return `Foo` resolved to its fully qualified name.
///
/// # Parameters
///
/// - `expression`: The expression to check.
/// - `context`: The lint context.
///
/// # Returns
///
/// The class name if the expression is a class constant access to the `class` constant
/// as a `StringIdentifier` or `None` if it is not.
fn get_class_name<'a>(expression: &Expression, context: &'a LintContext) -> Option<&'a StringIdentifier> {
    // Is this an access?
    let Expression::Access(access) = expression else {
        return None;
    };

    // Is this a class constant access?
    let Access::ClassConstant(class_constant_access) = access else {
        return None;
    };

    // Is the LHS an identifier?
    let Expression::Identifier(class_name) = class_constant_access.class.as_ref() else {
        return None;
    };

    // Is the RHS an identifier?
    let ClassLikeConstantSelector::Identifier(value) = &class_constant_access.constant else {
        return None;
    };

    // Is the RHS an identifier named "class"?
    if context.interner.lookup(&value.value) != "class" {
        return None;
    }

    Some(context.semantics.names.get(&class_name.position()))
}

/// Returns the name of the object instantiated by the given expression.
///
/// Given `new Foo()`, this function would return `Foo` resolved to its fully qualified name.
///
/// # Parameters
///
/// - `expression`: The expression to check.
/// - `context`: The lint context.
///
/// # Returns
///
/// The name of the object instantiated by the expression as a `StringIdentifier`
/// or `None` if it is not an instantiation.
fn get_object_name<'a>(expression: &Expression, context: &'a LintContext) -> Option<&'a StringIdentifier> {
    let Expression::Instantiation(instantiation) = expression else {
        return None;
    };

    let Expression::Identifier(class_name) = instantiation.class.as_ref() else {
        return None;
    };

    Some(context.semantics.names.get(&class_name.position()))
}
