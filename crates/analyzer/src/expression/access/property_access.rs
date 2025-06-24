use std::rc::Rc;

use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::property::resolve_instance_properties;
use crate::utils::expression::get_property_access_expression_id;

impl Analyzable for PropertyAccess {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_property_access(
            context,
            block_context,
            artifacts,
            self.span(),
            &self.object,
            self.arrow.span(),
            &self.property,
            false,
        )
    }
}

impl Analyzable for NullSafePropertyAccess {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_property_access(
            context,
            block_context,
            artifacts,
            self.span(),
            &self.object,
            self.question_mark_arrow.span(),
            &self.property,
            true,
        )
    }
}

#[inline]
fn analyze_property_access<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    span: Span,
    object: &Expression,
    arrow_span: Span,
    property_selector: &ClassLikeMemberSelector,
    is_null_safe: bool,
) -> Result<(), AnalysisError> {
    let resolution_result = resolve_instance_properties(
        context,
        block_context,
        artifacts,
        object,
        property_selector,
        arrow_span,
        is_null_safe,
        false, // `for_assignment`
    )?;

    let property_access_id = get_property_access_expression_id(
        object,
        property_selector,
        is_null_safe,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    if context.settings.memoize_properties
        && let Some(property_access_id) = &property_access_id
        && let Some(existing_type) = block_context.locals.get(property_access_id).cloned()
    {
        artifacts.set_rc_expression_type(&span, existing_type);

        return Ok(());
    }

    let mut resulting_expression_type = None;
    if !resolution_result.has_error_path {
        for resolved_property in resolution_result.properties {
            resulting_expression_type = Some(add_optional_union_type(
                resolved_property.property_type,
                resulting_expression_type.as_ref(),
                context.codebase,
                context.interner,
            ));
        }

        if resolution_result.has_ambiguous_path
            || resolution_result.encountered_mixed
            || resolution_result.has_possibly_defined_property
        {
            resulting_expression_type = Some(add_optional_union_type(
                get_mixed_any(),
                resulting_expression_type.as_ref(),
                context.codebase,
                context.interner,
            ));
        }

        if resolution_result.has_invalid_path || resolution_result.encountered_null {
            resulting_expression_type = Some(add_optional_union_type(
                get_null(),
                resulting_expression_type.as_ref(),
                context.codebase,
                context.interner,
            ));
        }
    }

    let resulting_type = Rc::new(resulting_expression_type.unwrap_or_else(get_never));
    if let Some(property_access_id) = property_access_id {
        block_context.locals.insert(property_access_id, resulting_type.clone());
    }

    artifacts.set_rc_expression_type(&span, resulting_type);

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = accessing_generic_property,
        code = indoc! {r#"
            <?php

            /**
             * @template K
             * @template V
             */
            class Collection
            {
                /**
                 * @var array<K, V>
                 */
                public $items = [];
            }

            /**
             * @param Collection<int, non-empty-string> $collection
             *
             * @return null|non-empty-string
             */
            function read_entrys(Collection $collection, int $key): null|string {
                return $collection->items[$key] ?? null;
            }
        "#}
    }

    test_analysis! {
        name = accessing_enum_properties,
        code = indoc! {r#"
            <?php

            enum Color: string {
                case Red = 'red';
                case Green = 'green';
                case Blue = 'blue';
            }

            /**
             * @return 'Red'|'Green'|'Blue'
             */
            function get_color_name(Color $color): string {
                return $color->name;
            }

            /**
             * @return 'red'|'green'|'blue'
             */
            function get_color_value(Color $color): string {
                return $color->value;
            }
        "#}
    }

    test_analysis! {
        name = redundant_nullsafe_property_access,
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(Foo $foo): void {
                echo $foo?->bar;
            }
        "#},
        issues = [
            TypingIssueKind::RedundantNullsafeOperator,
        ]
    }

    test_analysis! {
        name = accessing_property_on_null,
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                echo $foo->bar;
            }
        "#},
        issues = [
            TypingIssueKind::PossiblyNullPropertyAccess,
        ]
    }

    test_analysis! {
        name = accessing_property_on_nullsafe,
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                echo $foo?->bar;
            }
        "#},
        issues = []
    }

    test_analysis! {
        name = accessing_property_on_mixed,
        code = indoc! {r#"
            <?php

            function test(mixed $value): void {
                echo $value->bar;
            }
        "#},
        issues = [
            TypingIssueKind::MixedPropertyAccess,
            TypingIssueKind::MixedAnyArgument,
        ]
    }

    test_analysis! {
        name = accessing_property_on_non_object,
        code = indoc! {r#"
            <?php

            function test(int $value): void {
                echo $value->bar;
            }
        "#},
        issues = [
            TypingIssueKind::InvalidPropertyAccess,
        ]
    }

    test_analysis! {
        name = accessing_non_existent_property,
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar = '';
            }

            function i_take_null(null $_): void {}

            function test(Foo $foo): void {
                i_take_null($foo->baz);
            }
        "#},
        issues = [
            TypingIssueKind::NonExistentProperty,
            TypingIssueKind::MixedAnyArgument,
        ]
    }

    test_analysis! {
        name = accessing_property_on_generic_object,
        code = indoc! {r#"
            <?php

            /**
             * @template T
             */
            class GenericClass {
                /**
                 * @var T
                 */
                public $value;
            }

            /**
             * @param GenericClass<int> $generic
             *
             * @return int
             */
            function test(GenericClass $generic): int {
                return $generic->value;
            }
        "#}
    }

    test_analysis! {
        name = property_access_definite_null_error,
        code = indoc! {r#"
            <?php

            function test(): null {
                $obj = null;
                $value = $obj->property;
                return $value;
            }
        "#},
        issues = [
            TypingIssueKind::NullPropertyAccess,
        ]
    }

    test_analysis! {
        name = property_access_dynamic_name_unknown_type,
        code = indoc! {r#"
            <?php

            class Foo { public string $known = "val"; }

            function test(): string {
                $obj = new Foo();
                return $obj->{$propName};
             }
        "#},
        issues = [
            TypingIssueKind::UndefinedVariable,
            TypingIssueKind::InvalidMemberSelector,
            TypingIssueKind::MixedAnyReturnStatement,
        ]
    }

    test_analysis! {
        name = property_access_dynamic_name_literal_string,
        code = indoc! {r#"
            <?php
            class Foo { public string $known = "val"; }

            function test(): string {
                $obj = new Foo();
                $propName = "k" . "nown";
                return $obj->{$propName};
            }
        "#},
    }

    test_analysis! {
        name = property_access_dynamic_name_invalid_type,
        code = indoc! {r#"
            <?php

            class Foo { public string $known = "val"; }

            function test(): string {
                $obj = new Foo();
                $propName = 123;
                return $obj->{$propName};
            }
        "#},
        issues = [
            TypingIssueKind::InvalidMemberSelector,
            TypingIssueKind::InvalidReturnStatement,
        ]
    }

    test_analysis! {
        name = property_access_on_generic_object_type_error,
        code = indoc! {r#"
            <?php

            function get_prop(object $obj): mixed {
                return $obj->some_property;
            }
        "#},
        issues = [TypingIssueKind::AmbiguousObjectPropertyAccess]
    }

    test_analysis! {
        name = property_access_nullsafe_on_nullable_union_valid,
        code = indoc! {r#"
            <?php
            class Bar { public string $prop = "value"; }

            function get_prop_safe(Bar|null $obj): ?string {
                return $obj?->prop;
            }
        "#},
    }

    test_analysis! {
        name = property_access_on_interface_variable,
        code = indoc! {r#"
            <?php

            interface MyInterface {}

            function get_prop_from_interface(MyInterface $iface): null {
                return $iface->some_prop;
            }
        "#},
        issues = [
            TypingIssueKind::NonExistentProperty,
            TypingIssueKind::MixedAnyReturnStatement,
        ]
    }

    test_analysis! {
        name = property_access_on_enum_variable,
        code = indoc! {r#"
            <?php

            enum X {}

            function get_prop_from_enum(X $x): null {
                return $x->some_prop;
            }
        "#},
        issues = [
            TypingIssueKind::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = property_access_on_final_class_variable,
        code = indoc! {r#"
            <?php

            final class X {}

            function get_prop_from_final_class(X $x): null {
                return $x->some_prop;
            }
        "#},
        issues = [
            TypingIssueKind::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = property_access_chain_intermediate_possibly_null,
        code = indoc! {r#"
            <?php

            class A { public ?B $b_prop = null; }
            class B { public string $c_prop = "value"; }

            function get_nested(A $a): ?string {
                return $a->b_prop->c_prop;
            }
        "#},
        issues = [
            TypingIssueKind::PossiblyNullPropertyAccess
        ]
    }

    test_analysis! {
        name = property_access_chain_nullsafe_intermediate,
        code = indoc! {r#"
            <?php

            class A { public ?B $b_prop = null; }
            class B { public string $c_prop = "value"; }

            function get_nested_safe(A $a): ?string {
                return $a->b_prop?->c_prop;
            }
        "#},
    }

    test_analysis! {
        name = property_access_on_void_function_result,
        code = indoc! {r#"
            <?php

            function returns_void(): void {}

            $result = returns_void();
            echo $result->property;
        "#},
        issues = [
            TypingIssueKind::NullPropertyAccess,
        ]
    }

    test_analysis! {
        name = property_access_multiple_selectors,
        code = indoc! {r#"
            <?php

            class a
            {
                public string $x = '';
                public bool $y = false;
            }

            class b
            {
                public int $x = 1;
                public float $y = 1.2;
            }

            /**
             * @template T of scalar
             */
            class c
            {
                /**
                 * @var list<T>
                 */
                public array $x = [];

                public false $y = false;
            }

            /**
             * @param a|b|c<int> $o
             * @param 'x'|'y' $p
             * @return bool|float|string|int|list<int>
             */
            function get(a|b|c $o, string $p): bool|float|string|int|array
            {
                return $o->{$p};
            }
        "#},
    }
}
