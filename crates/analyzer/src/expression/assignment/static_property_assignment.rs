use std::borrow::Cow;
use std::rc::Rc;

use mago_codex::get_class_like;
use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::assignment::property_assignment::add_unspecialized_property_assignment_dataflow;
use crate::issue::TypingIssueKind;
use crate::resolver::class_name::resolve_classnames_from_expression;

pub(crate) fn analyze<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    expression: (&Expression, &Variable),
    assign_value_type: &TUnion,
    var_id: &Option<String>,
) -> Result<(), AnalysisError> {
    let class_expression = expression.0;
    let property_variable = expression.1;

    let property_name_id = match property_variable {
        Variable::Direct(direct_variable) => direct_variable.name,
        Variable::Indirect(indirect_variable) => {
            let Some(rc) = artifacts.get_rc_expression_type(indirect_variable.expression.as_ref()) else {
                return Err(AnalysisError::UserError(format!(
                    "Cannot resolve type for indirect variable `{}`",
                    indirect_variable.expression.span().start
                )));
            };

            let Some(prop_name_str) = rc.get_single().get_literal_string_value() else {
                return Err(AnalysisError::UserError(format!(
                    "Indirect variable `{}` does not resolve to a string",
                    indirect_variable.expression.span().start
                )));
            };

            context.interner.intern(format!("${prop_name_str}"))
        }
        Variable::Nested(nested_variable) => {
            let Some(rc) = artifacts.get_rc_expression_type(nested_variable.variable.as_ref()) else {
                return Err(AnalysisError::UserError(format!(
                    "Cannot resolve type for nested variable `{}`",
                    nested_variable.variable.span().start
                )));
            };

            let Some(prop_name_str) = rc.get_single().get_literal_string_value() else {
                return Err(AnalysisError::UserError(format!(
                    "Nested variable `{}` does not resolve to a string",
                    nested_variable.variable.span().start
                )));
            };

            context.interner.intern(format!("${prop_name_str}"))
        }
    };

    let resolved_names =
        resolve_classnames_from_expression(context, block_context, artifacts, class_expression, false)?;

    for resolved_name in resolved_names {
        let Some(fq_class_name) = resolved_name.fq_class_id else {
            // TODO: we should probably report an error here
            continue;
        };

        let property_id = (fq_class_name, property_name_id);

        artifacts.symbol_references.add_reference_to_class_member(&block_context.scope, property_id, false);

        let declaring_property_class =
            context.codebase.get_declaring_class_for_property(&fq_class_name, &property_id.1);

        if let Some(declaring_property_class) = declaring_property_class {
            let mut class_property_type =
                if let Some(prop_type) = context.codebase.get_property_type(&fq_class_name, &property_id.1) {
                    Cow::Borrowed(prop_type)
                } else {
                    Cow::Owned(get_mixed_any())
                };

            add_unspecialized_property_assignment_dataflow(
                context,
                artifacts,
                expression.1.span(),
                &fq_class_name,
                &property_name_id,
                assign_value_type,
            );

            let declaring_class_metadata = get_class_like(context.codebase, context.interner, &fq_class_name);

            if let Some(declaring_class_metadata) = declaring_class_metadata {
                let mut property_type = class_property_type.into_owned();

                expander::expand_union(
                    context.codebase,
                    context.interner,
                    &mut property_type,
                    &TypeExpansionOptions {
                        self_class: Some(&declaring_class_metadata.name),
                        static_class_type: StaticClassType::Name(declaring_class_metadata.name),
                        parent_class: declaring_class_metadata.get_direct_parent_class_ref(),
                        file_path: Some(&context.source.identifier),
                        ..Default::default()
                    },
                );

                class_property_type = Cow::Owned(property_type);
            }

            let mut union_comparison_result = ComparisonResult::new();

            let type_match_found = union_comparator::is_contained_by(
                context.codebase,
                context.interner,
                assign_value_type,
                &class_property_type,
                false,
                assign_value_type.ignore_falsable_issues,
                false,
                &mut union_comparison_result,
            );

            if type_match_found
                && union_comparison_result.replacement_union_type.is_some()
                && let Some(union_type) = union_comparison_result.replacement_union_type
                && let Some(var_id) = var_id.clone()
            {
                block_context.locals.insert(var_id, Rc::new(union_type));
            }

            if !type_match_found && union_comparison_result.type_coerced.is_none() {
                context.buffer.report(
                    TypingIssueKind::InvalidPropertyAssignmentValue,
                    Issue::error("Invalid property assignment value").with_annotation(
                        Annotation::primary(class_expression.span()).with_message(format!(
                            "{}::${} with declared type {}, cannot be assigned type {}",
                            context.interner.lookup(declaring_property_class),
                            context.interner.lookup(&property_id.1),
                            class_property_type.get_id(Some(context.interner)),
                            assign_value_type.get_id(Some(context.interner)),
                        )),
                    ),
                );
            }

            if union_comparison_result.type_coerced.is_some() {
                if union_comparison_result.type_coerced_from_nested_mixed.is_some() {
                    context.buffer.report(
                        TypingIssueKind::MixedPropertyTypeCoercion,
                        Issue::error("Mixed property type coercion").with_annotation(
                            Annotation::primary(class_expression.span()).with_message(format!(
                                "{} expects {}, parent type {} provided",
                                var_id.clone().unwrap_or("This property".to_string()),
                                class_property_type.get_id(Some(context.interner)),
                                assign_value_type.get_id(Some(context.interner)),
                            )),
                        ),
                    );
                } else {
                    context.buffer.report(
                        TypingIssueKind::PropertyTypeCoercion,
                        Issue::error("Property type coercion").with_annotation(
                            Annotation::primary(class_expression.span()).with_message(format!(
                                "{} expects {}, parent type {} provided",
                                var_id.clone().unwrap_or("This property".to_string()),
                                class_property_type.get_id(Some(context.interner)),
                                assign_value_type.get_id(Some(context.interner)),
                            )),
                        ),
                    );
                }
            }

            if let Some(var_id) = var_id.clone() {
                block_context.locals.insert(var_id, Rc::new(assign_value_type.clone()));
            }
        }
    }

    Ok(())
}
