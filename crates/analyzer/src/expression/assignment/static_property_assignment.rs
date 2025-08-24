use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::static_property::resolve_static_properties;

pub(crate) fn analyze<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    property_access: &StaticPropertyAccess<'arena>,
    assigned_value_type: &TUnion,
    property_access_id: &Option<String>,
) -> Result<(), AnalysisError> {
    let property_resolution =
        resolve_static_properties(context, block_context, artifacts, property_access.class, &property_access.property)?;

    let mut resolved_property_type = None;
    let mut matched_all_properties = true;
    for resolved_property in property_resolution.properties {
        artifacts.symbol_references.add_reference_to_class_member(
            &block_context.scope,
            (resolved_property.declaring_class_id, resolved_property.property_name),
            false,
        );

        let mut union_comparison_result = ComparisonResult::new();

        let type_match_found = union_comparator::is_contained_by(
            context.codebase,
            assigned_value_type,
            &resolved_property.property_type,
            assigned_value_type.ignore_nullable_issues,
            assigned_value_type.ignore_falsable_issues,
            false,
            &mut union_comparison_result,
        );

        if !type_match_found && union_comparison_result.type_coerced.is_none() {
            context.collector.report_with_code(
                IssueCode::InvalidPropertyAssignmentValue,
                Issue::error("Invalid property assignment value").with_annotation(
                    Annotation::primary(property_access.class.span()).with_message(format!(
                        "{}::{} with declared type {}, cannot be assigned type {}",
                        resolved_property.declaring_class_id,
                        resolved_property.property_name,
                        resolved_property.property_type.get_id(),
                        assigned_value_type.get_id(),
                    )),
                ),
            );
        }

        if union_comparison_result.type_coerced.is_some() {
            if union_comparison_result.type_coerced_from_nested_mixed.is_some() {
                context.collector.report_with_code(
                    IssueCode::MixedPropertyTypeCoercion,
                    Issue::error("Mixed property type coercion").with_annotation(
                        Annotation::primary(property_access.class.span()).with_message(format!(
                            "{} expects {}, parent type {} provided",
                            property_access_id.clone().unwrap_or("This property".to_string()),
                            resolved_property.property_type.get_id(),
                            assigned_value_type.get_id(),
                        )),
                    ),
                );
            } else {
                context.collector.report_with_code(
                    IssueCode::PropertyTypeCoercion,
                    Issue::error("Property type coercion").with_annotation(
                        Annotation::primary(property_access.class.span()).with_message(format!(
                            "{} expects {}, parent type {} provided",
                            property_access_id.clone().unwrap_or("This property".to_string()),
                            resolved_property.property_type.get_id(),
                            assigned_value_type.get_id(),
                        )),
                    ),
                );
            }
        }

        if let Some(var_id) = property_access_id.clone() {
            block_context.locals.insert(var_id, Rc::new(assigned_value_type.clone()));
        }

        resolved_property_type = Some(add_optional_union_type(
            resolved_property.property_type,
            resolved_property_type.as_ref(),
            context.codebase,
        ));

        matched_all_properties &= type_match_found;
    }

    let mut resulting_type = if matched_all_properties && context.settings.memoize_properties {
        Some(assigned_value_type.clone())
    } else {
        resolved_property_type
    };

    if property_resolution.has_ambiguous_path
        || property_resolution.encountered_mixed
        || property_resolution.has_possibly_defined_property
    {
        resulting_type = Some(add_optional_union_type(get_mixed(), resulting_type.as_ref(), context.codebase));
    }

    if property_resolution.has_error_path
        || property_resolution.has_invalid_path
        || property_resolution.encountered_null
    {
        resulting_type = Some(add_optional_union_type(get_never(), resulting_type.as_ref(), context.codebase));
    }

    let resulting_type = Rc::new(resulting_type.unwrap_or_else(get_never));

    if context.settings.memoize_properties
        && let Some(property_access_id) = property_access_id
    {
        block_context.locals.insert(property_access_id.clone(), resulting_type.clone());
    }

    artifacts.set_rc_expression_type(property_access, resulting_type);

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = write_public_static_property,
        code = indoc! {r#"
            <?php
            class MyClass { public static string $prop = ""; }

            /** @param string $_s */
            function i_take_string(string $_s): void {}

            MyClass::$prop = "new value";
            i_take_string(MyClass::$prop);
        "#},
    }

    test_analysis! {
        name = write_protected_static_property_from_child,
        code = indoc! {r#"
            <?php
            class ParentClass { protected static int $prop = 1; }
            class ChildClass extends ParentClass {
                public static function setProp(int $val): void {
                    self::$prop = $val;
                    parent::$prop = $val + 1;
                }
            }
        "#},
    }

    test_analysis! {
        name = write_private_static_property_from_same_class,
        code = indoc! {r#"
            <?php
            class PrivateWriteTest {
                private static int $value = 0;
                public static function setValue(int $new): void {
                    self::$value = $new;
                }
            }
        "#},
    }

    test_analysis! {
        name = write_wrong_type_to_typed_static_property,
        code = indoc! {r#"
            <?php
            class MyClass { public static string $prop = ""; }
            MyClass::$prop = 123;
        "#},
        issues = [
            IssueCode::InvalidPropertyAssignmentValue,
        ]
    }

    test_analysis! {
        name = write_to_undefined_static_property,
        code = indoc! {r#"
            <?php
            class MyClass {}
            MyClass::$undefined = 'new';
        "#},
        issues = [
            IssueCode::NonExistentProperty,
        ]
    }

    test_analysis! {
        name = write_private_static_property_from_outside,
        code = indoc! {r#"
            <?php
            class PrivateWrite { private static int $value = 0; }
            PrivateWrite::$value = 1;
        "#},
        issues = [
            IssueCode::InvalidPropertyRead,
        ]
    }

    test_analysis! {
        name = write_protected_static_property_from_outside,
        code = indoc! {r#"
            <?php
            class MyClass { protected static int $prop = 1; }
            MyClass::$prop = 500;
        "#},
        issues = [
            IssueCode::InvalidPropertyRead,
        ]
    }

    test_analysis! {
        name = assigning_static_property_with_union_type,
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
            function delta(A|B $obj, string $prop): void {
                $obj::${$prop} = null;
                $obj::$$prop = null;
            }
        "#},
    }
}
