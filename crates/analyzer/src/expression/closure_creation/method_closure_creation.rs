use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::get_mixed_closure;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::method::resolve_method_targets;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for MethodClosureCreation<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let method_resolution =
            resolve_method_targets(context, block_context, artifacts, self.object, &self.method, false, self.span())?;

        let mut callables = vec![];
        for resolved_method in method_resolution.resolved_methods {
            callables.push(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Method(
                *resolved_method.method_identifier.get_class_name(),
                *resolved_method.method_identifier.get_method_name(),
            ))));
        }

        let resulting_type = if callables.is_empty() {
            if method_resolution.has_invalid_target { get_never() } else { get_mixed_closure() }
        } else {
            TUnion::from_vec(callables)
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::code::IssueCode;
    use crate::test_analysis;
    use indoc::indoc;

    test_analysis! {
        name = method_closure_creation_happy_path,
        code = indoc! {r#"
            <?php
            class Greeter {
                public function greet(string $name): string {
                    return "Hello, " . $name;
                }
            }

            /** @param (callable(string):string) $cb */
            function call_it(callable $cb): void {
                echo $cb("World");
            }

            $greeter = new Greeter();
            $closure = $greeter->greet(...);
            call_it($closure);
        "#},
        issues = []
    }

    test_analysis! {
        name = method_closure_creation_on_non_object,
        code = indoc! {r#"
            <?php
            $my_string = "hello";
            $closure = $my_string->method(...);
        "#},
        issues = [
            IssueCode::InvalidMethodAccess,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_ambiguous_object,
        code = indoc! {r#"
            <?php
            /** @param object $obj */
            function test($obj) {
                $_closure = $obj->method(...);
            }
        "#},
        issues = [
            IssueCode::AmbiguousObjectMethodAccess,
        ]
    }

    test_analysis! {
        name = method_closure_creation_basic_happy_path,
        code = indoc! {r#"
            <?php
            class Greeter {
                public function greet(string $name): string {
                    return "Hello, " . $name;
                }
            }
            /** @param callable(string):string $cb */
            function call_it(callable $cb): void {
                echo $cb("World");
            }

            $greeter = new Greeter();
            $closure = $greeter->greet(...);
            call_it($closure);
        "#},
        issues = []
    }

    test_analysis! {
        name = method_closure_creation_from_parent_method,
        code = indoc! {r#"
            <?php
            class ParentGreeter {
                public function say_hi(): string { return "hi"; }
            }

            class ChildGreeter extends ParentGreeter {}

            /** @param callable():string $_cb */
            function call_it(callable $_cb): void {}

            $greeter = new ChildGreeter();
            $closure = $greeter->say_hi(...);
            call_it($closure);
        "#},
    }

    test_analysis! {
        name = method_closure_creation_from_interface_method,
        code = indoc! {r#"
            <?php
            interface Logger { public function log(string $message): void; }
            class FileLogger implements Logger {
                public function log(string $message): void {}
            }

            /** @param callable(string):void $_cb */
            function call_it(callable $_cb): void {}

            $logger = new FileLogger();
            $closure = $logger->log(...);
            call_it($closure);
        "#},
    }

    test_analysis! {
        name = method_closure_creation_with_dynamic_literal_string_name,
        code = indoc! {r#"
            <?php
            class DynamicCaller {
                public function method(): int { return 1; }
            }

            /** @param callable():int $_cb */
            function call_it(callable $_cb): void {}

            $obj = new DynamicCaller();
            $method = "method";
            $closure = $obj->{$method}(...);
            call_it($closure);
        "#},
        issues = []
    }

    test_analysis! {
        name = method_closure_creation_non_existent_method,
        code = indoc! {r#"
            <?php
            class MyClass {}
            $obj = new MyClass();
            $closure = $obj->undefinedMethod(...);
        "#},
        issues = [
            IssueCode::NonExistentMethod,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_non_object_type,
        code = indoc! {r#"
            <?php
            $my_string = "hello";
            $closure = $my_string->method(...);
        "#},
        issues = [
            IssueCode::InvalidMethodAccess,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_definitely_null,
        code = indoc! {r#"
            <?php
            $obj = null;
            $closure = $obj->method(...);
        "#},
        issues = [
            IssueCode::MethodAccessOnNull,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_possibly_null_object,
        code = indoc! {r#"
            <?php
            class MyClass { public function method(): void {} }
            /** @param MyClass|null $obj */
            function test($obj) {
                $_closure = $obj->method(...);
            }
        "#},
        issues = [
            IssueCode::PossibleMethodAccessOnNull,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_mixed_type,
        code = indoc! {r#"
            <?php
            /** @param mixed $obj */
            function test($obj) {
                $_closure = $obj->method(...);
            }
        "#},
        issues = [
            IssueCode::MixedMethodAccess,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_generic_object,
        code = indoc! {r#"
            <?php
            /** @param object $obj */
            function test($obj) {
                $_closure = $obj->method(...);
            }
        "#},
        issues = [
            IssueCode::AmbiguousObjectMethodAccess,
        ]
    }

    test_analysis! {
        name = method_closure_creation_with_string_name,
        code = indoc! {r#"
            <?php
            class DynamicCaller {
                public function method_foo(): int { return 1; }
            }
            $obj = new DynamicCaller();
            $methodName = "method" . "_foo"; // non-literal string

            $_closure = $obj->{$methodName}(...);
        "#},
    }

    test_analysis! {
        name = method_closure_creation_with_invalid_selector_type,
        code = indoc! {r#"
            <?php
            class DynamicCaller {
                public function methodA(): int { return 1; }
            }
            $obj = new DynamicCaller();
            $methodName = 123;
            $_closure = $obj->{$methodName}(...);
        "#},
        issues = [
            IssueCode::InvalidMemberSelector,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_union_of_object_and_non_object,
        code = indoc! {r#"
            <?php

            class MyClass { public function method(): void {} }

            function test(MyClass|int $val) {
                $_closure = $val->method(...);
            }
        "#},
        issues = [
            IssueCode::InvalidMethodAccess,
        ]
    }

    test_analysis! {
        name = method_closure_creation_on_union_where_one_lacks_method,
        code = indoc! {r#"
            <?php
            class ClassA { public function thing(): void {} }
            class ClassB { /* has no thing method */ }

            function test(ClassA|ClassB $obj) {
                $_closure = $obj->thing(...);
            }
        "#},
        issues = [
            IssueCode::NonExistentMethod,
        ]
    }
}
