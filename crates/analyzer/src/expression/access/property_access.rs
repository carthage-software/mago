use std::rc::Rc;

use mago_atom::Atom;
use mago_atom::concat_atom;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::NullSafePropertyAccess;
use mago_syntax::ast::PropertyAccess;
use mago_syntax::ast::Variable;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::property::resolve_instance_properties;
use crate::utils::expression::get_property_access_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PropertyAccess<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_property_access(
            context,
            block_context,
            artifacts,
            self.span(),
            self.object,
            self.arrow.span(),
            &self.property,
            false,
        )
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for NullSafePropertyAccess<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_property_access(
            context,
            block_context,
            artifacts,
            self.span(),
            self.object,
            self.question_mark_arrow.span(),
            &self.property,
            true,
        )
    }
}

#[inline]
fn analyze_property_access<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    span: Span,
    object: &'ast Expression<'arena>,
    arrow_span: Span,
    property_selector: &'ast ClassLikeMemberSelector<'arena>,
    is_null_safe: bool,
) -> Result<(), AnalysisError> {
    let property_access_id = get_property_access_expression_id(
        object,
        property_selector,
        is_null_safe,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    if context.settings.memoize_properties
        && let Some(property_access_id) = &property_access_id
        && let Some(existing_type) = block_context.locals.get(property_access_id).cloned()
    {
        add_memoized_property_reference(context, block_context, artifacts, object, property_selector)?;

        artifacts.set_rc_expression_type(&span, existing_type);

        return Ok(());
    }

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

    let mut resulting_expression_type = None;
    if !resolution_result.has_error_path {
        for resolved_property in resolution_result.properties {
            resulting_expression_type = Some(add_optional_union_type(
                resolved_property.property_type,
                resulting_expression_type.as_ref(),
                context.codebase,
            ));
        }

        if resolution_result.has_ambiguous_path
            || resolution_result.encountered_mixed
            || resolution_result.has_possibly_defined_property
        {
            resulting_expression_type =
                Some(add_optional_union_type(get_mixed(), resulting_expression_type.as_ref(), context.codebase));
        }

        if resolution_result.has_invalid_path || resolution_result.encountered_null {
            resulting_expression_type =
                Some(add_optional_union_type(get_null(), resulting_expression_type.as_ref(), context.codebase));
        }
    }

    let mut resulting_type = resulting_expression_type.unwrap_or_else(get_never);

    let object_has_nullsafe_null = artifacts.get_expression_type(object).is_some_and(|t| t.has_nullsafe_null());
    if resolution_result.all_properties_non_nullable
        && ((is_null_safe && resolution_result.encountered_null) || object_has_nullsafe_null)
    {
        resulting_type.set_nullsafe_null(true);
    }

    let resulting_type = Rc::new(resulting_type);
    if let Some(property_access_id) = property_access_id {
        block_context.locals.insert(property_access_id, resulting_type.clone());
    }

    artifacts.set_rc_expression_type(&span, resulting_type);

    Ok(())
}

/// Adds symbol reference for a memoized property access.
///
/// When property access is memoized, we still need to track the symbol reference
/// so that unused property detection works correctly.
fn add_memoized_property_reference<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    object: &'ast Expression<'arena>,
    property_selector: &'ast ClassLikeMemberSelector<'arena>,
) -> Result<(), AnalysisError> {
    let property_name = match property_selector {
        ClassLikeMemberSelector::Identifier(ident) => concat_atom!("$", ident.value),
        _ => return Ok(()),
    };

    let is_this = matches!(object, Expression::Variable(Variable::Direct(var)) if var.name == "$this");
    let mut declaring_classes: Vec<Atom> = Vec::new();

    if is_this {
        if let Some(class_name) = block_context.scope.get_class_like_name()
            && let Some(declaring_class) = context.codebase.get_declaring_property_class(&class_name, &property_name)
        {
            declaring_classes.push(declaring_class);
        }
    } else if let Some(object_type) = artifacts.get_rc_expression_type(object) {
        for atomic in object_type.types.iter() {
            for class_name in atomic.get_all_object_names() {
                if let Some(declaring_class) =
                    context.codebase.get_declaring_property_class(&class_name, &property_name)
                {
                    declaring_classes.push(declaring_class);
                }
            }
        }
    }

    // Add references (after releasing the immutable borrow)
    for declaring_class in declaring_classes {
        artifacts.symbol_references.add_reference_for_property_read(
            &block_context.scope,
            declaring_class,
            property_name,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = accessing_generic_property,
        code = indoc! {r"
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
            function read_entries(Collection $collection, int $key): null|string {
                return $collection->items[$key] ?? null;
            }
        "}
    }

    test_analysis! {
        name = accessing_enum_properties,
        code = indoc! {r"
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
        "}
    }

    test_analysis! {
        name = redundant_nullsafe_property_access,
        code = indoc! {r"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(Foo $foo): void {
                echo $foo?->bar;
            }
        "},
        issues = [
            IssueCode::RedundantNullsafeOperator,
        ]
    }

    test_analysis! {
        name = accessing_property_on_null,
        code = indoc! {r"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                echo $foo->bar;
            }
        "},
        issues = [
            IssueCode::PossiblyNullPropertyAccess,
        ]
    }

    test_analysis! {
        name = accessing_property_on_null_inside_coalescing,
        code = indoc! {r"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                $bar = $foo->bar ?? 'default';

                echo $bar;
            }
        "},
    }

    test_analysis! {
        name = accessing_property_on_null_inside_isset,
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                if (isset($foo->bar)) {
                    echo "Property exists";
                }
            }
        "#},
    }

    test_analysis! {
        name = accessing_property_on_nullsafe,
        code = indoc! {r"
            <?php

            class Foo {
                public string $bar = '';
            }

            function test(?Foo $foo): void {
                echo $foo?->bar;
            }
        "},
        issues = []
    }

    test_analysis! {
        name = accessing_property_on_mixed,
        code = indoc! {r"
            <?php

            function test(mixed $value): void {
                echo $value->bar;
            }
        "},
        issues = [
            IssueCode::MixedPropertyAccess,
            IssueCode::MixedArgument,
        ]
    }

    test_analysis! {
        name = accessing_property_on_non_object,
        code = indoc! {r"
            <?php

            function test(int $value): void {
                echo $value->bar;
            }
        "},
        issues = [
            IssueCode::InvalidPropertyAccess,
        ]
    }

    test_analysis! {
        name = accessing_non_existent_property,
        code = indoc! {r"
            <?php

            class Foo {
                public string $bar = '';
            }

            function i_take_null(null $_): void {}

            function test(Foo $foo): void {
                i_take_null($foo->baz);
            }
        "},
        issues = [
            IssueCode::NonExistentProperty,
            IssueCode::MixedArgument,
        ]
    }

    test_analysis! {
        name = accessing_property_on_generic_object,
        code = indoc! {r"
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
        "}
    }

    test_analysis! {
        name = property_access_definite_null_error,
        code = indoc! {r"
            <?php

            function test(): null {
                $obj = null;
                $value = $obj->property;
                return $value;
            }
        "},
        issues = [
            IssueCode::NullPropertyAccess,
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
            IssueCode::UndefinedVariable,
            IssueCode::InvalidMemberSelector,
            IssueCode::MixedReturnStatement,
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
            IssueCode::InvalidMemberSelector,
            IssueCode::InvalidReturnStatement,
        ]
    }

    test_analysis! {
        name = property_access_on_generic_object_type_error,
        code = indoc! {r"
            <?php

            function get_prop(object $obj): mixed {
                return $obj->some_property;
            }
        "},
        issues = [IssueCode::AmbiguousObjectPropertyAccess]
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
        code = indoc! {r"
            <?php

            interface MyInterface {}

            function get_prop_from_interface(MyInterface $iface): null {
                return $iface->some_prop;
            }
        "},
        issues = [
            IssueCode::NonExistentProperty,
            IssueCode::MixedReturnStatement,
        ]
    }

    test_analysis! {
        name = property_access_on_enum_variable,
        code = indoc! {r"
            <?php

            enum X {}

            function get_prop_from_enum(X $x): null {
                return $x->some_prop;
            }
        "},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = property_access_on_final_class_variable,
        code = indoc! {r"
            <?php

            final class X {}

            function get_prop_from_final_class(X $x): null {
                return $x->some_prop;
            }
        "},
        issues = [
            IssueCode::NonExistentProperty,
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
            IssueCode::PossiblyNullPropertyAccess
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
        code = indoc! {r"
            <?php

            function returns_void(): void {}

            $result = returns_void();
            echo $result->property;
        "},
        issues = [
            IssueCode::NullPropertyAccess,
        ]
    }

    test_analysis! {
        name = property_access_multiple_selectors,
        code = indoc! {r"
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
        "},
    }

    test_analysis! {
        name = accessing_non_existent_class_property,
        code = indoc! {r"
            <?php

            function example($class): void {
                if ($class instanceof NonExistingClass) {
                    $class->bar;
                }
            }
        "},
        issues = [
            IssueCode::NonExistentClassLike,
            IssueCode::UnusedStatement,
        ]
    }

    test_analysis! {
        name = trait_property_access,
        code = indoc! {r#"
            <?php

            trait A {
                private string $x = "hello 1";
                protected string $y = "hello 2";
            }

            class B {
                use A;

                public function c(): void {
                    echo $this->x; // private property access
                    echo $this->y; // protected property access
                }
            }

            new B()->c();
        "#},
    }
}
