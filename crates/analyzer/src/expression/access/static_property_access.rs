use std::rc::Rc;

use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::static_property::resolve_static_properties;
use crate::utils::expression::get_static_property_access_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for StaticPropertyAccess<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let property_access_id = get_static_property_access_expression_id(
            self.class,
            &self.property,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            Some(context.codebase),
        );

        if context.settings.memoize_properties
            && let Some(property_access_id) = &property_access_id
            && let Some(existing_type) = block_context.locals.get(property_access_id).cloned()
        {
            artifacts.set_rc_expression_type(self, existing_type);

            return Ok(());
        }

        let resolution_result =
            resolve_static_properties(context, block_context, artifacts, self.class, &self.property)?;

        let mut resulting_expression_type = None;
        if !resolution_result.has_error_path {
            for resolved_property in resolution_result.properties {
                artifacts.symbol_references.add_reference_to_class_member(
                    &block_context.scope,
                    (resolved_property.declaring_class_id, resolved_property.property_name),
                    false,
                );

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

        let resulting_type = Rc::new(resulting_expression_type.unwrap_or_else(get_never));
        if let Some(property_access_id) = property_access_id {
            block_context.locals.insert(property_access_id, resulting_type.clone());
        }

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::code::IssueCode;
    use crate::test_analysis;
    use indoc::indoc;

    test_analysis! {
        name = read_public_static_property_directly,
        code = indoc! {r#"
            <?php
            class MyClass { public static string $prop = "value"; }
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            i_take_string(MyClass::$prop);
        "#},
    }

    test_analysis! {
        name = read_public_static_property_from_parent,
        code = indoc! {r#"
            <?php
            class ParentClass { public static string $prop = "value"; }
            class ChildClass extends ParentClass {}
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            i_take_string(ChildClass::$prop);
        "#},
    }

    test_analysis! {
        name = read_protected_static_property_from_child,
        code = indoc! {r#"
            <?php
            class ParentClass { protected static int $prop = 1; }
            class ChildClass extends ParentClass {
                public static function getValue(): int {
                    return self::$prop;
                }
            }
            /** @param int $_i */
            function i_take_int(int $_i): void {}
            i_take_int(ChildClass::getValue());
        "#},
    }

    test_analysis! {
        name = read_private_static_property_from_same_class,
        code = indoc! {r#"
            <?php
            class PrivateTest {
                private static int $secret = 42;
                public static function getSecret(): int {
                    return self::$secret;
                }
            }
        "#},
    }

    test_analysis! {
        name = read_static_property_with_static_keyword,
        code = indoc! {r#"
            <?php
            class Base { public static string $name = "Base"; }
            class Sub extends Base { public static string $name = "Sub"; }
            function get_name(Base $instance): string {
                return $instance::$name;
            }
        "#},
    }

    test_analysis! {
        name = read_static_property_dynamically_with_literal,
        code = indoc! {r#"
            <?php
            class MyClass { public static string $prop = "value"; }
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            $propName = 'prop';
            i_take_string(MyClass::${$propName});
        "#},
    }

    test_analysis! {
        name = read_undefined_static_property,
        code = indoc! {r#"
            <?php
            class MyClass {}
            echo MyClass::$non_existent;
        "#},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = read_private_static_property_from_outside,
        code = indoc! {r#"
            <?php
            class MyClass { private static int $secret = 1; }
            echo MyClass::$secret;
        "#},
        issues = [
            IssueCode::InvalidPropertyRead,
            IssueCode::NoValue,
        ]
    }

    test_analysis! {
        name = read_private_static_property_from_parent_in_child,
        code = indoc! {r#"
            <?php
            class P { private static int $secret = 1; }
            class C extends P {
                public function getSecret() {
                    return self::$secret;
                }
            }
        "#},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = read_protected_static_property_from_outside,
        code = indoc! {r#"
            <?php
            class MyClass { protected static int $prop = 1; }
            echo MyClass::$prop;
        "#},
        issues = [
            IssueCode::InvalidPropertyRead,
            IssueCode::NoValue,
        ]
    }

    test_analysis! {
        name = read_instance_property_statically,
        code = indoc! {r#"
            <?php
            class HasInstanceProp { public int $instance_prop = 1; }
            echo HasInstanceProp::$instance_prop;
        "#},
        issues = [
            IssueCode::InvalidStaticPropertyAccess,
            IssueCode::NoValue,
        ]
    }

    test_analysis! {
        name = read_static_property_on_interface,
        code = indoc! {r#"
            <?php
            interface MyInterface {}
            echo MyInterface::$some_prop;
        "#},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = read_static_property_on_enum,
        code = indoc! {r#"
            <?php
            enum MyEnum {}
            echo MyEnum::$some_prop;
        "#},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = reading_static_property_with_union_type,
        code = indoc! {r#"
            <?php

            class A {
                public static null|int $x = 1;
                public static null|bool $y = true;
            }

            class B {
                public static null|float $x = 2.5;
                public static null|string $y = "hello";
            }

            /** @param 'x'|'y' $prop */
            function delta(A|B $obj, string $prop): int|float|bool|string|null {
                return $obj::${$prop};
            }
        "#},
    }

    test_analysis! {
        name = static_property_reconciliation,
        code = indoc! {r#"
            <?php

            class A
            {
                private static null|string $foo = null;

                public static function getFoo(): string
                {
                    if (self::$foo !== null) {
                        return self::$foo;
                    }

                    self::$foo = 'bar';

                    return self::$foo;
                }
            }
        "#},
    }
}
