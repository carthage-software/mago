use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_span::HasSpan;
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
use crate::resolver::static_method::resolve_static_method_targets;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for StaticMethodCall<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let method_resolution =
            resolve_static_method_targets(context, block_context, artifacts, self.class, &self.method, self.span())?;

        let mut invocation_targets = vec![];
        for resolved_method in method_resolution.resolved_methods {
            let metadata = context
                .codebase
                .get_class_like(&resolved_method.classname)
                .expect("class-like metadata should exist for resolved method");

            let method_metadata = context
                .codebase
                .get_method_by_id(&resolved_method.method_identifier)
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
                inferred_return_type: None,
                method_context: Some(method_target_context),
                span: self.span(),
            });
        }

        analyze_invocation_targets(
            context,
            block_context,
            artifacts,
            method_resolution.template_result,
            invocation_targets,
            InvocationArgumentsSource::ArgumentList(&self.argument_list),
            self.span(),
            None,
            method_resolution.has_invalid_target,
            method_resolution.encountered_mixed,
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = calling_non_static_method_statically_is_ok,
        code = indoc! {r#"
            <?php

            class Example {
                private string $value = '';

                function doWork(): void {
                    $something = self::getSomething(); // Ok
                    $something .= $this->getSomething(); // Ok
                    $something .= Example::getSomething(); // Ok
                    $something .= static::getSomething(); // Ok

                    echo 'Doing work with: ' . $something;
                }

                function getSomething(): string {
                    return $this->value;
                }
            }

            class SubExample extends Example {
                function doWork(): void {
                    $something = self::getSomething(); // Ok
                    $something .= $this->getSomething(); // Ok
                    $something .= Example::getSomething(); // Ok
                    $something .= SubExample::getSomething(); // Ok
                    $something .= static::getSomething(); // Ok
                    $something .= parent::getSomething(); // Ok

                    echo 'Doing work with: ' . $something;
                }
            }

            trait TraitExample {
                function doWork(): void {
                    $something = self::getSomething(); // Ok
                    $something .= $this->getSomething(); // Ok
                    $something .= static::getSomething(); // Ok

                    echo 'Doing work with: ' . $something;
                }

                function getSomething(): string {
                    return 'Trait value';
                }
            }

            class TraitUser {
                use TraitExample;

                function doWorkToo(): void {
                    $something = self::getSomething(); // Ok
                    $something .= $this->getSomething(); // Ok
                    $something .= TraitUser::getSomething(); // Ok
                    $something .= static::getSomething(); // Ok

                    echo 'Doing work with: ' . $something;
                }
            }

            $e = new Example();
            $s = new SubExample();
            $t = new TraitUser();

            $e->doWork();
            $s->doWork();
            $t->doWork();
            $t->doWorkToo();
        "#}
    }

    test_analysis! {
        name = calling_static_method_on_interface_string,
        code = indoc! {r#"
            <?php

            interface Example {
                public static function doTheThing(): void;

                public static function getSomeValue(): int;
            }

            /**
             * @param array<class-string<Example>> $examples
             *
             * @return array<string, int>
             */
            function process(array $examples): array {
                $result = [];
                foreach ($examples as $example) {
                    $example::doTheThing();
                    $value = $example::getSomeValue();

                    $result[$example] = $value;
                }

                return $result;
            }
        "#},
        issues = [
            IssueCode::PossiblyStaticAccessOnInterface,
            IssueCode::PossiblyStaticAccessOnInterface,
        ]
    }

    test_analysis! {
        name = calling_static_method_on_interface_name,
        code = indoc! {r#"
            <?php

            interface Example {
                public static function doTheThing(): void;

                public static function getSomeValue(): int;
            }

            Example::doTheThing();

            echo Example::getSomeValue();
        "#},
        issues = [
            IssueCode::StaticAccessOnInterface,
            IssueCode::StaticAccessOnInterface,
            IssueCode::MixedArgument,
        ]
    }
}
