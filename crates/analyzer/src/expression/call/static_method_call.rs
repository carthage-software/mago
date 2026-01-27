use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::StaticMethodCall;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::plugin::ExpressionHookResult;
use crate::plugin::context::HookContext;
use crate::resolver::static_method::resolve_static_method_targets;
use crate::utils::expression::expression_is_nullsafe;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for StaticMethodCall<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if context.plugin_registry.has_static_method_call_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            let result = context.plugin_registry.before_static_method_call(self, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }

            match result {
                ExpressionHookResult::Continue => {}
                ExpressionHookResult::Skip => {
                    return Ok(());
                }
                ExpressionHookResult::SkipWithType(ty) => {
                    artifacts.set_expression_type(&self.span(), ty);
                    return Ok(());
                }
            }
        }

        if block_context.flags.collect_initializations()
            && let Expression::Parent(_) = self.class
            && let ClassLikeMemberSelector::Identifier(method_ident) = &self.method
        {
            if method_ident.value.eq_ignore_ascii_case("__construct") {
                block_context.flags.set_calls_parent_constructor(true);
            } else {
                let method_name = mago_atom::ascii_lowercase_atom(method_ident.value);
                if context.settings.class_initializers.contains(&method_name) {
                    block_context.calls_parent_initializer = Some(method_name);
                }
            }
        }

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

        let class_has_nullsafe_null = artifacts.get_expression_type(self.class).is_some_and(|t| t.has_nullsafe_null());

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
            expression_is_nullsafe(self.class) || method_resolution.encountered_null,
            class_has_nullsafe_null,
            method_resolution.all_methods_non_nullable_return,
        )?;

        if context.plugin_registry.has_static_method_call_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.after_static_method_call(self, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = calling_non_static_method_statically_is_ok,
        code = indoc! {r"
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
        "}
    }

    test_analysis! {
        name = calling_static_method_on_interface_string,
        code = indoc! {r"
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
        "},
        issues = [
            IssueCode::PossiblyStaticAccessOnInterface,
            IssueCode::PossiblyStaticAccessOnInterface,
        ]
    }

    test_analysis! {
        name = calling_static_method_on_interface_name,
        code = indoc! {r"
            <?php

            interface Example {
                public static function doTheThing(): void;

                public static function getSomeValue(): int;
            }

            Example::doTheThing();

            echo Example::getSomeValue();
        "},
        issues = [
            IssueCode::StaticAccessOnInterface,
            IssueCode::StaticAccessOnInterface,
            IssueCode::MixedArgument,
        ]
    }
}
