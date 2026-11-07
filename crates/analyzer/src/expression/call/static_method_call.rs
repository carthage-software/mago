use mago_allocator::Arena;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_span::HasSpan;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::StaticMethodCall;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::expression::call::method_call::analyze_undocumented_method_return_type;
use crate::expression::call::method_call::prepare_unresolved_method_targets;
use crate::expression::call::record_external_method_call;
use crate::expression::call::record_external_method_call_targets;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodInvocationKind;
use crate::invocation::MethodTargetContext;
use crate::plugin::ExpressionHookResult;
use crate::plugin::context::HookContext;
use crate::resolver::static_method::resolve_static_method_targets;
use crate::utils::expression::expression_is_nullsafe;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for StaticMethodCall<'arena> {
    #[allow(clippy::expect_used)]
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        if context.plugin_registry.has_static_method_call_hooks() {
            let mut hook_context = HookContext::new(context.codebase, context.source_file, block_context, artifacts);
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
            if method_ident.value.eq_ignore_ascii_case(b"__construct") {
                block_context.flags.set_calls_parent_constructor(true);
            } else {
                let method_name = mago_word::ascii_lowercase_word(method_ident.value);
                let parent_meta = block_context
                    .scope
                    .get_class_like()
                    .and_then(|m| m.direct_parent_class)
                    .and_then(|p| context.codebase.get_class_like(p.as_bytes()));
                if let Some(parent_meta) = parent_meta
                    && context.is_class_initializer_for(parent_meta, method_name)
                {
                    block_context.calls_parent_initializer = Some(method_name);
                }
            }
        }

        let mut method_resolution =
            resolve_static_method_targets(context, block_context, artifacts, self.class, &self.method, self.span())?;
        let undocumented_template_result =
            (!method_resolution.undocumented_methods.is_empty()).then(|| method_resolution.template_result.clone());

        let mut invocation_targets = vec![];
        for resolved_method in method_resolution.resolved_methods {
            let metadata = context
                .codebase
                .get_class_like(resolved_method.classname.as_bytes())
                .expect("class-like metadata should exist for resolved method");

            let method_metadata = context
                .codebase
                .get_method_by_id(&resolved_method.method_identifier)
                .expect("method metadata should exist for resolved method");

            let method_display = format!(
                "{}::{}",
                resolved_method.method_identifier.get_class_name(),
                resolved_method.method_identifier.get_method_name(),
            );
            crate::utils::availability::check_method_availability(
                context,
                method_metadata,
                &method_display,
                self.span(),
            );

            let method_target_context = MethodTargetContext {
                invocation_kind: MethodInvocationKind::Static,
                declaring_method_id: Some(resolved_method.method_identifier),
                class_like_metadata: metadata,
                class_type: resolved_method.static_class_type,
                declaring_object_type: resolved_method.declaring_object,
            };

            invocation_targets.push(InvocationTarget::FunctionLike {
                identifier: FunctionLikeIdentifier::Method(
                    resolved_method.method_identifier.get_class_name(),
                    resolved_method.method_identifier.get_method_name(),
                ),
                metadata: method_metadata,
                inferred_return_type: None,
                effective_signature: None,
                method_context: Some(method_target_context),
                span: self.span(),
            });
        }

        if !method_resolution.unresolved_methods.is_empty() {
            method_resolution.has_invalid_target |= prepare_unresolved_method_targets(
                context,
                artifacts,
                std::mem::take(&mut method_resolution.unresolved_methods),
                InvocationArgumentsSource::ArgumentList(&self.argument_list),
                self.span(),
                MethodInvocationKind::Static,
                &mut invocation_targets,
            )?;
        }

        let has_resolved_methods = !invocation_targets.is_empty();

        record_external_method_call(context, artifacts, &invocation_targets, self.span());
        if !method_resolution.undocumented_methods.is_empty() {
            record_external_method_call_targets(
                context,
                artifacts,
                method_resolution.undocumented_methods.iter().map(|method| (method.classname, method.method_name)),
                self.span(),
            );
        }

        let class_has_nullsafe_null = artifacts.get_expression_type(self.class).is_some_and(|t| t.has_nullsafe_null());

        if has_resolved_methods || method_resolution.undocumented_methods.is_empty() {
            analyze_invocation_targets(
                context,
                block_context,
                artifacts,
                method_resolution.template_result,
                invocation_targets,
                InvocationArgumentsSource::ArgumentList(&self.argument_list),
                self.span(),
                None,
                method_resolution.encountered_mixed,
                expression_is_nullsafe(self.class) || method_resolution.encountered_null,
                class_has_nullsafe_null,
            )?;
        }

        if !method_resolution.undocumented_methods.is_empty() {
            analyze_undocumented_method_return_type(
                context,
                block_context,
                artifacts,
                method_resolution.undocumented_methods,
                undocumented_template_result.as_ref().expect("undocumented calls have a template result"),
                &self.argument_list,
                self.span(),
                MethodInvocationKind::Static,
                !has_resolved_methods,
            )?;
        }

        if context.plugin_registry.has_static_method_call_hooks() {
            let mut hook_context = HookContext::new(context.codebase, context.source_file, block_context, artifacts);
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
        name = abstract_method_trait_parent_class_string,
        code = indoc! {"
            <?php

            abstract class BaseItem {
                abstract public static function getValue(): int;
            }

            class ParentItem extends BaseItem {
                public static function getValue(): int {
                    return 42;
                }
            }

            /** @phpstan-require-extends BaseItem */
            trait ItemTrait {
                public static function getTotal(): int {
                    $class = parent::class;
                    return $class::getValue();
                }

                public static function getInvalidTotal(): int {
                    return BaseItem::getValue();
                }
            }

            final class Item extends ParentItem {
                use ItemTrait;
            }

            echo Item::getTotal();
        "},
        issues = [IssueCode::InvalidStaticMethodCall]
    }

    test_analysis! {
        name = abstract_method_trait_parent_requirement,
        code = indoc! {"
            <?php

            abstract class BaseItem {
                abstract public static function getValue(): int;
            }

            class ParentItem extends BaseItem {
                public static function getValue(): int {
                    return 1;
                }
            }

            /** @phpstan-require-extends BaseItem */
            trait ItemTrait {
                public static function getTotal(): int {
                    return parent::getValue();
                }
            }

            final class Item extends ParentItem {
                use ItemTrait;
            }

            echo Item::getTotal();
        "}
    }

    test_analysis! {
        name = abstract_method_trait_self_class_string,
        code = indoc! {"
            <?php

            trait ItemTrait {
                abstract public static function getValue(): int;

                public static function getTotal(): int {
                    $class = self::class;
                    return $class::getValue();
                }
            }

            final class Item {
                use ItemTrait;

                public static function getValue(): int {
                    return 1;
                }
            }

            echo Item::getTotal();
        "}
    }

    test_analysis! {
        name = abstract_method_literal_string,
        code = indoc! {"
            <?php

            abstract class Item {
                abstract public static function getValue(): int;
            }

            $class = 'Item';
            echo $class::getValue();
        "},
        issues = [IssueCode::InvalidStaticMethodCall]
    }

    test_analysis! {
        name = abstract_method_unconstrained_string,
        code = indoc! {"
            <?php

            function getTotal(string $class): void {
                $class::getValue();
            }
        "}
    }

    test_analysis! {
        name = abstract_method_non_static_access_diagnostic,
        code = indoc! {"
            <?php

            abstract class Item {
                abstract public function getValue(): int;
            }

            Item::getValue();
        "},
        issues = [IssueCode::InvalidStaticMethodAccess]
    }

    test_analysis! {
        name = abstract_method_non_static_self_call,
        code = indoc! {"
            <?php

            abstract class Item {
                abstract public function getValue(): int;

                public function getTotal(): int {
                    return self::getValue();
                }
            }
        "},
        issues = [IssueCode::InvalidStaticMethodCall]
    }

    test_analysis! {
        name = abstract_method_interface_class_string_diagnostic,
        code = indoc! {"
            <?php

            interface Item {
                public static function getValue(): int;
            }

            $class = Item::class;
            $class::getValue();
        "},
        issues = [IssueCode::PossiblyStaticAccessOnInterface]
    }

    test_analysis! {
        name = abstract_method_self_call,
        code = indoc! {"
            <?php

            abstract class ParentItem {
                abstract protected static function getValue(): int;

                public static function getTotal(): int {
                    return self::getValue();
                }
            }

            final class ChildItem extends ParentItem {
                protected static function getValue(): int {
                    return 1;
                }
            }

            echo ChildItem::getTotal();
        "},
        issues = [IssueCode::InvalidStaticMethodCall]
    }

    test_analysis! {
        name = abstract_method_exact_calls,
        code = indoc! {"
            <?php

            abstract class ParentItem {
                abstract public static function getValue(): int;
            }

            final class ChildItem extends ParentItem {
                public static function getValue(): int {
                    return parent::getValue();
                }
            }

            echo ParentItem::getValue();
            $class = ParentItem::class;
            echo $class::getValue();
        "},
        issues = [
            IssueCode::InvalidStaticMethodCall,
            IssueCode::InvalidStaticMethodCall,
            IssueCode::InvalidStaticMethodCall,
        ]
    }

    test_analysis! {
        name = abstract_method_late_static_call,
        code = indoc! {"
            <?php

            abstract class ParentItem {
                abstract protected static function getValue(): int;

                public static function getTotal(): int {
                    return static::getValue();
                }
            }

            final class ChildItem extends ParentItem {
                protected static function getValue(): int {
                    return 1;
                }
            }

            echo ChildItem::getTotal();
        "}
    }

    test_analysis! {
        name = abstract_method_concrete_self_and_parent_calls,
        code = indoc! {"
            <?php

            abstract class ParentItem {
                protected static function getValue(): int {
                    return 1;
                }
            }

            final class ChildItem extends ParentItem {
                public static function getTotal(): int {
                    return self::getValue() + parent::getValue();
                }
            }

            echo ChildItem::getTotal();
        "}
    }

    test_analysis! {
        name = abstract_method_trait_self_requirement,
        code = indoc! {"
            <?php

            trait ItemTrait {
                abstract protected static function getValue(): int;

                public static function getTotal(): int {
                    return self::getValue();
                }
            }

            final class Item {
                use ItemTrait;

                protected static function getValue(): int {
                    return 1;
                }
            }

            echo Item::getTotal();
        "}
    }

    test_analysis! {
        name = abstract_method_runtime_receivers,
        code = indoc! {"
            <?php

            abstract class Item {
                abstract public static function getValue(): int;

                public function getTotal(): int {
                    return $this::getValue();
                }
            }

            /** @param class-string<Item> $class */
            function fromClass(string $class): int {
                return $class::getValue();
            }

            function fromObject(Item $item): int {
                return $item::getValue();
            }
        "}
    }

    test_analysis! {
        name = abstract_method_magic_probe,
        code = indoc! {"
            <?php

            abstract class Item {
                /** @param list<mixed> $arguments */
                abstract public static function __callStatic(string $name, array $arguments): mixed;

                public static function getValue(): int {
                    return 1;
                }

                public static function getTotal(): int {
                    return self::getValue();
                }
            }
        "}
    }

    test_analysis! {
        name = calling_non_static_method_statically_is_ok,
        code = indoc! {"
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
        code = indoc! {"
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
        code = indoc! {"
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
