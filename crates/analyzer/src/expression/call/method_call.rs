use mago_codex::get_class_like;
use mago_codex::get_method_by_id;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::resolver::method::resolve_method_targets;

impl Analyzable for MethodCall {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_method_call(
            context,
            block_context,
            artifacts,
            &self.object,
            &self.method,
            &self.argument_list,
            false, // is_nullsafe
            self.span(),
        )
    }
}

impl Analyzable for NullSafeMethodCall {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_method_call(
            context,
            block_context,
            artifacts,
            &self.object,
            &self.method,
            &self.argument_list,
            true, // is_nullsafe
            self.span(),
        )
    }
}

fn analyze_method_call<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    object: &Expression,
    selector: &ClassLikeMemberSelector,
    argument_list: &ArgumentList,
    is_null_safe: bool,
    span: Span,
) -> Result<(), AnalysisError> {
    let method_resultion =
        resolve_method_targets(context, block_context, artifacts, object, selector, is_null_safe, span)?;

    let mut invocation_targets = vec![];
    for resolved_method in method_resultion.resolved_methods {
        let metadata = get_class_like(context.codebase, context.interner, &resolved_method.classname)
            .expect("class-like metadata should exist for resolved method");

        let method_metadata = get_method_by_id(context.codebase, context.interner, &resolved_method.method_identifier)
            .expect("method metadata should exist for resolved method");

        let method_target_context = MethodTargetContext {
            declaring_method_id: Some(resolved_method.method_identifier),
            class_like_metadata: metadata,
            class_type: resolved_method.static_class_type,
        };

        invocation_targets.push(InvocationTarget::FunctionLike {
            identifier: FunctionLikeIdentifier::Method(
                *resolved_method.method_identifier.get_class_name(),
                *resolved_method.method_identifier.get_method_name(),
            ),
            metadata: method_metadata,
            method_context: Some(method_target_context),
            span,
        });
    }

    analyze_invocation_targets(
        context,
        block_context,
        artifacts,
        method_resultion.template_result,
        invocation_targets,
        InvocationArgumentsSource::ArgumentList(argument_list),
        span,
        method_resultion.has_invalid_target,
        method_resultion.encountered_mixed,
        is_null_safe && method_resultion.encountered_null,
    )
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = nullsafe_method_call_on_null,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            interface WriteInterface
            {
                /**
                 * @param non-empty-string $data
                 */
                public function write(string $data): void;
            }

            function get_writer(): null|WriteInterface
            {
                return null;
            }

            function write_line(string $message): void
            {
                $message = $message . "\n";

                get_writer()?->write($message);
            }
        "#}
    }

    test_analysis! {
        name = possible_method_call_on_null,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            interface WriteInterface
            {
                /**
                 * @param non-empty-string $data
                 */
                public function write(string $data): void;
            }

            function get_writer(): null|WriteInterface
            {
                return null;
            }

            function write_line(string $message): void
            {
                $message = $message . "\n";

                get_writer()->write($message);
            }
        "#},
        issues = [
            TypingIssueKind::PossibleMethodAccessOnNull
        ]
    }

    test_analysis! {
        name = method_call_on_mixed,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            function get_mixed(): mixed
            {
                return "Hello, World!";
            }

            function call_method_on_mixed(): void
            {
                $mixed = get_mixed();
                $mixed->someMethod();
            }
        "#},
        issues = [
            TypingIssueKind::MixedAssignment,
            TypingIssueKind::MixedMethodAccess
        ]
    }

    test_analysis! {
        name = method_call_on_mixed_any,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            function call_method_on_mixed_any(): void
            {
                $mixed_any->someMethod();
            }
        "#},
        issues = [
            TypingIssueKind::UndefinedVariable,
            TypingIssueKind::MixedAnyMethodAccess
        ]
    }

    test_analysis! {
        name = method_call_on_non_object,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            function call_method_on_non_object(): void
            {
                $non_object = 42;
                $non_object->someMethod();
            }
        "#},
        issues = [
            TypingIssueKind::InvalidMethodAccess
        ]
    }

    test_analysis! {
        name = method_call_on_generic_parameter,
        code = indoc! {r#"
            <?php

            class A
            {
                public function getString(): string
                {
                    return 'Hello, world!';
                }
            }

            class B
            {
                public function getString(): string
                {
                    return 'Hello, world!';
                }
            }

            /**
             * @template T of A|B
             *
             * @param T $object
             */
            function foo(A|B $object): string
            {
                return $object->getString();
            }
        "#},
    }

    test_analysis! {
        name = ambiguous_object_method_call,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            function call_ambiguous_method(object $obj): void
            {
                $obj->someMethod();
            }
        "#},
        issues = [
            TypingIssueKind::AmbiguousObjectMethodAccess
        ]
    }

    test_analysis! {
        name = template_resolution,
        code = indoc! {r#"
            <?php

            /**
             * @template-covariant T
             */
            interface TypeInterface
            {
                /**
                 * @param mixed $value
                 * @return T
                 */
                public function assert(mixed $value): mixed;
            }

            /**
             * @param TypeInterface<non-empty-string> $type
             *
             * @return string
             */
            function to_string(mixed $value, TypeInterface $type): string
            {
                return $type->assert($value);
            }
        "#},
    }

    test_analysis! {
        name = intersection_read_write_calls,
        code = indoc! {r#"
            <?php

            interface ReadHandle {
                public function read(): string;
            }

            interface WriteHandle {
                public function write(string $data): void;
            }

            /**
             * @template T as array-key
             * @param iterable<T, ReadHandle&WriteHandle> $handles
             * @return array<T, string>
             */
            function task(iterable $handles): array {
                $result = [];
                foreach ($handles as $index => $handle) {
                    $data = $handle->read();
                    $handle->write($data);

                    $result[$index] = $data;
                }
                return $result;
            }
        "#},
    }

    test_analysis! {
        name = intersection_template_resolution,
        code = indoc! {r#"
            <?php

            interface MockObject
            {
            }

            abstract class TestCase
            {
                /**
                 * @template T of object
                 *
                 * @param class-string<T> $className
                 *
                 * @return MockObject&T
                 */
                protected function createMock(string $className): MockObject
                {
                    exit('Not implemented');
                }

                /**
                 * @template T of object
                 *
                 * @param class-string<T> $className
                 *
                 * @return T&MockObject
                 */
                protected function createMockTwo(string $className): MockObject
                {
                    exit('Not implemented');
                }
            }

            interface ServiceInterface
            {
            }

            class MyTestCase extends TestCase
            {
                private null|(MockObject&ServiceInterface) $service = null;

                public function setup(): void
                {
                    $this->service = $this->createMock(ServiceInterface::class);
                    $this->service = $this->createMockTwo(ServiceInterface::class);
                }
            }
        "#},
    }
}
