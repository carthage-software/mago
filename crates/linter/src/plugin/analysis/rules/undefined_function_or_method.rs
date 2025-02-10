use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_php_version::feature::Feature;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UndefinedFunctionOrMethodRule;

const ALLOW_DYNAMIC_STATIC_CALLS: &str = "allow_dynamic_static_calls";
const ALLOW_DYNAMIC_STATIC_CALLS_DEFAULT: bool = true;
const ALLOW_TRAIT_CALLS: &str = "allow_trait_calls";
const ALLOW_TRAIT_CALLS_DEFAULT: bool = false;

impl Rule for UndefinedFunctionOrMethodRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Undefined Function or Method", Level::Error)
            .with_description(indoc! {r#"
                Flags any call or closure creation referencing a non-existent function, or static method
                that doesn't exist, or is non-static/abstract. This prevents runtime errors and
                clarifies developer intent.
            "#})
            .with_option(RuleOptionDefinition {
                name: ALLOW_DYNAMIC_STATIC_CALLS,
                r#type: "boolean",
                description: "Allow dynamic static method calls via __callStatic.",
                default: Value::Boolean(ALLOW_DYNAMIC_STATIC_CALLS_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: ALLOW_TRAIT_CALLS,
                r#type: "boolean",
                description: "Allow static method calls on traits.",
                default: Value::Boolean(ALLOW_TRAIT_CALLS_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "Calling a defined static method",
                indoc! {r#"
                    <?php

                    class Foo {
                        public static function bar() { return 42; }
                    }

                    echo Foo::bar(); // OK: Foo::bar is defined and static.
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Calling a non-static method as static",
                indoc! {r#"
                    <?php

                    class Foo {
                        public function bar() { return 42; }
                    }

                    echo Foo::bar();
                    // Error: 'bar' is not static, so calling Foo::bar() is invalid.
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Calling an abstract static method directly",
                indoc! {r#"
                    <?php

                    abstract class Base {
                        abstract public static function doSomething();
                    }

                    Base::doSomething();
                    // Error: 'doSomething' is abstract and can't be called directly.
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Calling an undefined static method",
                indoc! {r#"
                    <?php

                    class Foo {
                        public static function bar() {}
                    }

                    Foo::baz();
                    // Error: 'baz' is not defined in class Foo
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::FunctionCall(function_call) => {
                let Expression::Identifier(identifier) = function_call.function.as_ref() else {
                    return LintDirective::default();
                };

                let function_name = context.resolve_function_name(identifier);
                let function_name_id = context.interner.intern(function_name);

                // If the function actually exists, no issue
                if context.codebase.function_exists(context.interner, &function_name_id) {
                    return LintDirective::default();
                }

                // Report undefined function usage
                context.report(
                    Issue::new(context.level(), format!("Call to undefined function `{function_name}`."))
                        .with_annotation(
                            Annotation::primary(identifier.span())
                                .with_message(format!("Function `{function_name}` does not exist.")),
                        )
                        .with_help(format!(
                            "Ensure the function `{function_name}` is defined or imported before using it."
                        )),
                );

                LintDirective::Continue
            }
            Node::StaticMethodCall(static_method_call) => {
                // Ensure we have a straightforward class identifier (not a variable, array access, etc.)
                let Expression::Identifier(class_identifier) = static_method_call.class.as_ref() else {
                    return LintDirective::default();
                };

                // Resolve the class name from the interner
                let class_id = context.semantics.names.get(class_identifier);
                let class_name = context.interner.lookup(class_id);

                // Check if this class/trait actually exists
                let Some(class_like) = context.codebase.get_named_class_like(context.interner, class_id) else {
                    // If not, report an undefined class
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Attempt to call static method on undefined class `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(class_identifier.span())
                                .with_message(format!("Class `{class_name}` does not exist.")),
                        )
                        .with_help(format!("Define or import `{class_name}` before using it.")),
                    );

                    return LintDirective::Continue;
                };

                // Check for interface usage
                if class_like.is_interface() {
                    context.report(
                        Issue::new(context.level(), "Calling a static method on an interface.")
                            .with_annotation(
                                Annotation::primary(class_identifier.span())
                                    .with_message(format!("`{class_name}` is an interface, not a class or an enum.")),
                            )
                            .with_note("Interfaces methods are abstract.")
                            .with_help("Use a concrete class instead."),
                    );

                    return LintDirective::Continue;
                }

                // Check if it's a trait, and if we allow static method calls on traits
                if class_like.is_trait() && context.php_version.is_deprecated(Feature::CallStaticMethodOnTrait) {
                    let allow_trait_calls = context
                        .option(ALLOW_TRAIT_CALLS)
                        .and_then(|o| o.as_bool())
                        .unwrap_or(ALLOW_TRAIT_CALLS_DEFAULT);

                    if !allow_trait_calls {
                        context.report(
                            Issue::new(context.level(), "Calling a static trait method is not allowed.".to_string())
                                .with_annotation(
                                    Annotation::primary(class_identifier.span())
                                        .with_message(format!("`{class_name}` is a trait, not a class or an enum.")),
                                )
                                .with_note("Creating closures from static methods on a trait is disallowed.")
                                .with_help("Use a concrete class instead."),
                        );
                    }
                }

                // Attempt to get a valid method name from the AST
                let ClassLikeMemberSelector::Identifier(method_identifier) = &static_method_call.method else {
                    return LintDirective::Continue;
                };

                let method_name = context.interner.lookup(&method_identifier.value);

                // If the method doesn't exist, check for __callStatic dynamic fallback
                let Some(method_info) = context.codebase.get_method(class_like, &method_identifier.value) else {
                    // If there's a __callStatic method, calls might be handled dynamically
                    if class_like.methods.appering_members.contains_key(&context.interner.intern("__callStatic")) {
                        let allow_dynamic_calls = context
                            .option(ALLOW_DYNAMIC_STATIC_CALLS)
                            .and_then(|o| o.as_bool())
                            .unwrap_or(ALLOW_DYNAMIC_STATIC_CALLS_DEFAULT);

                        if allow_dynamic_calls {
                            return LintDirective::Continue;
                        }
                    }

                    // Otherwise, it's truly undefined
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Call to undefined static method `{method_name}` on class `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("Method `{method_name}` does not exist on `{class_name}`.")),
                        )
                        .with_help(format!("Define `{method_name}` in `{class_name}` before calling it statically.")),
                    );

                    return LintDirective::Continue;
                };

                // Check if the method is truly static
                if !method_info.is_static {
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Attempt to call non-static method `{method_name}` statically on `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("`{method_name}` is not static in `{class_name}`.")),
                        )
                        .with_help(format!(
                            "Call `{method_name}` on an instance of `{class_name}` instead, or declare it as static."
                        )),
                    );
                }

                // Check if it's abstract, and whether we allow calling abstract methods
                if method_info.is_abstract && !should_allow_abstract_call(class_like, method_name) {
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Attempt to call abstract static method `{method_name}` on `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("Method `{method_name}` is abstract in `{class_name}`.")),
                        )
                        .with_help(format!("Provide a concrete implementation of `{method_name}` before calling it.")),
                    );
                }

                // Otherwise, all good
                LintDirective::Continue
            }
            Node::FunctionClosureCreation(function_closure_creation) => {
                let Expression::Identifier(identifier) = function_closure_creation.function.as_ref() else {
                    return LintDirective::default();
                };

                let function_name = context.resolve_function_name(identifier);
                let function_name_id = context.interner.intern(function_name);

                // If function actually exists, no issue
                if context.codebase.function_exists(context.interner, &function_name_id) {
                    return LintDirective::default();
                }

                // Otherwise, it's undefined
                context.report(
                    Issue::new(
                        context.level(),
                        format!("Attempt to create closure from undefined function `{function_name}`."),
                    )
                    .with_annotation(
                        Annotation::primary(identifier.span())
                            .with_message(format!("Function `{function_name}` does not exist.")),
                    )
                    .with_help(format!(
                        "Ensure `{function_name}` is defined or imported before referencing it as a closure."
                    )),
                );

                LintDirective::Prune
            }
            Node::StaticMethodClosureCreation(static_method_closure_creation) => {
                // Similar logic, but for creating a closure from a static method
                let Expression::Identifier(class_identifier) = static_method_closure_creation.class.as_ref() else {
                    return LintDirective::default();
                };

                let class_id = context.semantics.names.get(class_identifier);
                let class_name = context.interner.lookup(class_id);

                let Some(class_like) = context.codebase.get_named_class_like(context.interner, class_id) else {
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Creating a closure from a static method on undefined class `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(class_identifier.span())
                                .with_message(format!("Class `{class_name}` does not exist.")),
                        )
                        .with_help(format!("Define or import `{class_name}` before referencing it.")),
                    );

                    return LintDirective::Continue;
                };

                // Check for interface usage
                if class_like.is_interface() {
                    context.report(
                        Issue::new(context.level(), "Creating a closure from a static method on an interface.")
                            .with_annotation(
                                Annotation::primary(class_identifier.span())
                                    .with_message(format!("`{class_name}` is an interface, not a class or an enum.")),
                            )
                            .with_note("Interfaces methods are abstract.")
                            .with_help("Use a concrete class instead."),
                    );

                    return LintDirective::Continue;
                }

                // Check if it's a trait, and if we allow static method calls on traits
                if class_like.is_trait() && context.php_version.is_deprecated(Feature::CallStaticMethodOnTrait) {
                    let allow_trait_calls = context
                        .option(ALLOW_TRAIT_CALLS)
                        .and_then(|o| o.as_bool())
                        .unwrap_or(ALLOW_TRAIT_CALLS_DEFAULT);

                    if !allow_trait_calls {
                        context.report(
                            Issue::new(
                                context.level(),
                                "Creating a closure from a static trait method is not allowed.".to_string(),
                            )
                            .with_annotation(
                                Annotation::primary(class_identifier.span())
                                    .with_message(format!("`{class_name}` is a trait, not a class or an enum.")),
                            )
                            .with_note("Creating closures from static methods on a trait is disallowed.")
                            .with_help("Use a concrete class instead."),
                        );
                    }
                }

                // Attempt to get the method name
                let ClassLikeMemberSelector::Identifier(method_identifier) = &static_method_closure_creation.method
                else {
                    return LintDirective::Continue;
                };

                let method_name = context.interner.lookup(&method_identifier.value);

                // Check if the method is known, or if __callStatic can handle it
                let Some(method_info) = context.codebase.get_method(class_like, &method_identifier.value) else {
                    if class_like.methods.appering_members.contains_key(&context.interner.intern("__callStatic")) {
                        let allow_dynamic_calls = context
                            .option(ALLOW_DYNAMIC_STATIC_CALLS)
                            .and_then(|o| o.as_bool())
                            .unwrap_or(ALLOW_DYNAMIC_STATIC_CALLS_DEFAULT);

                        if allow_dynamic_calls {
                            return LintDirective::Continue;
                        }
                    }

                    context.report(
                        Issue::new(
                            context.level(),
                            format!(
                                "Creating a closure from an undefined static method `{class_name}::{method_name}`."
                            ),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("Method `{method_name}` does not exist on `{class_name}`.")),
                        )
                        .with_help(format!("Define `{class_name}::{method_name}` before referencing it as a closure.")),
                    );

                    return LintDirective::Continue;
                };

                // Check if it's truly static
                if !method_info.is_static {
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Creating a closure from non-static method `{class_name}::{method_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("Method `{method_name}` is not static in `{class_name}`.")),
                        )
                        .with_help(format!(
                            "Call this method on an instance or declare it as static in `{class_name}`."
                        )),
                    );
                }

                // Check if it's abstract
                if method_info.is_abstract && !should_allow_abstract_call(class_like, method_name) {
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Creating a closure from abstract static method `{class_name}::{method_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(method_identifier.span())
                                .with_message(format!("Method `{method_name}` is abstract in `{class_name}`.")),
                        )
                        .with_help("Provide a concrete implementation before creating a closure."),
                    );
                }

                LintDirective::Continue
            }
            _ => LintDirective::default(),
        }
    }
}

#[inline]
fn should_allow_abstract_call(class_like: &ClassLikeReflection, method_name: &str) -> bool {
    const BACKED_ENUM_METHODS: [&str; 2] = ["from", "tryFrom"];
    const UNIT_ENUM_METHODS: [&str; 1] = ["cases"];

    class_like.is_enum()
        && (UNIT_ENUM_METHODS.iter().any(|&m| m == method_name)
            || (class_like.backing_type.is_some() && BACKED_ENUM_METHODS.iter().any(|&m| m == method_name)))
}
